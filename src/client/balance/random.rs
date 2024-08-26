// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Random load balance implemention

use super::LoadBalance;
use crate::client::channel::RpcChannel;
use crate::client::discover::Change;
use crate::server::Serve;

/// Random load balance implemention
pub struct RandomBalance;

impl<S> LoadBalance<S> for RandomBalance
where
    S: Serve + 'static,
    S::Req: Send,
    S::Resp: Send,
{
    fn start_balance(&self, instances: Vec<RpcChannel<S>>) {
        todo!()
    }

    fn next(&self) -> Option<RpcChannel<S>> {
        todo!()
    }

    fn rebalance(&self, changes: Change<RpcChannel<S>>) {
        todo!()
    }
}
