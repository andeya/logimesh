// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Local and remote Cclient.

use crate::client::balance::LoadBalance;
use crate::client::channel::{Codec, RpcChannel, RpcConfig};
use crate::client::discover::{Change, Discover, Instance, LRChange, RpcChange};
use crate::client::stub::Stub;
use crate::client::{CoreConfig, RpcError};
use crate::component::Component;
use crate::server::Serve;
use crate::BoxError;
use std::ptr;
use std::sync::atomic::AtomicPtr;
use std::sync::Arc;
use thiserror::Error;
use tracing::warn;
/// A full client stbu config.
#[non_exhaustive]
pub struct Builder<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    /// The implementer of the local service.
    pub(crate) component: Component<S>,
    /// discover instance.
    pub(crate) discover: Arc<D>,
    /// load balance instance.
    pub(crate) load_balance: Arc<LB>,
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
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    /// Create a LRCall builder.
    pub fn new(component: Component<S>, discover: D, load_balance: LB) -> Self {
        Self {
            component,
            discover: Arc::new(discover),
            load_balance: Arc::new(load_balance),
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
    pub async fn try_build(self) -> Result<LRCall<S, D, LB, RF>, BoxError> {
        LRCall {
            config: self,
            channels: AtomicPtr::new(ptr::null_mut()),
        }
        .warm_up()
        .await
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
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    config: Builder<S, D, LB, RF>,
    channels: AtomicPtr<Vec<RpcChannel<S>>>,
}

impl<S, D, LB, RF> LRCall<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    async fn warm_up(self) -> Result<Self, BoxError> {
        let instances = self.config.discover.discover(&self.config.component.endpoint).await?;
        let mut channels: Vec<RpcChannel<S>> = Vec::new();
        for instance in instances {
            let channel = RpcChannel::new(RpcConfig {
                instance,
                transport_codec: self.config.transport_codec,
                core_config: self.config.core_config.clone(),
            })
            .await;
            match channel {
                Ok(channel) => {
                    channels.push(channel);
                },
                Err(e) => {
                    // TODO
                    println!("{e:?}");
                },
            }
        }

        self.config.load_balance.start_balance(channels);
        let load_balance = self.config.load_balance.clone();
        let transport_codec = self.config.transport_codec;
        let core_config = self.config.core_config.clone();
        if let Some(mut recv_change) = self.config.discover.watch(None) {
            tokio::spawn(async move {
                loop {
                    match recv_change.recv().await {
                        Ok(change) => match Self::convert_and_dial(transport_codec, &core_config, change).await {
                            Ok(change) => load_balance.rebalance(change),
                            Err(err) => warn!("[LOGIMESH] TCP connection establishment failed: {:?}", err),
                        },
                        Err(err) => warn!("[LOGIMESH] discovering subscription error: {:?}", err),
                    }
                }
            });
        }
        Ok(self)
    }

    async fn convert_and_dial(transport_codec: Codec, core_config: &CoreConfig, change: Change<Arc<Instance>>) -> Result<Option<RpcChange<RpcChannel<S>>>, BoxError> {
        match change.change {
            LRChange::Lpc => Ok(None),
            LRChange::Rpc(lr) => {
                todo!()
            },
        }
    }
}

impl<S, D, LB, RF> Stub for LRCall<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
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
