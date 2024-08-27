// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A `Transport` which implements `AsyncRead` and `AsyncWrite`.

pub mod codec;
pub use ::tarpc::serde_transport::{new, unix};
pub use ::tarpc::transport::channel;
pub use ::tarpc::Transport;

pub mod tcp {
    //! tcp transport
    pub use ::tarpc::serde_transport::tcp::*;
    // use serde::Deserialize;
    // use std::io;
    // use tarpc::tokio_serde::{Deserializer, Serializer};
    // use tokio::net::ToSocketAddrs;

    // pub async fn tcp_listen<A, Item, SinkItem, Codec, CodecFn>(addr: A, codec: super::codec::Codec) -> io::Result<Incoming<Item, SinkItem, Codec, CodecFn>>
    // where
    //     A: ToSocketAddrs,
    //     Item: for<'de> Deserialize<'de>,
    //     Codec: Serializer<SinkItem> + Deserializer<Item>,
    //     CodecFn: Fn() -> Codec,
    // {
    //     match codec.codec_fn() {
    //         super::codec::CodecFn::Json(codec_fn) => listen(addr, codec_fn).await,
    //     }
    // }
}
