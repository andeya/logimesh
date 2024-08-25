// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Provides a Stub trait, implemented by types that can call remote services.
pub(crate) mod channel;
pub use ::tarpc::client::stub::Stub;
// pub use config::*;
// pub use lrcall::*;
// mod config;
// mod lrcall;
/// The methods that all components should implement
pub trait Component {}

/// Transport serde codec
#[derive(Debug, Clone)]
pub enum TransportCodec {
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

impl Default for TransportCodec {
    fn default() -> Self {
        TransportCodec::Bincode
    }
}
