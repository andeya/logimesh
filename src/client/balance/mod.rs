// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! load balance for channel.

pub use random::*;
mod random;
use crate::client::channel::RpcChannel;
use crate::net::Address;
use crate::server::Serve;
use std::fmt::Debug;

/// [`LoadBalance`] promise the feature of the load balance policy.
pub trait LoadBalance<S>: Send + Sync + 'static
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
{
    /// `InstanceIter` is an iterator of [`crate::client::channel::RpcChannel`].
    type ChannelIter: Iterator<Item = RpcChannel<S>> + Send;
    /// Start a load balancing task.
    fn start_balance(&self, channels: Vec<RpcChannel<S>>);
    /// `get_picker` allows to get an RPC channel iterator.
    fn get_picker(&self) -> Self::ChannelIter;
    /// `rebalance` is the callback method be used in balance stub.
    /// If changes is `Option::None`, it indicates that the channels should be cleared.
    fn rebalance(&self, changes: Option<RpcChange<S>>);
}

/// Change indicates the change of the service discover.
///
/// Change contains the difference between the current discovery result and the previous one.
/// It is designed for providing detail information when dispatching an event for service
/// discovery result change.
///
/// Since the loadbalancer may rely on caching the result of discover to improve performance,
/// the discover implementation should dispatch an event when result changes.
#[derive(Clone)]
pub struct RpcChange<S: Serve> {
    /// All service instances list
    pub all: Vec<RpcChannel<S>>,
    /// The list of newly added services
    pub added: Vec<RpcChannel<S>>,
    /// The list of newly updated services
    pub updated: Vec<RpcChannel<S>>,
    /// The keys of newly removed services
    pub removed: Vec<Address>,
}

impl<S> Debug for RpcChange<S>
where
    S: Serve,
    S::Req: Debug,
    S::Resp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcChange")
            .field("all", &self.all)
            .field("added", &self.added)
            .field("updated", &self.updated)
            .field("removed", &self.removed)
            .finish()
    }
}
