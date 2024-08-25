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
pub mod endpoint;
pub mod net;
/// re-public `tarpc` crate something.
pub use crate::tarpc::*;

mod tarpc {
    #[doc(hidden)]
    pub use ::tarpc::serde;

    pub use ::tarpc::{tokio_serde, tokio_util};

    #[cfg_attr(docsrs, doc())]
    pub use ::tarpc::serde_transport;

    pub use ::tarpc::trace;

    pub use ::tarpc::{context, server, transport};

    pub use ::tarpc::Transport;

    pub use ::tarpc::{ChannelError, ClientMessage, Request, RequestName, Response, ServerError};
}

#[allow(unreachable_pub)]
mod sealed {
    pub trait Sealed<T> {}
}

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
