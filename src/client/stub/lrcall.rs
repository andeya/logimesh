// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Local and remote Cclient.

use crate::client::balance::LoadBalance;
use crate::client::channel::Codec;
use crate::client::discover::Discover;
use crate::client::stub::Stub;
use crate::client::{CoreConfig, RpcError};
use crate::component::Component;
use crate::server::Serve;
use crate::BoxError;
use thiserror::Error;

/// A full client stbu config.
#[non_exhaustive]
pub struct Builder<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    /// The implementer of the local service.
    pub(crate) component: Component<S>,
    /// discover instance.
    pub(crate) discover: D,
    /// load balance instance.
    pub(crate) load_balance: LB,
    /// transport codec type.
    pub(crate) transport_codec: Codec,
    /// Settings that control the behavior of the underlying client.
    pub(crate) core_config: CoreConfig,
    /// A callback function for judging whether to re-initiate the request.
    pub(crate) retry_fn: Option<RF>,
}

impl<S, D, LB, RF> Builder<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    /// Create a LRCall builder.
    pub fn new(component: Component<S>, discover: D, load_balance: LB) -> Self {
        Self {
            component,
            discover,
            load_balance,
            transport_codec: Default::default(),
            core_config: Default::default(),
            retry_fn: None,
        }
    }
    /// Set transport serde codec
    pub fn with_transport_codec(mut self, transport_codec: Codec) -> Self {
        self.transport_codec = transport_codec;
        self
    }
    /// The number of requests that can be in flight at once.
    /// `max_in_flight_requests` controls the size of the map used by the client
    /// for storing pending requests.
    /// Default is 1000.
    pub fn with_max_in_flight_requests(mut self, max_in_flight_requests: usize) -> Self {
        self.core_config.max_in_flight_requests = max_in_flight_requests;
        self
    }
    /// The number of requests that can be buffered client-side before being sent.
    /// `pending_requests_buffer` controls the size of the channel clients use
    /// to communicate with the request dispatch task.
    /// Default is 100.
    pub fn with_pending_request_buffer(mut self, pending_request_buffer: usize) -> Self {
        self.core_config.pending_request_buffer = pending_request_buffer;
        self
    }
    /// Set a callback function for judging whether to re-initiate the request.
    pub fn with_enable_retry(mut self, retry_fn: RF) -> Self {
        self.retry_fn = Some(retry_fn);
        self
    }
    /// Build a local and remote client.
    pub fn build(self) -> LRCall<S, D, LB, RF> {
        LRCall { config: self }
    }
}

/// load bnalance error
#[derive(Error, Debug)]
pub enum LoadBalanceError {
    /// retry error
    #[error("load balance retry reaches end")]
    Retry,
    /// discover error
    #[error("load balance discovery error: {0:?}")]
    Discover(#[from] BoxError),
}

/// A local and remote client.
pub struct LRCall<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    config: Builder<S, D, LB, RF>,
}

impl<S, D, LB, RF> LRCall<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    async fn warm_up(self) -> Result<Self, RpcError> {
        todo!()
    }
}

impl<S, D, LB, RF> Stub for LRCall<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    type Req = S::Req;

    type Resp = S::Resp;

    async fn call(&self, ctx: tarpc::context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        todo!()
    }
}
