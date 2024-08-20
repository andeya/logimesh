// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu that supports both local calls and remote calls.
#![allow(dead_code)]

use crate::client::stub::Stub;
use crate::client::RpcError;
use crate::{context, server};

/// A client stbu config.
pub struct Config<ServLookup> {
    serv_lookup: ServLookup,
}

/// A client stbu that supports both local calls and remote calls.
pub struct LRCall<Serve, ServLookup> {
    config: Config<ServLookup>,
    serve: Serve,
}

impl<Serve, ServLookup> LRCall<Serve, ServLookup>
where
    Serve: server::Serve + Clone,
{
    /// Return a new client stbu that supports both local calls and remote calls.
    pub fn new(serve: impl Into<Serve>, config: Config<ServLookup>) -> Self {
        Self { serve: serve.into(), config }
    }
}

impl<Serve, ServLookup> Stub for LRCall<Serve, ServLookup>
where
    Serve: server::Serve + Clone,
{
    type Req = Serve::Req;
    type Resp = Serve::Resp;

    async fn call(&self, _: context::Context, _request: Self::Req) -> Result<Self::Resp, RpcError> {
        todo!()
    }
}
