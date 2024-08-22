// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu that supports both local calls and remote calls.
#![allow(dead_code)]

use std::sync::Arc;

use crate::client::stub::config::TransportCodec;
use crate::client::stub::{formats, Config, Stub};
use crate::client::{Channel, RpcError};
use crate::discover::ServiceInfo;
use crate::serde_transport::tcp;
use crate::{context, discover, server};
use anyhow::Ok;

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
                discover::CallType::Remote => match self.new_channel(service_info.address.as_str()).await {
                    core::result::Result::Ok(channel) => {
                        stub_list.push(ChannelWithInfo { service_info, channel });
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
    async fn reconnent(&self, chan_with_info: &mut ChannelWithInfo<Serve>) -> Result<(), anyhow::Error> {
        chan_with_info.channel = self.new_channel(chan_with_info.service_info.address.as_str()).await?;
        Ok(())
    }
    async fn new_channel(&self, address: &str) -> Result<Channel<Serve::Req, Serve::Resp>, anyhow::Error> {
        match self.config.transport_codec {
            TransportCodec::Bincode => {
                // Bincode codec using [bincode](https://docs.rs/bincode) crate.
                let mut conn = tcp::connect(address, formats::Bincode::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(self.stub_config.clone(), conn.await?).spawn())
            },
            TransportCodec::Json => {
                // JSON codec using [serde_json](https://docs.rs/serde_json) crate.
                let mut conn = tcp::connect(address, formats::Json::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(self.stub_config.clone(), conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-messagepack")]
            MessagePack => {
                /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
                let mut conn = tcp::connect(address, formats::MessagePack::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(self.stub_config.clone(), conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-cbor")]
            Cbor => {
                /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
                let mut conn = tcp::connect(address, formats::Cbor::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(self.stub_config.clone(), conn.await?).spawn())
            },
        }
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
    channel: Channel<Serve::Req, Serve::Resp>,
}

impl<Serve: server::Serve> crate::client::stub::Stub for ChannelWithInfo<Serve> {
    type Req = Serve::Req;
    type Resp = Serve::Resp;

    async fn call(&self, ctx: context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        self.channel.call(ctx, request).await
    }
}
