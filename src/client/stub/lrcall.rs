// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu that supports both local calls and remote calls.
#![allow(dead_code)]

use core::alloc;
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

use futures::future::{BoxFuture, FutureExt};

use crate::client::stub::config::TransportCodec;
use crate::client::stub::{formats, LRConfig, Stub};
use crate::client::{Channel, RpcError};
use crate::discover::{CallType, ServiceInfo};
use crate::serde_transport::tcp;
use crate::{context, discover, server};

/// Create a builder for a client stub that supports both local and remote calls.
pub struct NewLRCall<Serve, ServiceLookup, RetryFn>
where
    Serve: server::Serve,
{
    lrcall: LRCall<Serve, ServiceLookup, RetryFn>,
}

impl<Serve, ServiceLookup, RetryFn> NewLRCall<Serve, ServiceLookup, RetryFn>
where
    Serve: server::Serve + Clone,
    Serve::Req: crate::serde::Serialize + Send + 'static,
    Serve::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    ServiceLookup: discover::ServiceLookup,
    RetryFn: Fn(&Result<Serve::Resp, RpcError>, u32) -> bool,
{
    /// Return a new client stbu that supports both local calls and remote calls.
    pub fn new(serve: Serve, config: LRConfig<ServiceLookup>, retry_fn: RetryFn) -> Self {
        let mut stub_config = tarpc::client::Config::default();
        stub_config.max_in_flight_requests = config.max_in_flight_requests;
        stub_config.pending_request_buffer = config.pending_request_buffer;
        Self {
            lrcall: LRCall {
                serve,
                config,
                stub_config,
                retry_fn,
                call_type: Cell::new(CallType::Local),
                rpc_channels: RefCell::new(vec![]),
                warm_up_error: None,
            },
        }
    }

    /// Spawn a client stbu that supports both local calls and remote calls.
    pub async fn spawn(self) -> Result<LRCall<Serve, ServiceLookup, RetryFn>, RpcError> {
        self.lrcall.warm_up().await
    }
}

/// A client stbu that supports both local calls and remote calls.
pub struct LRCall<Serve, ServiceLookup, RetryFn>
where
    Serve: server::Serve,
{
    config: LRConfig<ServiceLookup>,
    stub_config: tarpc::client::Config,
    serve: Serve,
    retry_fn: RetryFn,
    call_type: Cell<CallType>,
    rpc_channels: RefCell<Vec<ChannelWithInfo<Serve>>>,
    warm_up_error: Option<anyhow::Error>,
}

impl<Serve, ServiceLookup, RetryFn> LRCall<Serve, ServiceLookup, RetryFn>
where
    Serve: server::Serve + Clone,
    Serve::Req: crate::serde::Serialize + Send + 'static,
    Serve::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    ServiceLookup: discover::ServiceLookup,
    RetryFn: Fn(&Result<Serve::Resp, RpcError>, u32) -> bool,
{
    /// Warm up client stubs.
    async fn warm_up(self) -> Result<Self, RpcError> {
        self.init_lb_stub().await?;
        Ok(self)
    }
    fn lookup_service(&self) -> anyhow::Result<Arc<ServiceInfo>> {
        self.config.service_lookup.lookup_service(self.config.service_name.as_str())
    }
    async fn init_lb_stub(&self) -> Result<(), RpcError> {
        // TODO
        let service_info = self.lookup_service();
        match service_info {
            Ok(service_info) => *self.rpc_channels.borrow_mut() = self.new_channels_with_info(service_info).await?,
            Err(e) => {
                // TODO
                println!("{:?}", e)
            },
        }
        Ok(())
    }
    async fn new_channels_with_info(&self, service_info: Arc<ServiceInfo>) -> Result<Vec<ChannelWithInfo<Serve>>, RpcError> {
        let mut stub_list = vec![];
        match service_info.call_type {
            discover::CallType::Local => {},
            discover::CallType::Remote => {
                for address in service_info.addresses.clone() {
                    match ChannelWithInfo::new(address, self.config.transport_codec.clone(), self.stub_config.clone()).await {
                        core::result::Result::Ok(channel) => {
                            stub_list.push(channel);
                        },
                        Err(e) => {
                            // TODO
                            println!("{:?}", e)
                        },
                    }
                }
            },
        }
        self.call_type.set(service_info.call_type);
        Ok(stub_list)
    }
}

