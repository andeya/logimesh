// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu config.

pub use ::tarpc::tokio_serde::formats::*;

/// Transport serde codec
#[derive(Debug, Clone, Copy)]
pub enum Codec {
    /// Bincode codec using [bincode](https://docs.rs/bincode) crate.
    Bincode,
    /// JSON codec using [serde_json](https://docs.rs/serde_json) crate.
    Json,
    /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
    #[cfg(feature = "serde-transport-messagepack")]
    MessagePack,
    /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
    #[cfg(feature = "serde-transport-cbor")]
    Cbor,
}

impl Default for Codec {
    fn default() -> Self {
        Codec::Bincode
    }
}
