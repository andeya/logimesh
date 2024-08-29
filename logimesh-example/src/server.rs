// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::Parser;
use service::{init_tracing, CompHello, World};
use std::net::{IpAddr, Ipv6Addr};

#[derive(Parser)]
struct Flags {
    /// Sets the port number to listen on.
    #[clap(long)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();
    init_tracing("Tarpc Example Server")?;

    logimesh::tokio_tcp_listen!(CompHello, logimesh::server::TcpConfig::new((IpAddr::V6(Ipv6Addr::LOCALHOST), flags.port)));

    Ok(())
}
