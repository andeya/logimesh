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

use faststr::FastStr;

/// Critical errors that result in a Channel disconnecting.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ClientError {
    /// Could not read from the transport.
    #[error("Service discovery failed: {0}")]
    Discover(FastStr),
    /// Could not ready the transport for writes.
    #[error("New service load balance failed: {0}")]
    NewBalance(FastStr),
    /// Could not ready the transport for writes.
    #[error("New LRCall failed: {0}")]
    NewLRCall(FastStr),
}
