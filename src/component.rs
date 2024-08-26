// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! The component is a type that carries the service instance [`logimesh::server::Serve`] and endpoint information [`logimesh::component::Endpoint`].

use crate::net::Address;
use faststr::FastStr;
use metainfo::FastStrMap;
use std::fmt::Debug;

const DEFAULT_MAP_CAPACITY: usize = 10;

/// The component is a type that carries the service instance [`logimesh::server::Serve`] and endpoint information [`logimesh::component::Endpoint`].
pub struct Component<S> {
    /// The service instance [`logimesh::server::Serve`].
    pub serve: S,
    /// The endpoint information [`logimesh::component::Endpoint`]
    pub endpoint: Endpoint,
}

/// Endpoint contains the information of the service.
#[derive(Debug, Default)]
pub struct Endpoint {
    /// `service_name` is the most important information, which is used by the service discovering.
    pub service_name: FastStr,
    /// explicitly specified address
    pub address: Option<Address>,
    /// `tags` is used to store additional information of the endpoint.
    ///
    /// Users can use `tags` to store custom data, such as the datacenter name or the region name,
    /// which can be used by the service discoverer.
    pub tags: FastStrMap,
    /// A callback function used for creating keys.
    pub key_maker: Option<fn(&Self) -> FastStr>,
}

impl Endpoint {
    /// Creates a new endpoint info.
    #[inline]
    pub fn new(service_name: FastStr) -> Self {
        Self {
            service_name,
            address: None,
            tags: FastStrMap::with_capacity(DEFAULT_MAP_CAPACITY),
            key_maker: None,
        }
    }

    /// Create endpoint key.
    pub fn key(&self) -> FastStr {
        if let Some(f) = self.key_maker { f(self) } else { self.service_name.clone() }
    }

    /// Gets the service name of the endpoint.
    #[inline]
    pub fn service_name_ref(&self) -> &str {
        &self.service_name
    }

    /// Returns service name
    #[inline]
    pub fn service_name(&self) -> FastStr {
        self.service_name.clone()
    }

    /// Stes service name
    #[inline]
    pub fn set_service_name(&mut self, service_name: FastStr) {
        self.service_name = service_name;
    }

    /// Insert a tag into this `Endpoint`.
    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: FastStr) {
        self.tags.insert::<T>(val);
    }

    /// Check if `Endpoint` tags contain entry
    #[inline]
    pub fn contains<T: 'static>(&self) -> bool {
        self.tags.contains::<T>()
    }

    /// Get a reference to a tag previously inserted on this `Endpoint`.
    #[inline]
    pub fn get<T: 'static>(&self) -> Option<&FastStr> {
        self.tags.get::<T>()
    }

    /// Sets the address.
    #[inline]
    pub fn set_address(&mut self, address: Address) {
        self.address = Some(address)
    }

    /// Gets the address.
    #[inline]
    pub fn address(&self) -> Option<Address> {
        self.address.clone()
    }

    /// Clear the information
    #[inline]
    pub fn clear(&mut self) {
        self.service_name = FastStr::from_static_str("");
        self.address = None;
        self.tags.clear();
        self.key_maker = None;
    }
}
