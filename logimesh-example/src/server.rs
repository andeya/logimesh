// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::Parser;
use futures::future;
use futures::prelude::*;
use logimesh::server::incoming::Incoming;
use logimesh::server::{self, Channel};
use service::{init_tracing, CompHello, World};
use std::net::{IpAddr, Ipv6Addr};

#[derive(Parser)]
struct Flags {
    /// Sets the port number to listen on.
    #[clap(long)]
    port: u16,
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();
    init_tracing("Tarpc Example Server")?;

    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), flags.port);

    // JSON transport is provided by the json_transport logimesh module. It makes it easy
    // to start up a serde-powered json serialization strategy over TCP.
    let mut listener = logimesh::transport::tcp::listen(&server_addr, CompHello::TRANSPORT_CODEC.to_fn()).await?;
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(2, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the component attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let server = CompHello;
            channel.execute(server.logimesh_serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}
