// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Endpoint is a service information for discovery

use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

/// A service information for discovery
pub trait Endpoint: Debug + Clone + Send + Sync {
    /// `Key` identifies the endpoint, such as the cluster name.
    type Key: Hash + PartialEq + Eq + Send + Sync + Clone + 'static;
    fn key(&self) -> Self::Key;
    fn service_name(&self) -> Cow<'_, str>;
}

impl Endpoint for () {
    type Key = ();

    fn key(&self) -> Self::Key {
        ()
    }

    fn service_name(&self) -> Cow<'_, str> {
        "<no_service_name>"
    }
}
