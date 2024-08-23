// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Service discover.

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
pub struct ServiceInfo {
    /// service name
    pub name: String,
    /// service address, e.g. 172.0.10.10:8888
    pub addresses: Vec<String>,
    /// call type, such as local, remote.
    pub call_type: CallType,
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
