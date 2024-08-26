// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! load balance for channel.

pub mod random;
use crate::client::channel::RpcChannel;
use crate::client::discover::RpcChange;
use crate::server::Serve;

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
    fn rebalance(&self, changes: Option<RpcChange<RpcChannel<S>>>);
}
