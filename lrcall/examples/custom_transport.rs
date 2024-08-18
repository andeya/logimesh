// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use futures::prelude::*;
use lrcall::context::Context;
use lrcall::serde_transport as transport;
use lrcall::server::{BaseChannel, Channel};
use lrcall::tokio_serde::formats::Bincode;
use lrcall::tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio::net::{UnixListener, UnixStream};

#[lrcall::service]
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
    let bind_addr = "/tmp/lrcall_on_unix_example.sock";

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

            let fut = BaseChannel::with_defaults(transport).execute(Service.serve()).for_each(spawn);
            tokio::spawn(fut);
        }
    });

    let conn = UnixStream::connect(bind_addr).await?;
    let transport = transport::new(codec_builder.new_framed(conn), Bincode::default());
    PingServiceClient::rpc_client(PingServiceChannel::spawn(Default::default(), transport) )
        .ping(lrcall::context::rpc_current())
        .await?;

    Ok(())
}
