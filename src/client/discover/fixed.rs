// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Fixed instance list discover.

use super::{Discover, Discovery, Instance, InstanceCluster};
use crate::component::Endpoint;
use crate::net::address::Address;
use crate::BoxError;
use async_broadcast::Receiver;
use faststr::FastStr;
use std::future::Future;
use std::net::AddrParseError;
use std::sync::Arc;

/// [`FixedDiscover`] is a simple implementation of [`Discover`] that returns a fixed list of instances.
#[derive(Clone)]
pub struct FixedDiscover {
    instance_cluster: InstanceCluster,
}

impl FixedDiscover {
    /// Creates a new [`FixedDiscover`].
    pub fn new(instance_cluster: InstanceCluster) -> Self {
        Self { instance_cluster }
    }
    /// Creates a new [`FixedDiscover`] from address.
    pub fn from_address(address_list: Vec<Address>) -> Self {
        let instances = address_list
            .into_iter()
            .map(|address| {
                Arc::new(Instance {
                    address: address,
                    weight: 1,
                    tags: Default::default(),
                })
            })
            .collect();
        Self {
            instance_cluster: InstanceCluster::Rpc(instances),
        }
    }
    /// Creates a new [`FixedDiscover`] from address.
    pub fn from_address_str(address_list: Vec<impl AsRef<str>>) -> Result<Self, AddrParseError> {
        let mut list = Vec::new();
        for ele in address_list {
            list.push(ele.as_ref().parse()?);
        }
        Ok(Self::from_address(list))
    }
}

impl Discover for FixedDiscover {
    fn discover<'s>(&'s self, endpoint: &'s Endpoint) -> impl Future<Output = Result<Discovery, BoxError>> + Send {
        async move {
            Ok(Discovery {
                key: endpoint.key(),
                instance_cluster: self.instance_cluster.clone(),
            })
        }
    }

    fn watch(&self, _keys: Option<&[FastStr]>) -> Option<Receiver<Discovery>> {
        None
    }
}
