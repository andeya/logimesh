use futures::prelude::*;
use lrcall::server::{self, Channel};
use lrcall::{client, context};

#[allow(async_fn_in_trait)]
pub trait World: ::core::marker::Sized {
    #[doc = " Returns a greeting for name."]
    async fn hello(self, context: ::lrcall::context::Context, name: String) -> String;
    #[doc = r" Returns a serving function to use with"]
    #[doc = r" [InFlightRequest::execute](::lrcall::server::InFlightRequest::execute)."]
    fn serve(self) -> ServeWorld<Self> {
        ServeWorld { service: self }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct UnimplWorld;
impl World for UnimplWorld {
    #[doc = " Returns a greeting for name."]
    #[allow(unused_variables)]
    async fn hello(self, context: ::lrcall::context::Context, name: String) -> String {
        unimplemented!()
    }
}
#[doc = "The stub trait for service [`World`]."]
pub trait WorldRpcStub: ::lrcall::client::stub::Stub<Req = WorldRequest, Resp = WorldResponse> {}
impl<S> WorldRpcStub for S where S: ::lrcall::client::stub::Stub<Req = WorldRequest, Resp = WorldResponse> {}
#[doc = "The default WorldRpcStub implementation.\nUsage: `WorldChannel::spawn(config, transport)`"]
pub type WorldChannel = ::lrcall::client::Channel<WorldRequest, WorldResponse>;
#[doc = r" A serving function to use with [::lrcall::server::InFlightRequest::execute]."]
#[derive(Clone)]
pub struct ServeWorld<S> {
    service: S,
}
impl<S> ::lrcall::server::Serve for ServeWorld<S>
where
    S: World,
{
    type Req = WorldRequest;
    type Resp = WorldResponse;
    async fn serve(self, ctx: ::lrcall::context::Context, req: WorldRequest) -> ::core::result::Result<WorldResponse, ::lrcall::ServerError> {
        match req {
            WorldRequest::Hello { name } => ::core::result::Result::Ok(WorldResponse::Hello(World::hello(self.service, ctx, name).await)),
        }
    }
}
#[doc = r" The request sent over the wire from the client to the server."]
#[allow(missing_docs)]
#[derive(Debug)]
pub enum WorldRequest {
    Hello { name: String },
}
impl ::lrcall::RequestName for WorldRequest {
    fn name(&self) -> &'static str {
        match self {
            WorldRequest::Hello { .. } => "World.hello",
        }
    }
}
#[doc = r" The response sent over the wire from the server to the client."]
#[allow(missing_docs)]
#[derive(Debug)]
pub enum WorldResponse {
    Hello(String),
}
#[allow(unused, private_interfaces)]
#[derive(Clone, Debug)]
#[doc = r" The client stub that makes RPC calls to the server. All request methods return"]
#[doc = r" [Futures](::core::future::Future)."]
pub struct WorldClient<L = UnimplWorld, R = WorldChannel> {
    local_service: ::core::option::Option<L>,
    rpc_stub: ::core::option::Option<R>,
}
impl<L, R> WorldClient<L, R>
where
    L: World + ::core::clone::Clone,
    R: WorldRpcStub,
{
    #[doc = r" Return a new full client stub that supports both local calls and remote calls."]
    pub fn full_client(local_service: L, rpc_stub: R) -> Self {
        Self {
            local_service: ::core::option::Option::Some(local_service),
            rpc_stub: ::core::option::Option::Some(rpc_stub),
        }
    }
    #[doc = r" Return a new local client that supports local calls."]
    pub fn local_client(local_service: L) -> Self {
        Self {
            local_service: ::core::option::Option::Some(local_service),
            rpc_stub: ::core::option::Option::None,
        }
    }
    #[doc = r" Returns a new RPC client stub that supports remote calls."]
    pub fn rpc_client(rpc_stub: R) -> Self {
        Self {
            local_service: ::core::option::Option::None,
            rpc_stub: ::core::option::Option::Some(rpc_stub),
        }
    }
}
impl<L, R> WorldClient<L, R>
where
    L: World + ::core::clone::Clone,
    R: WorldRpcStub,
{
    #[allow(unused)]
    #[doc = " Returns a greeting for name."]
    pub async fn hello(&self, ctx: ::lrcall::context::Context, name: String) -> ::core::result::Result<String, ::lrcall::client::RpcError> {
        match ctx.call_type {
            ::lrcall::context::CallType::Local => {
                if let ::core::option::Option::Some(local_service) = &self.local_service {
                    return ::core::result::Result::Ok(local_service.clone().hello(ctx, name).await);
                }
            },
            ::lrcall::context::CallType::RPC => {
                if let ::core::option::Option::Some(rpc_stub) = &self.rpc_stub {
                    let request = WorldRequest::Hello { name };
                    let resp = rpc_stub.call(ctx, request);
                    return match resp.await? {
                        WorldResponse::Hello(msg) => ::core::result::Result::Ok(msg),
                        _ => ::core::unreachable!(),
                    };
                }
            },
        }
        return ::core::result::Result::Err(::lrcall::client::RpcError::ClientUnconfigured(ctx.call_type));
    }
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
