// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! RPC Channel
use std::cell::Cell;

use super::CoreConfig;
use crate::client::{Channel, RpcError};
use crate::context;
use crate::server::Serve;
pub use crate::transport::codec::*;
use crate::transport::tcp;

/// Settings that control the behavior of the RPC client.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RpcConfig {
    /// server address.
    pub address: String,
    /// transport codec type.
    pub transport_codec: Codec,
    /// Settings that control the behavior of the underlying client.
    pub core_config: CoreConfig,
}

impl RpcConfig {
    /// Returns a default RpcConfig.
    pub fn new(address: String) -> Self {
        let core = CoreConfig::default();
        Self {
            address,
            transport_codec: Default::default(),
            core_config: CoreConfig::default(),
        }
    }
    /// Set transport serde codec
    pub fn with_transport_codec(mut self, transport_codec: Codec) -> Self {
        self.transport_codec = transport_codec;
        self
    }
    /// The number of requests that can be in flight at once.
    /// `max_in_flight_requests` controls the size of the map used by the client
    /// for storing pending requests.
    /// Default is 1000.
    pub fn with_max_in_flight_requests(mut self, max_in_flight_requests: usize) -> Self {
        self.core_config.max_in_flight_requests = max_in_flight_requests;
        self
    }
    /// The number of requests that can be buffered client-side before being sent.
    /// `pending_requests_buffer` controls the size of the channel clients use
    /// to communicate with the request dispatch task.
    /// Default is 100.
    pub fn with_pending_request_buffer(mut self, pending_request_buffer: usize) -> Self {
        self.core_config.pending_request_buffer = pending_request_buffer;
        self
    }
    /// Set the underlying client config.
    pub(crate) fn with_core_config(mut self, core_config: CoreConfig) -> Self {
        self.core_config = core_config;
        self
    }
}

/// RPC channel which is client stub
pub struct RpcChannel<S: Serve> {
    config: RpcConfig,
    channel: Channel<S::Req, S::Resp>,
    is_shutdown: Cell<bool>,
}

impl<S: Serve> crate::client::stub::Stub for RpcChannel<S> {
    type Req = S::Req;
    type Resp = S::Resp;

    async fn call(&self, ctx: context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        let res = self.channel.call(ctx, request).await;
        if let Err(RpcError::Shutdown) = res {
            self.is_shutdown.set(true);
        }
        res
    }
}

impl<S: Serve> RpcChannel<S>
where
    S: Serve + Clone,
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
{
    pub(crate) async fn new(config: RpcConfig) -> Result<Self, anyhow::Error> {
        let channe = Self::new_channel(config.transport_codec.clone(), config.core_config.clone(), config.address.as_str()).await?;
        Ok(Self {
            config,
            channel: channe,
            is_shutdown: Cell::new(false),
        })
    }
    async fn new_channel(transport_codec: Codec, stub_config: tarpc::client::Config, address: &str) -> Result<Channel<S::Req, S::Resp>, anyhow::Error> {
        match transport_codec {
            Codec::Bincode => {
                // Bincode codec using [bincode](https://docs.rs/bincode) crate.
                let mut conn = tcp::connect(address, Bincode::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            Codec::Json => {
                // JSON codec using [serde_json](https://docs.rs/serde_json) crate.
                let mut conn = tcp::connect(address, Json::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-messagepack")]
            MessagePack => {
                /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
                let mut conn = tcp::connect(address, MessagePack::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-cbor")]
            Cbor => {
                /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
                let mut conn = tcp::connect(address, Cbor::default);
                conn.config_mut().max_frame_length(usize::MAX);
                Ok(tarpc::client::new(stub_config, conn.await?).spawn())
            },
        }
    }
    pub(crate) async fn reconnent(&mut self) -> Result<(), anyhow::Error> {
        self.channel = Self::new_channel(self.config.transport_codec.clone(), self.config.core_config.clone(), self.config.address.as_str()).await?;
        self.is_shutdown.set(false);
        Ok(())
    }
}
