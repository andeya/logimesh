// Modifications Copyright Andeya Lee 2024
// Based on original source code from Google LLC licensed under MIT
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use futures::prelude::*;
use logimesh::server::{self, Channel};
use logimesh::{client, context};

/// This is the component definition. It looks a lot like a trait definition.
/// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[logimesh::component]
pub trait World {
    async fn hello(name: String) -> String;
}

/// This is the type that implements the generated World trait. It is the business logic
/// and is used to start the server.
#[derive(Clone)]
struct CompHello;

impl World for CompHello {
    // Each defined rpc generates an async fn that serves the RPC
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
    fn logimesh_should_retry(_result: &::core::result::Result<WorldResponse, ::logimesh::client::core::RpcError>, _tried_times: u32) -> bool {
        false
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }

    let (client_transport, server_transport) = logimesh::transport::channel::unbounded();

    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(server.execute(CompHello.logimesh_serve()).for_each(spawn));

    // WorldClient is generated by the #[logimesh::component] attribute. It has a constructor `new`
    // that takes a config and any Transport as input.
    let client = WorldClient::new(client::core::Config::default(), client_transport).spawn();

    // The client has an RPC method for each RPC defined in the annotated trait. It takes the same
    // args as defined, with the addition of a Context, which is always the first arg. The Context
    // specifies a deadline and trace information which can be helpful in debugging requests.
    let hello = client.hello(context::current(), "Andeya".to_string()).await?;

    println!("{hello}");

    Ok(())
}
