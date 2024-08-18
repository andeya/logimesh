use futures::prelude::*;
use lrcall::server::incoming::Incoming;
use lrcall::server::BaseChannel;
use lrcall::{client, context, serde_transport};
use tokio_serde::formats::Json;

#[lrcall::derive_serde]
#[derive(Debug, PartialEq, Eq)]
pub enum TestData {
    Black,
    White,
}

#[lrcall::service]
pub trait ColorProtocol {
    async fn get_opposite_color(color: TestData) -> TestData;
}

#[derive(Clone)]
struct ColorServer;

impl ColorProtocol for ColorServer {
    async fn get_opposite_color(self, _: context::Context, color: TestData) -> TestData {
        match color {
            TestData::White => TestData::Black,
            TestData::Black => TestData::White,
        }
    }
}

#[cfg(test)]
async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::test]
async fn test_call() -> anyhow::Result<()> {
    let transport = lrcall::serde_transport::tcp::listen("localhost:56797", Json::default).await?;
    let addr = transport.local_addr();
    tokio::spawn(
        transport
            .take(1)
            .filter_map(|r| async { r.ok() })
            .map(BaseChannel::with_defaults)
            .execute(ColorServer.serve())
            .map(|channel| channel.for_each(spawn))
            .for_each(spawn),
    );

    let transport = serde_transport::tcp::connect(addr, Json::default).await?;
    let client = ColorProtocolClient::<ColorServer>::rpc_client((client::Config::default(), transport).into());

    let color = client.get_opposite_color(context::rpc_current(), TestData::White).await?;
    assert_eq!(color, TestData::Black);

    Ok(())
}
