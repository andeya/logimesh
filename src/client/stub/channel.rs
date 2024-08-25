use std::cell::Cell;

use crate::client::{Channel, RpcError};
use crate::serde_transport::tcp;
pub use crate::tokio_serde::formats;
use crate::{context, server};

use super::TransportCodec;

pub(crate) struct ChannelInstance<Serve: server::Serve> {
    address: String,
    transport_codec: TransportCodec,
    stub_config: tarpc::client::Config,
    channel: Channel<Serve::Req, Serve::Resp>,
    is_shutdown: Cell<bool>,
}

impl<Serve: server::Serve> crate::client::stub::Stub for ChannelInstance<Serve> {
    type Req = Serve::Req;
    type Resp = Serve::Resp;

    async fn call(&self, ctx: context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        let res = self.channel.call(ctx, request).await;
        if let Err(RpcError::Shutdown) = res {
            self.is_shutdown.set(true);
        }
        res
    }
}

impl<Serve: server::Serve> ChannelInstance<Serve>
where
    Serve: server::Serve + Clone,
    Serve::Req: crate::serde::Serialize + Send + 'static,
    Serve::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
{
    async fn new(address: String, transport_codec: TransportCodec, stub_config: tarpc::client::Config) -> Result<Self, anyhow::Error> {
        let channe = Self::new_channel(transport_codec.clone(), stub_config.clone(), address.as_str()).await?;
        Ok(Self {
            address,
            transport_codec,
            stub_config,
            channel: channe,
            is_shutdown: Cell::new(false),
        })
    }
    async fn new_channel(transport_codec: TransportCodec, stub_config: tarpc::client::Config, address: &str) -> Result<Channel<Serve::Req, Serve::Resp>, anyhow::Error> {
        match transport_codec {
            TransportCodec::Bincode => {
                // Bincode codec using [bincode](https://docs.rs/bincode) crate.
                let mut conn = tcp::connect(address, formats::Bincode::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            TransportCodec::Json => {
                // JSON codec using [serde_json](https://docs.rs/serde_json) crate.
                let mut conn = tcp::connect(address, formats::Json::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-messagepack")]
            MessagePack => {
                /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
                let mut conn = tcp::connect(address, formats::MessagePack::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-cbor")]
            Cbor => {
                /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
                let mut conn = tcp::connect(address, formats::Cbor::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
        }
    }
    async fn reconnent(&mut self) -> Result<(), anyhow::Error> {
        self.channel = Self::new_channel(self.transport_codec.clone(), self.stub_config.clone(), self.address.as_str()).await?;
        self.is_shutdown.set(false);
        Ok(())
    }
}
