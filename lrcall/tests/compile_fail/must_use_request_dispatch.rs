use lrcall::client;

#[lrcall::service]
trait World {
    async fn hello(name: String) -> String;
}

fn main() {
    let (client_transport, _) = lrcall::transport::channel::unbounded();

    #[deny(unused_must_use)]
    {
        WorldClient::<HelloService>::rpc_client((client::Config::default(), client_transport).into()).dispatch;
    }
}
