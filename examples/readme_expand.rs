use futures::prelude::*;
use logimesh::server::{self, Channel};
use logimesh::{client, context};

#[doc = " This is the service definition. It looks a lot like a trait definition."]
#[doc = " It defines one RPC, hello, which takes one arg, name, and returns a String."]
#[allow(async_fn_in_trait)]
pub trait World: ::core::marker::Sized + ::core::clone::Clone {
    async fn hello(self, context: ::logimesh::context::Context, name: String) -> String;
    #[doc = r" Returns a serving function to use with"]
    #[doc = r" [InFlightRequest::execute](::logimesh::server::InFlightRequest::execute)."]
    fn serve(self) -> ServeWorld<Self> {
        ServeWorld { service: self }
    }
    #[doc = r" Returns a client that supports both local calls and remote calls."]
    fn client<ServiceLookup: ::logimesh::discover::ServiceLookup>(
        self,
        config: ::logimesh::client::stub::lrcall::Config<ServiceLookup>,
    ) -> WorldClient<::logimesh::client::stub::lrcall::LRCall<ServeWorld<Self>, ServiceLookup>> {
        ::logimesh::client::stub::lrcall::LRCall::new(ServeWorld { service: self }, config).into()
    }
}
#[derive(Debug, Clone, Copy)]
pub struct UnimplWorld;
impl World for UnimplWorld {
    #[allow(unused_variables)]
    async fn hello(self, context: ::logimesh::context::Context, name: String) -> String {
        unimplemented!()
    }
}
#[doc = " The stub trait for service [`World`]."]
pub trait WorldStub: ::logimesh::client::stub::Stub<Req = WorldRequest, Resp = WorldResponse> {}
impl<S> WorldStub for S where S: ::logimesh::client::stub::Stub<Req = WorldRequest, Resp = WorldResponse> {}
#[doc = " The default WorldStub implementation."]
#[doc = " Usage: `WorldChannel::spawn(config, transport)`"]
pub type WorldChannel = ::logimesh::client::Channel<WorldRequest, WorldResponse>;
#[doc = r" A serving function to use with [::logimesh::server::InFlightRequest::execute]."]
#[derive(Clone)]
pub struct ServeWorld<S> {
    service: S,
}
impl<S> ::logimesh::server::Serve for ServeWorld<S>
where
    S: World,
{
    type Req = WorldRequest;
    type Resp = WorldResponse;
    async fn serve(self, ctx: ::logimesh::context::Context, req: WorldRequest) -> ::core::result::Result<WorldResponse, ::logimesh::ServerError> {
        match req {
            WorldRequest::Hello { name } => ::core::result::Result::Ok(WorldResponse::Hello(World::hello(self.service, ctx, name).await)),
        }
    }
}
#[doc = r" The request sent over the wire from the client to the server."]
#[allow(missing_docs)]
#[derive(
    Debug,
    :: logimesh :: serde :: Serialize,
    :: logimesh :: serde ::
Deserialize,
)]
#[serde(crate = "::logimesh::serde")]
pub enum WorldRequest {
    Hello { name: String },
}
impl ::logimesh::RequestName for WorldRequest {
    fn name(&self) -> &'static str {
        match self {
            WorldRequest::Hello { .. } => "World.hello",
        }
    }
}
#[doc = r" The response sent over the wire from the server to the client."]
#[allow(missing_docs)]
#[derive(
    Debug,
    :: logimesh :: serde :: Serialize,
    :: logimesh :: serde ::
Deserialize,
)]
#[serde(crate = "::logimesh::serde")]
pub enum WorldResponse {
    Hello(String),
}
#[allow(unused, private_interfaces)]
#[derive(Clone, Debug)]
#[doc = r" The client that makes LPC or RPC calls to the server. All request methods return"]
#[doc = r" [Futures](::core::future::Future)."]
pub struct WorldClient<Stub = WorldChannel>(Stub);
impl WorldClient {
    #[doc = r" Returns a new client that sends requests over the given transport."]
    pub fn new<T>(config: ::logimesh::client::Config, transport: T) -> ::logimesh::client::NewClient<Self, ::logimesh::client::RequestDispatch<WorldRequest, WorldResponse, T>>
    where
        T: ::logimesh::Transport<::logimesh::ClientMessage<WorldRequest>, ::logimesh::Response<WorldResponse>>,
    {
        let new_client = ::logimesh::client::new(config, transport);
        ::logimesh::client::NewClient {
            client: WorldClient(new_client.client),
            dispatch: new_client.dispatch,
        }
    }
}
impl<Stub> ::core::convert::From<Stub> for WorldClient<Stub>
where
    Stub: ::logimesh::client::stub::Stub<Req = WorldRequest, Resp = WorldResponse>,
{
    #[doc = r" Returns a new client that sends requests over the given transport."]
    fn from(stub: Stub) -> Self {
        WorldClient(stub)
    }
}
impl<Stub> WorldClient<Stub>
where
    Stub: ::logimesh::client::stub::Stub<Req = WorldRequest, Resp = WorldResponse>,
{
    #[allow(unused)]
    pub fn hello(&self, ctx: ::logimesh::context::Context, name: String) -> impl ::core::future::Future<Output = ::core::result::Result<String, ::logimesh::client::RpcError>> + '_ {
        let request = WorldRequest::Hello { name };
        let resp = self.0.call(ctx, request);
        async move {
            match resp.await? {
                WorldResponse::Hello(msg) => ::core::result::Result::Ok(msg),
                _ => ::core::unreachable!(),
            }
        }
    }
}

/// This is the type that implements the generated World trait. It is the business logic
/// and is used to start the server.
#[derive(Clone)]
struct HelloServer;

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (client_transport, server_transport) = logimesh::transport::channel::unbounded();

    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(server.execute(HelloServer.serve()).for_each(spawn));

    // WorldClient is generated by the #[logimesh::service] attribute. It has a constructor `new`
    // that takes a config and any Transport as input.
    let client = WorldClient::new(client::Config::default(), client_transport).spawn();

    // The client has an RPC method for each RPC defined in the annotated trait. It takes the same
    // args as defined, with the addition of a Context, which is always the first arg. The Context
    // specifies a deadline and trace information which can be helpful in debugging requests.
    let hello = client.hello(context::current(), "Stim".to_string()).await?;

    println!("{hello}");

    Ok(())
}