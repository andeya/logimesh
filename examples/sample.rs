use futures::prelude::*;
use tarpc::server::{self, Channel};
use tarpc::{client, context};

// This is the service definition. It looks a lot like a trait definition.
// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[tarpc::service]
trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone, Debug)]
struct HelloServer;

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let local_service=HelloServer;
    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(
        server
            .execute(local_service.clone().serve())
            // Handle all requests concurrently.
            .for_each(|response| async move {
                tokio::spawn(response);
            }),
    );
    let api = WorldAPI::new(local_service, client::Config::default(), client_transport);

    let hello = api.hello(Context::current(false), "Stim".to_string()).await?;

    println!("{hello}");

    Ok(())
}

struct WorldAPI<LS> {
    client: WorldClient<tarpc::client::Channel<WorldRequest, WorldResponse>>,
    local_service: LS,
}

struct Context {
    pub rpc: context::Context,
    pub is_rpc: bool,
}
impl Context {
    fn current(is_rpc: bool) -> Self {
        Context {
            rpc: context::Context::current(),
            is_rpc,
        }
    }
}

impl<LS> WorldAPI<LS>
where
    LS: World + Clone,
{
    fn new<T>(local_server: LS, config: ::tarpc::client::Config, client_transport: T) -> Self
    where
        T: ::tarpc::Transport<::tarpc::ClientMessage<WorldRequest>, ::tarpc::Response<WorldResponse>> + Send + 'static,
    {
        Self {
            client: WorldClient::new(config, client_transport).spawn(),
            local_service: local_server,
        }
    }
    async fn hello(&self, ctx: Context, name: String) -> ::core::result::Result<String, ::tarpc::client::RpcError> {
        if ctx.is_rpc {
            self.client.hello(ctx.rpc, name).await
        } else {
            Ok(self.local_service.clone().hello(ctx.rpc, name).await)
        }
    }
}
