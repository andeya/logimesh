use lrcall::client;

#[lrcall::service]
trait World {
    async fn hello(name: String) -> String;
}

fn main() {
    let (client_transport, _) = lrcall::transport::channel::unbounded();

    #[deny(unused_must_use)]
    {
        WorldClient::new(client::Config::default(), client_transport).dispatch;
    }
}
