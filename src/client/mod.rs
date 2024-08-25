// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//!
//! Provides a client that connects to a server and sends multiplexed requests.

pub mod balance;
pub mod discover;
pub mod stub;
pub use ::tarpc::client::{new, Channel, Config, NewClient, RequestDispatch, RpcError};
