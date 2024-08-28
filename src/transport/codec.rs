// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu config.

pub use ::tarpc::tokio_serde::formats::*;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::pin::Pin;
use std::sync::Arc;
use tarpc::tokio_serde::{Deserializer, Serializer};

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
    Bincode(Arc<RefCell<Bincode<Item, SinkItem>>>),
    /// JSON codec using [serde_json](https://docs.rs/serde_json) crate.
    Json(Arc<RefCell<Json<Item, SinkItem>>>),
    /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
    #[cfg(feature = "serde-transport-messagepack")]
    MessagePack(Arc<RefCell<MessagePack<Item, SinkItem>>>),
    /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
    #[cfg(feature = "serde-transport-cbor")]
    Cbor(Arc<RefCell<Cbor<Item, SinkItem>>>),
}

impl<Item, SinkItem> Clone for CodecFn<Item, SinkItem> {
    fn clone(&self) -> Self {
        match self {
            Self::Bincode(arg0) => Self::Bincode(arg0.clone()),
            Self::Json(arg0) => Self::Json(arg0.clone()),
            #[cfg(feature = "serde-transport-messagepack")]
            Self::MessagePack(arg0) => Self::MessagePack(arg0.clone()),
            #[cfg(feature = "serde-transport-cbor")]
            Self::Cbor(arg0) => Self::Cbor(arg0.clone()),
        }
    }
}

impl<Item, SinkItem> Default for CodecFn<Item, SinkItem> {
    fn default() -> Self {
        CodecFn::Bincode(Arc::new(RefCell::new(Bincode::default())))
    }
}

impl Codec {
    /// Returns the corresponding serde codec functions.
    pub fn to_fn<Item, SinkItem>(&self) -> CodecFn<Item, SinkItem> {
        match self {
            Self::Bincode => {
                // Bincode codec using [bincode](https://docs.rs/bincode) crate.
                CodecFn::Bincode(Arc::new(RefCell::new(Bincode::default())))
            },
            Self::Json => {
                // JSON codec using [serde_json](https://docs.rs/serde_json) crate.
                CodecFn::Json(Arc::new(RefCell::new(Json::default())))
            },
            #[cfg(feature = "serde-transport-messagepack")]
            Self::MessagePack => {
                /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
                CodecFn::MessagePack(Arc::new(RefCell::new(MessagePack::default())))
            },
            #[cfg(feature = "serde-transport-cbor")]
            Self::Cbor => {
                /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
                CodecFn::Cbor(Arc::new(RefCell::new(Cbor::default())))
            },
        }
    }
}

impl<Item, SinkItem> From<Codec> for CodecFn<Item, SinkItem> {
    fn from(value: Codec) -> Self {
        value.to_fn()
    }
}

impl<Item, SinkItem> Deserializer<Item> for CodecFn<Item, SinkItem>
where
    for<'a> Item: Unpin + Deserialize<'a>,
    SinkItem: Unpin,
{
    type Error = std::io::Error;

    fn deserialize(self: Pin<&mut Self>, src: &BytesMut) -> Result<Item, Self::Error> {
        match self.get_mut() {
            CodecFn::Bincode(c) => Ok(Pin::new(&mut *c.borrow_mut()).deserialize(src)?),
            CodecFn::Json(c) => Ok(Pin::new(&mut *c.borrow_mut()).deserialize(src)?),
            #[cfg(feature = "serde-transport-messagepack")]
            CodecFn::MessagePack(c) => Ok(Pin::new(&mut *c.borrow_mut()).deserialize(src)?),
            #[cfg(feature = "serde-transport-cbor")]
            CodecFn::Cbor(c) => Ok(Pin::new(&mut *c.borrow_mut()).deserialize(src)?),
        }
    }
}

impl<Item, SinkItem> Serializer<SinkItem> for CodecFn<Item, SinkItem>
where
    Item: Unpin,
    SinkItem: Serialize + Unpin,
{
    type Error = std::io::Error;

    fn serialize(self: Pin<&mut Self>, item: &SinkItem) -> Result<Bytes, Self::Error> {
        match self.get_mut() {
            CodecFn::Bincode(c) => Ok(Pin::new(&mut *c.borrow_mut()).serialize(item)?),
            CodecFn::Json(c) => Ok(Pin::new(&mut *c.borrow_mut()).serialize(item)?),
            #[cfg(feature = "serde-transport-messagepack")]
            CodecFn::MessagePack(c) => Ok(Pin::new(&mut *c.borrow_mut()).serialize(item)?),
            #[cfg(feature = "serde-transport-cbor")]
            CodecFn::Cbor(c) => Ok(Pin::new(&mut *c.borrow_mut()).serialize(item)?),
        }
    }
}

impl<Item, SinkItem> FnOnce<()> for CodecFn<Item, SinkItem> {
    type Output = Self;
    extern "rust-call" fn call_once(self, _: ()) -> Self::Output {
        self
    }
}

impl<Item, SinkItem> FnMut<()> for CodecFn<Item, SinkItem> {
    extern "rust-call" fn call_mut(&mut self, _: ()) -> Self::Output {
        self.clone()
    }
}

impl<Item, SinkItem> Fn<()> for CodecFn<Item, SinkItem> {
    extern "rust-call" fn call(&self, _: ()) -> Self::Output {
        self.clone()
    }
}
