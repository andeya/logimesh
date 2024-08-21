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

pub use logimesh_macro::{derive_serde, service};

pub mod client;
pub mod discover;
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
