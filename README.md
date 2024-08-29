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
extern crate logimesh;

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
# extern crate logimesh;
#
# // This is the component definition. It looks a lot like a trait definition.
# // It defines one RPC, hello, which takes one arg, name, and returns a String.
# #[logimesh::component]
# trait World {
#     /// Returns a greeting for name.
#     async fn hello(name: String) -> String;
# }

use logimesh::context;
use logimesh::transport::codec::Codec;

/// This is the type that implements the generated World trait. It is the business logic
/// and is used to start the server.
#[derive(Clone)]
pub struct CompHello;

impl World for CompHello {
    const TRANSPORT_CODEC: Codec = Codec::Json;
    async fn hello(self, ctx: context::Context, name: String) -> String {
        format!("Hello, {name}! context: {:?}", ctx)
    }
}
```

### `server.rs` file

```rust
extern crate tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logimesh::tokio_tcp_listen!(CompHello, logimesh::server::TcpConfig::new("[::1]:8888".parse::<std::net::SocketAddrV6>().unwrap()));
    Ok(())
}
```

### `client.rs` file

```rust
extern crate tokio;
extern crate logimesh;
extern crate anyhow;

use logimesh::client::balance::RandomBalance;
use logimesh::client::discover::FixedDiscover;
use logimesh::client::lrcall::ConfigExt;
use logimesh::component::Endpoint;
use logimesh::context;
use service::{CompHello, World};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = CompHello
        .logimesh_lrcall(
            Endpoint::new("p.s.m"),
            FixedDiscover::from_address(vec!["[::1]:8888".parse::<std::net::SocketAddrV6>().unwrap()]),
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
    .await;

    match hello {
        Ok(hello) => println!("{hello:?}"),
        Err(e) => println!("{:?}", anyhow::Error::from(e)),
    }

    Ok(())
}
```
