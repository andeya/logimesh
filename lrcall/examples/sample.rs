use anyhow::{anyhow, Ok};
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
    let lpc_service = HelloService;
    let (client_transport, server_transport) = lrcall::transport::channel::unbounded();

    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(
        server
            .execute(lpc_service.clone().serve())
            // Handle all requests concurrently.
            .for_each(|response| async move {
                tokio::spawn(response);
            }),
    );

    let rpc_stub = WorldChannel::spawn(client::Config::default(), client_transport);
    let local_ctx = context::Context::current(context::CallType::LPC);
    let rpc_ctx = context::Context::current(context::CallType::RPC);

    {
        let api = WorldClient::full_client(HelloService, rpc_stub.clone());
        let hello = api.hello(local_ctx, "Andeya---full-LPC".to_string()).await?;
        println!("full-LPC: {hello}");
        let hello = api.hello(rpc_ctx, "Andeya---full-RPC".to_string()).await?;
        println!("full-RPC: {hello}");
    }

    {
        let api = WorldClient::<UnimplWorld>::rpc_client(rpc_stub);
        let unimplemented = api.hello(local_ctx, "Andeya---rpc-LPC".to_string()).await.map_or_else(|e| Ok(e), |t| Err(anyhow!(t)))?;
        println!("rpc-LPC: {unimplemented:?}");
        let hello = api.hello(rpc_ctx, "Andeya---rpc-RPC".to_string()).await?;
        println!("rpc-RPC: {hello}");
    }

    {
        let api = WorldClient::<HelloService>::lpc_client(lpc_service);
        let hello = api.hello(local_ctx, "Andeya---lpc-LPC".to_string()).await?;
        println!("lpc-LPC: {hello}");
        let unimplemented = api.hello(rpc_ctx, "Andeya---lpc-RPC".to_string()).await.map_or_else(|e| Ok(e), |t| Err(anyhow!(t)))?;
        println!("lpc-RPC: {unimplemented:?}");
    }
    Ok(())
}
