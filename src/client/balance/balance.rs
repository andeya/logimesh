// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Service load balance
use tarpc::server::Serve;

use super::error::LoadBalanceError;
use crate::client::discover::{Change, Discover};
use crate::client::stub::channel::ChannelInstance;

/// [`LoadBalance`] promise the feature of the load balance policy.
pub trait LoadBalance<D, S>: Send + Sync + 'static
where
    D: Discover,
    S: Serve,
{
    /// `InstanceIter` is an iterator of [`crate::discovery::Instance`].
    type ChannelIter: Iterator<Item = ChannelInstance<S>> + Send;

    /// `get_picker` allows to get a channel iterator of a specified endpoint from self or
    /// service discovery.
    async fn get_picker<'future>(&'future self, endpoint: &'future D::Endpoint, discover: &'future D) -> Result<Self::ChannelIter, LoadBalanceError>;
    /// `rebalance` is the callback method be used in service discovering subscription.
    fn rebalance(&self, changes: Change<D::Key>);
}
