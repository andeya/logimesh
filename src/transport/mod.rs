// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A `Transport` which implements `AsyncRead` and `AsyncWrite`.

pub mod codec;
pub use ::tarpc::serde_transport::tcp;
pub use ::tarpc::transport::channel;
pub use ::tarpc::Transport;
