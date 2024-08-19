# lrcall

`lrcall` is a Rust procedure call framework that is compatible with local and remote procedure calls.
And lrcall focuses on ease of use. Defining a
service can be done in just a few lines of code, and most of the boilerplate of
writing a server is taken care of for you.

Secondary development based on [google/tarpc](https://github.com/google/tarpc)

[![Crates.io](https://img.shields.io/crates/v/lrcall)](https://crates.io/crates/lrcall)
[![Documentation](https://shields.io/docsrs/lrcall)](https://docs.rs/lrcall)
[![License](https://img.shields.io/crates/l/lrcall)](https://github.com/andeya/logimesh/blob/main/lrcall/LICENSE)


## What is LPC?
"LPC" stands for "Local Procedure Call," a function call where the work of
producing the return value is being done locally.

## What is RPC?
"RPC" stands for "Remote Procedure Call," a function call where the work of
producing the return value is being done somewhere else. When an rpc function is
invoked, behind the scenes the function contacts some other process somewhere
and asks them to evaluate the function instead. The original function then
returns the value produced by the other process.

## Some features of lrcall:
- The client supports both local calls and remote calls simultaneously, meaning that users can dynamically switch the calling method according to the context.
- Defining the schema in code, rather than in a separate language such as .proto.
- Pluggable transport: any type implementing `Stream<Item = Request> + Sink<Response>` can be used as a transport to connect the client and server.
- `Send + 'static` optional: if the transport doesn't require it, neither does lrcall!
- Cascading cancellation: dropping a request will send a cancellation message to the server. The server will cease any unfinished work on the request, subsequently cancelling any of its own
  requests, repeating for the entire chain of transitive dependencies.
- Configurable deadlines and deadline propagation: request deadlines default to 10s if unspecified. The server will automatically cease work when the deadline has passed. Any requests sent by the
  server that use the request context will propagate the request deadline. For example, if a server is handling a request with a 10s deadline, does 2s of work, then sends a request to another
  server, that server will see an 8s deadline.
- Distributed tracing: lrcall is instrumented with [tracing](https://github.com/tokio-rs/tracing) primitives extended with [OpenTelemetry](https://opentelemetry.io/) traces. Using a compatible tracing
  subscriber like [OTLP](https://github.com/open-telemetry/opentelemetry-rust/tree/main/opentelemetry-otlp), each RPC can be traced through the client, server, and other dependencies downstream of
  the server. Even for applications not connected to a distributed tracing collector, the instrumentation can also be ingested by regular loggers like [env_logger](https://github.com/env-logger-rs/env_logger/).
- Serde serialization: enabling the `serde1` Cargo feature will make service requests and responses `Serialize + Deserialize`. It's entirely optional, though: in-memory transports can be used, as
  well, so the price of serialization doesn't have to be paid when it's not needed.

## Usage
Add to your `Cargo.toml` dependencies:

```toml
lrcall = "0.1"
```

The `lrcall::service` attribute expands to a collection of items that form an rpc service.
These generated types make it easy and ergonomic to write servers with less boilerplate.
Simply implement the generated service trait, and you're off to the races!

## Example

This example uses [tokio](https://tokio.rs), so add the following dependencies to
your `Cargo.toml`:

```toml
anyhow = "1.0"
futures = "0.3"
lrcall = { version = "0.1", features = ["tokio1"] }
tokio = { version = "1.0", features = ["macros"] }
```

In the following example, we use an in-process channel for communication between
client and server. In real code, you will likely communicate over the network.
For a more real-world example, see [lrcall-example](lrcall-example).

First, let's set up the dependencies and service definition.

```rust
# extern crate futures;

use futures::{
    prelude::*,
};
use lrcall::{
    client, context,
    server::{self, incoming::Incoming, Channel},
};

// This is the service definition. It looks a lot like a trait definition.
// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[lrcall::service]
trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}
```

This service definition generates a trait called `World`. Next we need to
implement it for our Server struct.

```rust
# extern crate futures;
# use futures::{
#     prelude::*,
# };
# use lrcall::{
#     client, context,
#     server::{self, incoming::Incoming},
# };
# // This is the service definition. It looks a lot like a trait definition.
# // It defines one RPC, hello, which takes one arg, name, and returns a String.
# #[lrcall::service]
# trait World {
#     /// Returns a greeting for name.
#     async fn hello(name: String) -> String;
# }
// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct HelloService;

impl World for HelloService {
    // Each defined rpc generates an async fn that serves the RPC
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
}
```

Lastly let's write our `main` that will start the server. While this example uses an
[in-process channel](transport::channel), lrcall also ships a generic [`serde_transport`]
behind the `serde-transport` feature, with additional [TCP](serde_transport::tcp) functionality
available behind the `tcp` feature.

```rust
# extern crate futures;
# use futures::{
#     prelude::*,
# };
# use lrcall::{
#     client, context,
#     server::{self, Channel},
# };
# // This is the service definition. It looks a lot like a trait definition.
# // It defines one RPC, hello, which takes one arg, name, and returns a String.
# #[lrcall::service]
# trait World {
#     /// Returns a greeting for name.
#     async fn hello(name: String) -> String;
# }
# // This is the type that implements the generated World trait. It is the business logic
# // and is used to start the server.
# #[derive(Clone)]
# struct HelloService;
# impl World for HelloService {
    // Each defined rpc generates an async fn that serves the RPC
#     async fn hello(self, _: context::Context, name: String) -> String {
#         format!("Hello, {name}!")
#     }
# }
# #[cfg(not(feature = "tokio1"))]
# fn main() {}
# #[cfg(feature = "tokio1")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (client_transport, server_transport) = lrcall::transport::channel::unbounded();

    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(
        server.execute(HelloService.serve())
            // Handle all requests concurrently.
            .for_each(|response| async move {
                tokio::spawn(response);
            }));

    // WorldClient is generated by the #[lrcall::service] attribute. It has a constructor `new`
    // that takes a config and any Transport as input.
    let mut client = WorldClient::<HelloService>::rpc_client(WorldChannel::spawn(client::Config::default(), client_transport));

    // The client has an RPC method for each RPC defined in the annotated trait. It takes the same
    // args as defined, with the addition of a Context, which is always the first arg. The Context
    // specifies a deadline and trace information which can be helpful in debugging requests.
    let hello = client.hello(context::rpc_current(), "Andeya".to_string()).await?;

    println!("{hello}");

    Ok(())
}
```