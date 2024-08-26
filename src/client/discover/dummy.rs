// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Dummy discover.

use super::{Change, Discover, Instance};
use crate::component::Endpoint;
use crate::BoxError;
use async_broadcast::Receiver;
use faststr::FastStr;
use std::future::Future;
use std::sync::Arc;

/// [`DummyDiscover`] always returns an empty list.
///
/// Users that don't specify the address directly need to use their own [`Discover`].
#[derive(Clone)]
pub struct DummyDiscover;

impl Discover for DummyDiscover {
    fn discover<'s>(&'s self, _: &'s Endpoint) -> impl Future<Output = Result<Vec<Arc<Instance>>, BoxError>> + Send {
        async move { Ok(vec![]) }
    }

    fn watch(&self, _: Option<&[FastStr]>) -> Option<Receiver<Change<Instance>>> {
        None
    }
}