impl<Serve, ServiceLookup, RetryFn> Stub for LRCall<Serve, ServiceLookup, RetryFn>
where
    Serve: server::Serve + Clone,
    ServiceLookup: discover::ServiceLookup,
    RetryFn: Fn(&Result<Serve::Resp, RpcError>, u32) -> bool,
{
    type Req = Serve::Req;
    type Resp = Serve::Resp;

    async fn call(&self, ctx: context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        match self.call_type.get() {
            CallType::Local => self.serve.call(ctx, request).await,
            CallType::Remote => match self.rpc_channels.borrow().get(0) {
                Some(chan) => chan.call(ctx, request).await,
                None => Err(RpcError::Shutdown),
            },
        }
    }
}

struct ChannelWithInfo<Serve: server::Serve> {
    address: String,
    transport_codec: TransportCodec,
    stub_config: tarpc::client::Config,
    channel: Channel<Serve::Req, Serve::Resp>,
    is_shutdown: Cell<bool>,
}

impl<Serve: server::Serve> crate::client::stub::Stub for ChannelWithInfo<Serve> {
    type Req = Serve::Req;
    type Resp = Serve::Resp;

    async fn call(&self, ctx: context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        let res = self.channel.call(ctx, request).await;
        if let Err(RpcError::Shutdown) = res {
            self.is_shutdown.set(true);
        }
        res
    }
}

impl<Serve: server::Serve> ChannelWithInfo<Serve>
where
    Serve: server::Serve + Clone,
    Serve::Req: crate::serde::Serialize + Send + 'static,
    Serve::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
{
    async fn new(address: String, transport_codec: TransportCodec, stub_config: tarpc::client::Config) -> Result<Self, anyhow::Error> {
        let channe = Self::new_channel(transport_codec.clone(), stub_config.clone(), address.as_str()).await?;
        Ok(Self {
            address,
            transport_codec,
            stub_config,
            channel: channe,
            is_shutdown: Cell::new(false),
        })
    }
    async fn new_channel(transport_codec: TransportCodec, stub_config: tarpc::client::Config, address: &str) -> Result<Channel<Serve::Req, Serve::Resp>, anyhow::Error> {
        match transport_codec {
            TransportCodec::Bincode => {
                // Bincode codec using [bincode](https://docs.rs/bincode) crate.
                let mut conn = tcp::connect(address, formats::Bincode::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            TransportCodec::Json => {
                // JSON codec using [serde_json](https://docs.rs/serde_json) crate.
                let mut conn = tcp::connect(address, formats::Json::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-messagepack")]
            MessagePack => {
                /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
                let mut conn = tcp::connect(address, formats::MessagePack::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-cbor")]
            Cbor => {
                /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
                let mut conn = tcp::connect(address, formats::Cbor::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
        }
    }
    async fn reconnent(&mut self) -> Result<(), anyhow::Error> {
        self.channel = Self::new_channel(self.transport_codec.clone(), self.stub_config.clone(), self.address.as_str()).await?;
        self.is_shutdown.set(false);
        Ok(())
    }
}

impl<Serve> tower::Service<(context::Context, Serve::Req)> for ChannelWithInfo<Serve>
where
    Serve: server::Serve + Clone + Send + Sync,
    Serve::Req: crate::serde::Serialize + Send + 'static,
    Serve::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
{
    type Response = Serve::Resp;
    type Error = RpcError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    // type Future = impl Future<Output = Result<Self::Response, Self::Error>> + Send;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, (ctx, request): (context::Context, Serve::Req)) -> Self::Future {
        let chan = self.channel.clone();
        async move {
            let result = chan.call(ctx, request).await;
            result.map_err(|error| error.into())
        }
        .boxed()
    }
}
