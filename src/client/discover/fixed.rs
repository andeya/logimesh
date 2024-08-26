// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Fixed instance list discover.

use crate::component::Endpoint;
use crate::net::address::Address;
use crate::BoxError;
use async_broadcast::Receiver;
use faststr::FastStr;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use super::{Change, Discover, Instance};

/// [`FixedDiscover`] is a simple implementation of [`Discover`] that returns a fixed list of instances.
#[derive(Clone)]
pub struct FixedDiscover {
    instances: Vec<Arc<Instance>>,
}

impl FixedDiscover {
    /// Creates a new [`FixedDiscover`].
    pub fn new(instances: Vec<Arc<Instance>>) -> Self {
        Self { instances }
    }
}

impl From<Vec<SocketAddr>> for FixedDiscover {
    fn from(addrs: Vec<SocketAddr>) -> Self {
        let instances = addrs
            .into_iter()
            .map(|addr| {
                Arc::new(Instance {
                    address: Address::Ip(addr),
                    weight: 1,
                    tags: Default::default(),
                })
            })
            .collect();
        Self { instances }
    }
}

impl Discover for FixedDiscover {
    fn discover<'s>(&'s self, _: &'s Endpoint) -> impl Future<Output = Result<Vec<Arc<Instance>>, BoxError>> + Send {
        async move { Ok(self.instances.clone()) }
    }

    fn watch(&self, _keys: Option<&[FastStr]>) -> Option<Receiver<Change<Instance>>> {
        None
    }
}
