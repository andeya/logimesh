// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Dummy discover.

use super::{Discover, Discovery, InstanceCluster};
use crate::component::Endpoint;
use crate::BoxError;
use async_broadcast::Receiver;
use faststr::FastStr;
use std::future::Future;

/// [`DummyDiscover`] always returns an empty list.
///
/// Users that don't specify the address directly need to use their own [`Discover`].
#[derive(Clone)]
pub struct DummyDiscover;

impl Discover for DummyDiscover {
    fn discover<'s>(&'s self, endpoint: &'s Endpoint) -> impl Future<Output = Result<Discovery, BoxError>> + Send {
        async move {
            Ok(Discovery {
                key: endpoint.key(),
                instance_cluster: InstanceCluster::Rpc(vec![]),
            })
        }
    }

    fn watch(&self, _: Option<&[FastStr]>) -> Option<Receiver<Discovery>> {
        None
    }
}
