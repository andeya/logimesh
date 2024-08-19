use futures::prelude::*;
use lrcall::server::{self, Channel};
use lrcall::{client, context};

// This is the service definition. It looks a lot like a trait definition.
// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[lrcall::service]
pub trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone, Debug)]
pub struct HelloService;

impl World for HelloService {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let local_service = HelloService;
    let (client_transport, server_transport) = lrcall::transport::channel::unbounded();

    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(
        server
            .execute(local_service.clone().serve())
            // Handle all requests concurrently.
            .for_each(|response| async move {
                tokio::spawn(response);
            }),
    );

    let api = WorldClient::<UnimplWorld>::rpc_client(WorldChannel::spawn(client::Config::default(), client_transport));
    let hello = api.hello(context::Context::current(context::CallType::RPC), "Stim".to_string()).await?;
    println!("RPC: {hello}");

    let api = WorldClient::<HelloService>::local_client(local_service);
    let hello = api.hello(context::Context::current(context::CallType::Local), "Stim".to_string()).await?;
    println!("Local: {hello}");
    Ok(())
}
