// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Client Stub Information discovery
use crate::net::address::Address;
use crate::{endpoint, BoxError};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::Arc;

use async_broadcast::Receiver;

/// [`Instance`] contains information of an instance from the target service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    pub address: Address,
    pub weight: u32,
    pub tags: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

/// [`Discover`] is the most basic trait for Discover.
pub trait Discover: Send + Sync + 'static {
    /// A service information for discovery
    type Endpoint: endpoint::Endpoint;
    /// `Key` identifies the endpoint, such as the cluster name.
    /// NOTE: Be sure to use this default type and do not set other types.  
    type Key = <Self::Endpoint as endpoint::Endpoint>::Key;
    /// `discover` allows to request an endpoint and return a discover future.
    async fn discover<'s>(&'s self, endpoint: &'s Self::Endpoint) -> Result<Vec<Arc<Instance>>, BoxError>;
    /// `watch` should return a [`async_broadcast::Receiver`] which can be used to subscribe
    /// [`Change`].
    fn watch(&self, keys: Option<&[Self::Key]>) -> Option<Receiver<Change<Self::Key>>>;
}

/// Change indicates the change of the service discover.
#[derive(Debug, Clone)]
pub struct Change<K> {
    /// `key` should be the same as the output of `Endpoint::Key`,
    /// which is often used by cache.
    pub key: K,
    /// Use local or remote, and specific change information
    pub change: LRChange,
}

/// Use local or remote, and specific change information
#[derive(Debug, Clone)]
pub enum LRChange {
    /// Use local procedure call.
    Lpc,
    /// Use remote procedure call, and carry the change details.
    Rpc(RpcChange),
}

/// Change indicates the change of the service discover.
///
/// Change contains the difference between the current discovery result and the previous one.
/// It is designed for providing detail information when dispatching an event for service
/// discovery result change.
///
/// Since the loadbalancer may rely on caching the result of discover to improve performance,
/// the discover implementation should dispatch an event when result changes.
#[derive(Debug, Clone)]
pub struct RpcChange {
    pub all: Vec<Arc<Instance>>,
    pub added: Vec<Arc<Instance>>,
    pub updated: Vec<Arc<Instance>>,
    pub removed: Vec<Arc<Instance>>,
}

/// [`diff_address`] provides a naive implementation that compares prev and next only by the
/// address, and returns the [`RpcChange`], which means that the `updated` is always empty when using
/// this implementation.
///
/// The bool in the return value indicates whether there's diff between prev and next, which means
/// that if the bool is false, the [`RpcChange`] should be ignored, and the discover should not send
/// the event to loadbalancer.
///
/// If users need to compare the instances by also weight or tags, they should not use this.
pub fn diff_address<K>(prev: Vec<Arc<Instance>>, next: Vec<Arc<Instance>>) -> (RpcChange, bool)
where
    K: Hash + PartialEq + Eq + Send + Sync + 'static,
{
    let mut added = Vec::new();
    let updated = Vec::new();
    let mut removed = Vec::new();

    let mut prev_set = HashSet::with_capacity(prev.len());
    let mut next_set = HashSet::with_capacity(next.len());
    for i in &prev {
        prev_set.insert(i.address.clone());
    }
    for i in &next {
        next_set.insert(i.address.clone());
    }

    for i in &next {
        if !prev_set.contains(&i.address) {
            added.push(i.clone());
        }
    }
    for i in &prev {
        if !next_set.contains(&i.address) {
            removed.push(i.clone());
        }
    }

    let changed = !added.is_empty() || !removed.is_empty();

    (RpcChange { all: next, added, updated, removed }, changed)
}

/// [`StaticDiscover`] is a simple implementation of [`Discover`] that returns a static list of
/// instances.
#[derive(Clone)]
pub struct StaticDiscover {
    instances: Vec<Arc<Instance>>,
}

impl StaticDiscover {
    /// Creates a new [`StaticDiscover`].
    pub fn new(instances: Vec<Arc<Instance>>) -> Self {
        Self { instances }
    }
}

impl From<Vec<SocketAddr>> for StaticDiscover {
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

impl Discover for StaticDiscover {
    type Endpoint = ();

    async fn discover<'s>(&'s self, _: &'s Self::Endpoint) -> Result<Vec<Arc<Instance>>, BoxError> {
        Ok(self.instances.clone())
    }

    fn watch(&self, _keys: Option<&[Self::Key]>) -> Option<Receiver<Change<Self::Key>>> {
        None
    }
}

/// [`DummyDiscover`] always returns an empty list.
///
/// Users that don't specify the address directly need to use their own [`Discover`].
#[derive(Clone)]
pub struct DummyDiscover;

impl Discover for DummyDiscover {
    type Endpoint = ();
    async fn discover<'s>(&'s self, _: &'s Self::Endpoint) -> Result<Vec<Arc<Instance>>, BoxError> {
        Ok(vec![])
    }

    fn watch(&self, _: Option<&[Self::Key]>) -> Option<Receiver<Change<Self::Key>>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{Discover, Instance, StaticDiscover};
    use crate::net::Address;

    #[test]
    fn test_static_discover() {
        let discover = StaticDiscover::from(vec!["127.0.0.1:8000".parse().unwrap(), "127.0.0.2:9000".parse().unwrap()]);
        let resp = futures::executor::block_on(async { discover.discover(&()).await }).unwrap();
        let expected = vec![
            Arc::new(Instance {
                address: Address::Ip("127.0.0.1:8000".parse().unwrap()),
                weight: 1,
                tags: Default::default(),
            }),
            Arc::new(Instance {
                address: Address::Ip("127.0.0.2:9000".parse().unwrap()),
                weight: 1,
                tags: Default::default(),
            }),
        ];
        assert_eq!(resp, expected);
    }
}
