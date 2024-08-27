// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Local and remote Cclient.

use crate::client::balance::{LoadBalance, RpcChange};
use crate::client::channel::{Codec, RpcChannel, RpcConfig};
use crate::client::discover::{Discover, Discovery, Instance, InstanceCluster};
use crate::client::stub::Stub;
use crate::client::{CoreConfig, RpcError};
use crate::component::Component;
use crate::net::Address;
use crate::server::Serve;
use crate::BoxError;
use futures_util::{select, FutureExt};
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Notify;
use tracing::{trace, warn};

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
            notify: Arc::new(Notify::new()),
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
    notify: Arc<Notify>,
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
        let discovery = self.config.discover.discover(&self.config.component.endpoint).await?;
        let mut channels: Vec<RpcChannel<S>> = Vec::new();
        match discovery.instance_cluster {
            InstanceCluster::Lpc => {},
            InstanceCluster::Rpc(instances) => {
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
            },
        }
        let prev = channels.clone();
        self.config.load_balance.start_balance(channels);
        if let Some(mut recv_change) = self.config.discover.watch(None) {
            let load_balance = self.config.load_balance.clone();
            let transport_codec = self.config.transport_codec;
            let core_config = self.config.core_config.clone();
            let notify = self.notify.clone();
            tokio::spawn(async move {
                let mut prev = prev;
                loop {
                    select! {
                        _ = notify.notified().fuse() => {
                            return;
                        },
                        discovery = recv_change.recv().fuse() => match discovery {
                            Ok(Discovery{instance_cluster:InstanceCluster::Lpc,..}) => {
                                prev.clear();
                            },
                            Ok(Discovery{instance_cluster:InstanceCluster::Rpc(next),..}) => match Self::diff_and_dial(transport_codec, &core_config, &mut prev, next).await {
                                Ok(changes) => load_balance.rebalance(changes),
                                Err(err) => warn!("[LOGIMESH] TCP connection establishment failed: {:?}", err),
                            },
                            Err(err) => warn!("[LOGIMESH] discovering subscription error: {:?}", err),
                        },
                    }
                }
            });
        }
        Ok(self)
    }

    async fn diff_and_dial(transport_codec: Codec, core_config: &CoreConfig, prev: &mut Vec<RpcChannel<S>>, next: Vec<Arc<Instance>>) -> Result<Option<RpcChange<S>>, BoxError>
    where
        S: Serve,
    {
        let mut added: Vec<RpcChannel<S>> = Vec::new();
        let mut updated: Vec<RpcChannel<S>> = Vec::new();
        let mut removed: Vec<Address> = Vec::new();

        let mut next_set = HashSet::with_capacity(next.len());

        for i in &next {
            next_set.insert(i.address.clone());
        }

        for instance in &next {
            let mut is_new: bool = true;
            for c in prev.iter_mut() {
                if &instance.address == &c.config().instance.address {
                    is_new = false;
                    if &c.config().instance != instance {
                        let updated_channel = c.clone_update_instance(instance.clone());
                        *c = updated_channel.clone();
                        updated.push(updated_channel);
                    }
                    break;
                }
            }
            if is_new {
                let channel = RpcChannel::new(RpcConfig {
                    instance: instance.clone(),
                    transport_codec: transport_codec,
                    core_config: core_config.clone(),
                })
                .await;
                match channel {
                    Ok(channel) => {
                        added.push(channel);
                    },
                    Err(e) => {
                        warn!("[LOGIMESH] failed to connect: {e:?}");
                    },
                }
            }
        }

        for i in prev.iter() {
            if !next_set.contains(&i.config().instance.address) {
                removed.push(i.config().instance.address.clone());
            }
        }

        if removed.len() > 0 {
            prev.retain_mut(|c| {
                for address in &removed {
                    if &c.config().instance.address == address {
                        return false;
                    }
                }
                return true;
            });
        }

        prev.append(&mut added);

        let changed = !added.is_empty() || !removed.is_empty() || !updated.is_empty();
        if changed {
            Ok(Some(RpcChange {
                all: prev.clone(),
                added,
                updated,
                removed,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<S, D, LB, RF> Stub for LRCall<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: crate::serde::Serialize + Send + Clone + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    type Req = S::Req;

    type Resp = S::Resp;

    async fn call(&self, ctx: crate::context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        let mut picker = self.config.load_balance.get_picker();
        for i in 1.. {
            if let Some(channel) = picker.next() {
                let result = channel.call(ctx, request.clone()).await;
                if let Err(RpcError::Shutdown) = result {
                    // TODO: Change to asynchronous processing
                    match channel.reconnent().await {
                        Ok(_) => trace!("[LOGIMESH] success to reconnect"),
                        Err(e) => warn!("[LOGIMESH] failed to reconnect: {e:?}"),
                    };
                }
                if let Some(retry_fn) = &self.config.retry_fn {
                    if (retry_fn)(&result, i) {
                        tracing::trace!("Retrying on attempt {i}");
                        continue;
                    }
                }
                return result;
            }
        }
        unreachable!("Wow, that was a lot of attempts!");
    }
}

impl<S, D, LB, RF> Drop for LRCall<S, D, LB, RF>
where
    S: Serve + 'static,
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
    D: Discover,
    LB: LoadBalance<S>,
    RF: Fn(&Result<S::Resp, RpcError>, u32) -> bool,
{
    fn drop(&mut self) {
        self.notify.notify_waiters();
    }
}
