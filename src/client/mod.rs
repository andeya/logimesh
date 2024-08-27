// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Provides a client that connects to a server and sends multiplexed requests.

pub mod balance;
pub mod channel;
pub mod discover;
pub mod lrcall;
pub use core::stub::Stub;
pub use core::RpcError;
/// re-public tarpc some types.
pub mod core {
    // pub use ::tarpc::client::{new, Channel, Config, NewClient, RequestDispatch, RpcError};
    pub use ::tarpc::client::*;
}
