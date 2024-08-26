// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! The component is a type that carries the service instance [`logimesh::server::Serve`] and endpoint information [`logimesh::component::Endpoint`].

use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

/// The component is a type that carries the service instance [`logimesh::server::Serve`] and endpoint information [`logimesh::component::Endpoint`].
pub struct Component<S, E> {
    /// The service instance [`logimesh::server::Serve`].
    pub serve: S,
    /// The endpoint information [`logimesh::component::Endpoint`]
    pub endpoint: E,
}

/// A service information for discovery
pub trait Endpoint: Debug + Clone + Send + Sync {
    /// `Key` identifies the endpoint, such as the cluster name.
    type Key: Hash + PartialEq + Eq + Send + Sync + Clone + 'static;
    /// Returns endpoint key
    fn key(&self) -> Self::Key;
    /// Returns service name, recommended format is `Product.System.Module`
    fn service_name(&self) -> Cow<'_, str>;
}

impl Endpoint for () {
    type Key = ();

    fn key(&self) -> Self::Key {
        ()
    }

    fn service_name(&self) -> Cow<'_, str> {
        "<no_service_name>".into()
    }
}
