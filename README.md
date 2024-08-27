# logimesh

`logimesh` is a Rust microcomponent 2.0 framework inspired by the [_Towards Modern Development of Cloud Applications_](https://dl.acm.org/doi/pdf/10.1145/3593856.3595909) paper.

_（This is one of my amateur idea and is only developed in leisure-time.）_

[![Crates.io](https://img.shields.io/crates/v/logimesh)](https://crates.io/crates/logimesh)
[![Documentation](https://shields.io/docsrs/logimesh)](https://docs.rs/logimesh)
[![License](https://img.shields.io/crates/l/logimesh)](https://github.com/andeya/logimesh?tab=MIT-1-ov-file)

![component](https://raw.githubusercontent.com/andeya/logimesh/main/docs/component.png)

## Some features of logimesh:

-   The client supports both local calls and remote calls simultaneously, meaning that users can dynamically switch the calling method according to the context.

## Usage

Add to your `Cargo.toml` dependencies:

```toml
logimesh = "0.1"
```

The `logimesh::component` attribute expands to a collection of items that form an component component.
These generated types make it easy and ergonomic to write servers with less boilerplate.
Simply implement the generated component trait, and you're off to the races!

## Example

This example uses [tokio](https://tokio.rs), so add the following dependencies to
your `Cargo.toml`:

```toml
[lib]
name = "service"
path = "src/lib.rs"

...

[dependencies]
anyhow = "1.0"
futures = "0.3"
logimesh = { version = "0.1" }
tokio = { version = "1.0", features = ["macros"] }
```


For a more real-world example, see [logimesh-example](logimesh-example).

First, let's set up the dependencies and component definition.

### `lib.rs` file

```rust
# extern crate futures;

use futures::{
    prelude::*,
};
use logimesh::{
    client, context,
    server::{self, incoming::Incoming, Channel},
};

// This is the component definition. It looks a lot like a trait definition.
// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[logimesh::component]
trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}
```

This component definition generates a trait called `World`. Next we need to
implement it for our Server struct.

```rust
# extern crate futures;
# use futures::{
#     prelude::*,
# };
# use logimesh::{
#     client, context,
#     server::{self, incoming::Incoming},
# };
# // This is the component definition. It looks a lot like a trait definition.
# // It defines one RPC, hello, which takes one arg, name, and returns a String.
# #[logimesh::component]
# trait World {
#     /// Returns a greeting for name.
#     async fn hello(name: String) -> String;
# }
/// This is the type that implements the generated World trait. It is the business logic
/// and is used to start the server.
#[derive(Clone)]
struct CompHello;

impl World for CompHello {
    // Each defined rpc generates an async fn that serves the RPC
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
}
```

### `server.rs` file

```rust
use clap::Parser;
use futures::future;
use futures::prelude::*;
use logimesh::server::incoming::Incoming;
use logimesh::server::{self, Channel};
use logimesh::tokio_serde::formats::Json;
use service::{CompHello, World};
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

    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), flags.port);

    // JSON transport is provided by the json_transport logimesh module. It makes it easy
    // to start up a serde-powered json serialization strategy over TCP.
    let mut listener = logimesh::transport::tcp::listen(&server_addr, Json::default).await?;
    println!("Listening on port {}", listener.local_addr().port());
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
```

### `client.rs` file

```rust
use clap::Parser;
use logimesh::client::balance::RandomBalance;
use logimesh::client::discover::FixedDiscover;
use logimesh::client::lrcall::ConfigExt;
use logimesh::component::Endpoint;
use logimesh::context;
use service::{init_tracing, CompHello, World};
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

    let client = CompHello
        .logimesh_lrcall(
            Endpoint::new("p.s.m"),
            FixedDiscover::from_address(vec![flags.server_addr.into()]),
            RandomBalance::new(),
            ConfigExt::default(),
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
```