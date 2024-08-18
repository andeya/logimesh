// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::Parser;
use lrcall::tokio_serde::formats::Json;
use lrcall::{client, context};
use service::{init_tracing, WorldClient};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::Instrument;

#[derive(Parser)]
struct Flags {
    /// Sets the server address to connect to.
    #[clap(long)]
    server_addr: SocketAddr,
    /// Sets the name to say hello to.
    #[clap(long)]
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();
    init_tracing("Tarpc Example Client")?;

    let mut transport = lrcall::serde_transport::tcp::connect(flags.server_addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);

    // WorldClient is generated by the service attribute. It has a constructor `new` that takes a
    // config and any Transport as input.
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    let hello = async move {
        // Send the request twice, just to be safe! ;)
        tokio::select! {
            hello1 = client.hello(context::rpc_current(), format!("{}1", flags.name)) => { hello1 }
            hello2 = client.hello(context::rpc_current(), format!("{}2", flags.name)) => { hello2 }
        }
    }
    .instrument(tracing::info_span!("Two Hellos"))
    .await;

    match hello {
        Ok(hello) => tracing::info!("{hello:?}"),
        Err(e) => tracing::warn!("{:?}", anyhow::Error::from(e)),
    }

    // Let the background span processor finish.
    sleep(Duration::from_micros(1)).await;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
