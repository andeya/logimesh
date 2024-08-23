// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu that supports both local calls and remote calls.
#![allow(dead_code)]

use std::cell::Cell;
use std::sync::Arc;

use crate::client::stub::config::TransportCodec;
use crate::client::stub::{formats, Config, Stub};
use crate::client::{Channel, RpcError};
use crate::discover::ServiceInfo;
use crate::serde_transport::tcp;
use crate::{context, discover, server};

/// A client stbu that supports both local calls and remote calls.
pub struct LRCall<Serve, ServiceLookup, RetryFn> {
    config: Config<ServiceLookup>,
    stub_config: tarpc::client::Config,
    serve: Serve,
    retry_fn: RetryFn,
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
    /// Return a new client stbu that supports both local calls and remote calls.
    pub fn new(serve: impl Into<Serve>, config: Config<ServiceLookup>, retry_fn: RetryFn) -> Self {
        let mut stub_config = tarpc::client::Config::default();
        stub_config.max_in_flight_requests = config.max_in_flight_requests;
        stub_config.pending_request_buffer = config.pending_request_buffer;
        Self {
            serve: serve.into(),
            config,
            stub_config,
            retry_fn,
            warm_up_error: None,
        }
        .warm_up()
    }
    fn warm_up(self) -> Self {
        self
    }
    fn lookup_service(&self) -> anyhow::Result<Arc<Vec<ServiceInfo>>> {
        self.config.service_lookup.lookup_service(self.config.service_name.as_str())
    }
    async fn new_lb_stub(&self, _info_list: Vec<ServiceInfo>) {}
    async fn new_channels_with_info(&self, info_list: Vec<ServiceInfo>) -> Vec<ChannelWithInfo<Serve>> {
        let mut stub_list = vec![];
        for service_info in info_list {
            match service_info.call_type {
                discover::CallType::Local => {},
                discover::CallType::Remote => match ChannelWithInfo::new(service_info, self.config.transport_codec.clone(), self.stub_config.clone()).await {
                    core::result::Result::Ok(channel) => {
                        stub_list.push(channel);
                    },
                    Err(e) => {
                        // TODO
                        println!("{:?}", e)
                    },
                },
            }
        }
        stub_list
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

    async fn call(&self, _: context::Context, _request: Self::Req) -> Result<Self::Resp, RpcError> {
        todo!()
    }
}

struct ChannelWithInfo<Serve: server::Serve> {
    service_info: ServiceInfo,
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
    async fn new(service_info: ServiceInfo, transport_codec: TransportCodec, stub_config: tarpc::client::Config) -> Result<Self, anyhow::Error> {
        let channe = Self::new_channel(transport_codec.clone(), stub_config.clone(), service_info.address.as_str()).await?;
        Ok(Self {
            service_info,
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
        self.channel = Self::new_channel(self.transport_codec.clone(), self.stub_config.clone(), self.service_info.address.as_str()).await?;
        self.is_shutdown.set(false);
        Ok(())
    }
}
