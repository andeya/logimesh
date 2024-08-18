// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::Parser;
use futures::future;
use futures::prelude::*;
use lrcall::context;
use lrcall::server::incoming::Incoming;
use lrcall::server::{self, Channel};
use lrcall::tokio_serde::formats::Json;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use service::{init_tracing, World};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use tokio::time;

#[derive(Parser)]
struct Flags {
    /// Sets the port number to listen on.
    #[clap(long)]
    port: u16,
}

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct HelloServer(SocketAddr);

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        let sleep_time = Duration::from_millis(Uniform::new_inclusive(1, 10).sample(&mut thread_rng()));
        time::sleep(sleep_time).await;
        format!("Hello, {name}! You are connected from {}", self.0)
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();
    init_tracing("Tarpc Example Server")?;

    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), flags.port);

    // JSON transport is provided by the json_transport lrcall module. It makes it easy
    // to start up a serde-powered json serialization strategy over TCP.
    let mut listener = lrcall::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let server = HelloServer(channel.transport().peer_addr().unwrap());
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}