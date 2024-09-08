// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Service Discovery Based on ETCD.

#![allow(dead_code)]
#![allow(unused_variables)]

use super::{Discover, Discovery, InstanceCluster};
use crate::client::ClientError;
use crate::component::Endpoint;
use async_broadcast::Receiver;
use etcd_client::Client;
pub use etcd_client::{ConnectOptions as EtcdConnectOptions, Error as EtcdError};
use faststr::FastStr;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ervice Discovery Based on ETCD.
pub struct EtcdDiscover {
    etcd_client: Client,
    instance_clusters: Arc<RwLock<HashMap<u16, InstanceCluster>>>,
}
impl EtcdDiscover {
    /// Creates a ETCD discover.
    pub async fn new<E: AsRef<str>, S: AsRef<[E]>>(endpoints: S, options: Option<EtcdConnectOptions>) -> Result<Self, EtcdError> {
        let etcd_client = Client::connect(endpoints, options).await?;
        Ok(Self {
            etcd_client,
            instance_clusters: Default::default(),
        })
    }
}
impl Discover for EtcdDiscover {
    fn discover<'s>(&'s self, endpoint: &'s Endpoint) -> impl Future<Output = Result<Discovery, ClientError>> + Send {
        async { todo!() }
    }

    fn watch(&self, keys: Option<&[FastStr]>) -> Option<Receiver<Discovery>> {
        todo!()
    }
}
