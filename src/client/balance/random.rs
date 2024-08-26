// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Random load balance implemention

use super::LoadBalance;
use std::hash::Hash;
use tarpc::server::Serve;

/// Random load balance implemention
pub struct RandomBalance;

impl<Key, S> LoadBalance<Key, S> for RandomBalance
where
    Key: Hash + PartialEq + Eq + Send + Sync + Clone + 'static,
    S: Serve,
{
    fn start_balance(&self, instances: Vec<crate::client::channel::RpcChannel<S>>) {
        todo!()
    }

    fn next(&self) -> Option<crate::client::channel::RpcChannel<S>> {
        todo!()
    }

    fn rebalance(&self, changes: crate::client::discover::Change<Key, crate::client::channel::RpcChannel<S>>) {
        todo!()
    }
}
