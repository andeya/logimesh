// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use futures::prelude::*;
use logimesh::context::Context;
use logimesh::server::{BaseChannel, Channel};
use logimesh::tokio_serde::formats::Bincode;
use logimesh::tokio_util::codec::length_delimited::LengthDelimitedCodec;
use logimesh::transport;
use tokio::net::{UnixListener, UnixStream};

#[logimesh::component]
pub trait PingService {
    async fn ping();
}

#[derive(Clone)]
struct Service;

impl PingService for Service {
    async fn ping(self, _: Context) {}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind_addr = "/tmp/logimesh_on_unix_example.sock";

    let _ = std::fs::remove_file(bind_addr);

    let listener = UnixListener::bind(bind_addr).unwrap();
    let codec_builder = LengthDelimitedCodec::builder();
    async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }
    tokio::spawn(async move {
        loop {
            let (conn, _addr) = listener.accept().await.unwrap();
            let framed = codec_builder.new_framed(conn);
            let transport = transport::new(framed, Bincode::default());

            let fut = BaseChannel::with_defaults(transport).execute(Service.logimesh_serve()).for_each(spawn);
            tokio::spawn(fut);
        }
    });

    let conn = UnixStream::connect(bind_addr).await?;
    let transport = transport::new(codec_builder.new_framed(conn), Bincode::default());
    PingServiceClient::new(Default::default(), transport).spawn().ping(logimesh::context::current()).await?;

    Ok(())
}
