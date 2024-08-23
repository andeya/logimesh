// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::Parser;
use logimesh::client::stub::LRConfig;
use logimesh::client::stub::TransportCodec::Json;
use logimesh::context;
use logimesh::discover::{service_lookup, ServiceInfo};
use service::{init_tracing, CompHello, World as _};
use std::net::SocketAddr;
use std::sync::Arc;
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

    let client = CompHello
        .logimesh_client(
            LRConfig::new(
                "p.s.m".into(),
                service_lookup(|service_name| Ok(Arc::new(ServiceInfo::new(service_name.into(), vec![flags.server_addr.to_string()])))),
            )
            .with_transport_codec(Json),
        )
        .await?;

    let hello = async move {
        // Send the request twice, just to be safe! ;)
        tokio::select! {
            hello1 = client.hello(context::current(), format!("{}1", flags.name)) => { hello1 }
            hello2 = client.hello(context::current(), format!("{}2", flags.name)) => { hello2 }
        }
    }
    .instrument(tracing::info_span!("Two Hellos"))
    .await;

    match hello {
        Ok(hello) => tracing::info!("{hello:?}"),
        Err(e) => tracing::warn!("{:?}", anyhow::Error::from(e)),
    }

    // Let the background span processor finish.
    sleep(Duration::from_micros(10)).await;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
