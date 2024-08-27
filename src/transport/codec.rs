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

/// Transport serde codec function
#[derive(Debug)]
pub enum CodecFn<Item, SinkItem> {
    /// Bincode codec using [bincode](https://docs.rs/bincode) crate.
    Bincode(fn() -> Bincode<Item, SinkItem>),
    /// JSON codec using [serde_json](https://docs.rs/serde_json) crate.
    Json(fn() -> Json<Item, SinkItem>),
    /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
    #[cfg(feature = "serde-transport-messagepack")]
    MessagePack(MessagePack<Item, SinkItem>),
    /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
    #[cfg(feature = "serde-transport-cbor")]
    Cbor(Cbor<Item, SinkItem>),
}

impl<Item, SinkItem> Default for CodecFn<Item, SinkItem> {
    fn default() -> Self {
        CodecFn::Bincode(Bincode::default)
    }
}

impl Codec {
    /// Returns the corresponding serde codec functions.
    pub fn codec_fn<Item, SinkItem>(&self) -> CodecFn<Item, SinkItem> {
        match self {
            Self::Bincode => {
                // Bincode codec using [bincode](https://docs.rs/bincode) crate.
                CodecFn::Bincode(Bincode::default)
            },
            Self::Json => {
                // JSON codec using [serde_json](https://docs.rs/serde_json) crate.
                CodecFn::Json(Json::default)
            },
            #[cfg(feature = "serde-transport-messagepack")]
            Self::MessagePack => {
                /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
                CodecFn::MessagePack(MessagePack::default)
            },
            #[cfg(feature = "serde-transport-cbor")]
            Self::Cbor => {
                /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
                CodecFn::Cbor(Cbor::default)
            },
        }
    }
}

// impl<Item, SinkItem> From<Codec> for CodecFn<Item, SinkItem> {
//     fn from(value: Codec) -> Self {
//         value.codec_fn()
//     }
// }
// use serde::Deserialize;
// use tarpc::tokio_serde::{Deserializer, Serializer};
// impl<Item, SinkItem> FnOnce<()> for CodecFn<Item, SinkItem>
// where
//     Item: for<'de> Deserialize<'de>,
// {
//     type Output = impl Serializer<SinkItem> + Deserializer<Item>;

//     extern "rust-call" fn call_once(self, args: ()) -> Self::Output {
//         todo!()
//     }
// }
