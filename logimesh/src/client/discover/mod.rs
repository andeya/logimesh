// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Client Stub Information discovery

use super::ClientError;
use crate::component::Endpoint;
use crate::net::address::Address;
use async_broadcast::Receiver;
use core::marker::Send;
pub use dummy::DummyDiscover;
use faststr::FastStr;
pub use fixed::FixedDiscover;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

mod dummy;
pub mod etcd;
mod fixed;

/// [`Discover`] is the most basic trait for Discover.
pub trait Discover: Send + Sync + 'static {
    /// `discover` allows to request an endpoint and return a discover future.
    fn discover<'s>(&'s self, endpoint: &'s Endpoint) -> impl Future<Output = Result<Discovery, ClientError>> + Send;
    /// `watch` should return a [`async_broadcast::Receiver`] which can be used to subscribe [`Discovery`].
    fn watch(&self, keys: Option<&[FastStr]>) -> Option<Receiver<Discovery>>;
}

/// [`Instance`] contains information of an instance from the target service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    /// service address
    pub address: Address,
    /// service weight
    pub weight: u32,
    /// service tags extension
    pub tags: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

/// Discovery indicates the change of the service discover.
#[derive(Debug, Clone)]
pub struct Discovery {
    /// Endpoint key.
    pub key: FastStr,
    /// Local Serve or remote instance cluster.
    pub instance_cluster: InstanceCluster,
}

/// Local Serve or remote instance cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceCluster {
    /// Use local procedure call.
    Lpc,
    /// Use remote procedure call, and carry the instance list.
    Rpc(Vec<Arc<Instance>>),
}

#[cfg(test)]
mod tests {
    use super::{FixedDiscover, Instance};
    use crate::client::discover::{Discover, InstanceCluster};
    use crate::component::Endpoint;
    use crate::net::Address;
    use std::sync::Arc;

    #[test]
    fn test_fixed_discover() {
        let discover = FixedDiscover::from_address_str(vec!["127.0.0.1:8000", "127.0.0.2:9000"]).unwrap();
        let resp = futures::executor::block_on(async { discover.discover(&Endpoint::default()).await }).unwrap();
        let expected = InstanceCluster::Rpc(vec![
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
        ]);
        assert_eq!(resp.instance_cluster, expected);
    }
}
