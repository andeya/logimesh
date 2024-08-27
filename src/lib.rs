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
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(impl_trait_in_assoc_type)]

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

#[allow(unreachable_pub, dead_code)]
mod sealed {
    pub trait Sealed<T> {}
}

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Convert from [`Result<T, BoxError>`] to [`Result<T, anyhow::Error>`]
pub trait IntoAnyResult<T> {
    /// Convert from [`Result<T, BoxError>`] to [`Result<T, anyhow::Error>`]
    fn any_result(self) -> Result<T, anyhow::Error>;
}

impl<T> IntoAnyResult<T> for Result<T, BoxError> {
    fn any_result(self) -> Result<T, anyhow::Error> {
        self.map_err(|e| anyhow::anyhow!("{e:?}"))
    }
}

#[test]
fn any_result() {
    let r: Result<bool, BoxError> = Err(BoxError::from("NaN".parse::<u32>().unwrap_err()));
    println!("{:?}", r.any_result());
}
