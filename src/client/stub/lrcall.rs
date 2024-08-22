// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu that supports both local calls and remote calls.
#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Ok;

use crate::client::stub::{Config, Stub};
use crate::client::{Channel, RpcError};
use crate::discover::ServiceInfo;
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
    async fn new_lb_stub(&self, info_list: Vec<ServiceInfo>) {}
    async fn new_channels_with_info(&self, info_list: Vec<ServiceInfo>) -> Vec<StubWithMeta<Channel<Serve::Req, Serve::Resp>>> {
        let mut stub_list = vec![];
        for service_info in info_list {
            match service_info.call_type {
                discover::CallType::Local => {},
                discover::CallType::Remote => match self.new_channel(service_info.address.as_str()).await {
                    core::result::Result::Ok(stub) => {
                        stub_list.push(StubWithMeta { service_info, stub });
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
    async fn new_channel(&self, address: &str) -> Result<Channel<Serve::Req, Serve::Resp>, anyhow::Error> {
        let mut transport = crate::serde_transport::tcp::connect(address, crate::client::stub::formats::Json::default);
        transport.config_mut().max_frame_length(usize::MAX);
        Ok(tarpc::client::new(self.stub_config.clone(), transport.await?).spawn())
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

struct StubWithMeta<Stub> {
    service_info: ServiceInfo,
    stub: Stub,
}

impl<Stub> crate::client::stub::Stub for StubWithMeta<Stub>
where
    Stub: crate::client::stub::Stub,
{
    type Req = Stub::Req;
    type Resp = Stub::Resp;

    async fn call(&self, ctx: context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        self.stub.call(ctx, request).await
    }
}
