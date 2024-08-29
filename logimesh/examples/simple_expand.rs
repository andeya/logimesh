use futures::prelude::*;
use logimesh::server::{self, Channel};
use logimesh::{client, context};
#[doc = " This is the component definition. It looks a lot like a trait definition."]
#[doc = " It defines one RPC, hello, which takes one arg, name, and returns a String."]
#[allow(async_fn_in_trait)]
pub trait World: ::core::marker::Sized + ::core::clone::Clone + 'static {
    #[doc = r" The transport codec."]
    const TRANSPORT_CODEC: ::logimesh::transport::codec::Codec = ::logimesh::transport::codec::Codec::Bincode;
    async fn hello(self, context: ::logimesh::context::Context, name: String) -> String;
    #[doc = r" Returns a serving function to use with"]
    #[doc = r" [InFlightRequest::execute](::logimesh::server::InFlightRequest::execute)."]
    fn logimesh_serve(self) -> ServeWorld<Self> {
        ServeWorld { service: self }
    }
    #[doc = r" Returns a client that supports local and remote calls."]
    async fn logimesh_lrclient<D, LB>(
        self,
        endpoint: ::logimesh::component::Endpoint,
        discover: D,
        load_balance: LB,
        config_ext: ::logimesh::client::lrcall::ConfigExt,
    ) -> ::core::result::Result<WorldLRClient<Self, D, LB>, ::logimesh::client::ClientError>
    where
        D: ::logimesh::client::discover::Discover,
        LB: ::logimesh::client::balance::LoadBalance<ServeWorld<Self>>,
    {
        Ok(WorldLRClient(WorldClient(
            ::logimesh::client::lrcall::Builder::<ServeWorld<Self>, D, LB, fn(&::core::result::Result<WorldResponse, ::logimesh::client::core::RpcError>, u32) -> bool>::new(
                ::logimesh::component::Component {
                    serve: ServeWorld { service: self },
                    endpoint,
                },
                discover,
                load_balance,
            )
            .with_config_ext(config_ext)
            .with_transport_codec(Self::TRANSPORT_CODEC)
            .with_retry_fn(Self::logimesh_should_retry)
            .try_spawn()
            .await?,
        )))
    }
    #[doc = r" Judge whether a retry should be made according to the result returned by the call."]
    #[doc = r" When `::logimesh::client::stub::Config.enable_retry` is true, the method will be called."]
    #[doc = r" So you should implement your own version."]
    #[allow(unused_variables)]
    fn logimesh_should_retry(result: &::core::result::Result<WorldResponse, ::logimesh::client::core::RpcError>, tried_times: u32) -> bool {
        false
    }
    #[doc = r" Returns the Self::TRANSPORT_CODEC."]
    #[doc = r" NOTE: Implementation is not allowed to be overridden."]
    #[doc = r" If you need to modify the encoder, Self::TRANSPORT_CODEC should be specified."]
    fn __logimesh_codec(&self) -> ::logimesh::transport::codec::Codec {
        Self::TRANSPORT_CODEC
    }
}
#[derive(Debug, Clone, Copy)]
pub struct UnimplWorld;
impl World for UnimplWorld {
    const TRANSPORT_CODEC: ::logimesh::transport::codec::Codec = ::logimesh::transport::codec::Codec::Bincode;
    #[allow(unused_variables)]
    async fn hello(self, context: ::logimesh::context::Context, name: String) -> String {
        unimplemented!()
    }
}
#[doc = " The stub trait for service [`World`]."]
pub trait WorldStub: ::logimesh::client::Stub<Req = WorldRequest, Resp = WorldResponse> {}
impl<S> WorldStub for S where S: ::logimesh::client::Stub<Req = WorldRequest, Resp = WorldResponse> {}
#[doc = " The default WorldStub implementation."]
#[doc = " Usage: `WorldChannel::spawn(config, transport)`"]
pub type WorldChannel = ::logimesh::client::core::Channel<WorldRequest, WorldResponse>;
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
    Clone,
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
    pub fn new<T>(config: ::logimesh::client::core::Config, transport: T) -> ::logimesh::client::core::NewClient<Self, ::logimesh::client::core::RequestDispatch<WorldRequest, WorldResponse, T>>
    where
        T: ::logimesh::Transport<::logimesh::ClientMessage<WorldRequest>, ::logimesh::Response<WorldResponse>>,
    {
        let new_client = ::logimesh::client::core::new(config, transport);
        ::logimesh::client::core::NewClient {
            client: WorldClient(new_client.client),
            dispatch: new_client.dispatch,
        }
    }
}
impl<Stub> ::core::convert::From<Stub> for WorldClient<Stub>
where
    Stub: ::logimesh::client::Stub<Req = WorldRequest, Resp = WorldResponse>,
{
    #[doc = r" Returns a new client that sends requests over the given transport."]
    fn from(stub: Stub) -> Self {
        WorldClient(stub)
    }
}
impl<Stub> WorldClient<Stub>
where
    Stub: ::logimesh::client::Stub<Req = WorldRequest, Resp = WorldResponse>,
{
    #[allow(unused)]
    pub fn hello(&self, ctx: ::logimesh::context::Context, name: String) -> impl ::core::future::Future<Output = ::core::result::Result<String, ::logimesh::client::core::RpcError>> + '_ {
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
#[doc = r" A client new-type that supports local and remote calls."]
pub struct WorldLRClient<S, D, LB>(WorldClient<::logimesh::client::lrcall::LRCall<ServeWorld<S>, D, LB, fn(&::core::result::Result<WorldResponse, ::logimesh::client::core::RpcError>, u32) -> bool>>)
where
    S: World,
    D: ::logimesh::client::discover::Discover,
    LB: ::logimesh::client::balance::LoadBalance<ServeWorld<S>>;
impl<S, D, LB> ::std::ops::Deref for WorldLRClient<S, D, LB>
where
    S: World,
    D: ::logimesh::client::discover::Discover,
    LB: ::logimesh::client::balance::LoadBalance<ServeWorld<S>>,
{
    type Target = WorldClient<::logimesh::client::lrcall::LRCall<ServeWorld<S>, D, LB, fn(&::core::result::Result<WorldResponse, ::logimesh::client::core::RpcError>, u32) -> bool>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This is the type that implements the generated World trait. It is the business logic
/// and is used to start the server.
#[derive(Clone)]
struct CompHello;

impl World for CompHello {
    const TRANSPORT_CODEC: ::logimesh::transport::codec::Codec = ::logimesh::transport::codec::Codec::Bincode;
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
