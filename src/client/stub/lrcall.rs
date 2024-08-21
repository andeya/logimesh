// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu that supports both local calls and remote calls.
#![allow(dead_code)]

use crate::client::stub::Stub;
use crate::client::RpcError;
use crate::{context, discover, server};

/// A client stbu config.
pub struct Config<ServiceLookup> {
    service_lookup: ServiceLookup,
}

/// A client stbu that supports both local calls and remote calls.
pub struct LRCall<Serve, ServiceLookup> {
    config: Config<ServiceLookup>,
    serve: Serve,
}

impl<Serve, ServiceLookup> LRCall<Serve, ServiceLookup>
where
    Serve: server::Serve + Clone,
    ServiceLookup: discover::ServiceLookup,
{
    /// Return a new client stbu that supports both local calls and remote calls.
    pub fn new(serve: impl Into<Serve>, config: Config<ServiceLookup>) -> Self {
        Self { serve: serve.into(), config }
    }
}

impl<Serve, ServiceLookup> Stub for LRCall<Serve, ServiceLookup>
where
    Serve: server::Serve + Clone,
    ServiceLookup: discover::ServiceLookup,
{
    type Req = Serve::Req;
    type Resp = Serve::Resp;

    async fn call(&self, _: context::Context, _request: Self::Req) -> Result<Self::Resp, RpcError> {
        todo!()
    }
}
