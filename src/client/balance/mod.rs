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
use std::hash::Hash;
use tarpc::server::Serve;

/// [`LoadBalance`] promise the feature of the load balance policy.
pub trait LoadBalance<Key, S>: Send + Sync + 'static
where
    Key: Hash + PartialEq + Eq + Send + Sync + Clone + 'static,
    S: Serve,
{
    /// Start a load balancing task.
    fn start_balance(&self, instances: Vec<RpcChannel<S>>);
    /// `next` Return the next channel in the load balancing round.
    fn next(&self) -> Option<RpcChannel<S>>;
    /// `rebalance` is the callback method be used in balance stub.
    fn rebalance(&self, changes: Change<Key, RpcChannel<S>>);
}
