// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! load balance for channel.

pub mod random;
use crate::client::channel::RpcChannel;
use crate::client::discover::Change;
use tarpc::server::Serve;

/// [`LoadBalance`] promise the feature of the load balance policy.
pub trait LoadBalance<S>: Send + Sync + 'static
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
{
    /// Start a load balancing task.
    fn start_balance(&self, instances: Vec<RpcChannel<S>>);
    /// `next` Return the next channel in the load balancing round.
    fn next(&self) -> Option<RpcChannel<S>>;
    /// `rebalance` is the callback method be used in balance stub.
    fn rebalance(&self, changes: Change<RpcChannel<S>>);
}
