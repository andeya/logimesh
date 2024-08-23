// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Service discover.

use std::marker::PhantomData;
use std::sync::Arc;

/// Service discover.
pub trait ServiceDiscovery: ServiceRegister + ServiceLookup {}
impl<T> ServiceDiscovery for T where T: ServiceRegister + ServiceLookup {}

/// Service register.
pub trait ServiceRegister {
    /// register service
    fn register_service(&self, service_info: ServiceInfo) -> anyhow::Result<()>;
}

/// Service lookup.
pub trait ServiceLookup {
    /// Returns service information
    fn lookup_service(&self, service_name: &str) -> anyhow::Result<Arc<ServiceInfo>>;
}

/// Service information.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct ServiceInfo {
    /// service name
    pub name: String,
    /// service address, e.g. 172.0.10.10:8888
    pub addresses: Vec<String>,
    /// call type, such as local, remote.
    pub call_type: CallType,
}

impl ServiceInfo {
    /// Create a service information.
    pub fn new(name: String, addresses: Vec<String>) -> Self {
        Self {
            name,
            addresses,
            call_type: CallType::Remote,
        }
    }
    /// Set call type.
    pub fn with_call_type(mut self, call_type: CallType) -> Self {
        self.call_type = call_type;
        self
    }
}
/// Call Type.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum CallType {
    /// Local call type.
    Local = 0,
    /// Remote call type.
    Remote = 1,
}

/// A ServiceLookup wrapper around a Fn.
#[derive(Debug)]
pub struct ServiceLookupFn<F> {
    f: F,
    data: PhantomData<fn(&str) -> anyhow::Result<Arc<ServiceInfo>>>,
}

impl<F> Clone for ServiceLookupFn<F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self { f: self.f.clone(), data: PhantomData }
    }
}

impl<F> Copy for ServiceLookupFn<F> where F: Copy {}

/// Creates a [`ServiceLookup`] wrapper around a `Fn(&str) -> anyhow::Result<Arc<ServiceInfo>>`.
pub fn service_lookup_from_fn<F>(f: F) -> ServiceLookupFn<F>
where
    F: Fn(&str) -> anyhow::Result<Arc<ServiceInfo>>,
{
    ServiceLookupFn { f, data: PhantomData }
}

/// Create a [`ServiceLookup`] from a set of fixed addresses.
pub fn service_lookup_from_addresses(addresses: Vec<String>) -> ServiceLookupFn<impl Fn(&str) -> anyhow::Result<Arc<ServiceInfo>>> {
    let addresses = Arc::new(addresses);
    ServiceLookupFn {
        f: move |service_name: &str| Ok(Arc::new(ServiceInfo::new(service_name.into(), Vec::from_iter(addresses.iter().cloned())))),
        data: PhantomData,
    }
}

impl<F> ServiceLookup for ServiceLookupFn<F>
where
    F: Fn(&str) -> anyhow::Result<Arc<ServiceInfo>>,
{
    fn lookup_service(&self, service_name: &str) -> anyhow::Result<Arc<ServiceInfo>> {
        (self.f)(service_name)
    }
}
