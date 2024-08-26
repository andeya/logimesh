// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! logimesh is a Rust microservice 2.0 framework.
#![deny(missing_docs)]
#![allow(clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(associated_type_defaults)]

pub use logimesh_macro::{component, derive_serde};
pub mod client;
pub mod component;
pub mod context;
pub mod net;
pub mod server;
pub mod transport;
pub use ::tarpc::{serde, tokio_serde, tokio_util, ChannelError, ClientMessage, Request, RequestName, Response, ServerError};
pub use transport::Transport;
pub mod trace;

#[allow(unreachable_pub)]
mod sealed {
    pub trait Sealed<T> {}
}

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
