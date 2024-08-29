// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! RPC Channel

use crate::client::core::stub::Stub;
use crate::client::core::{Channel, Config, RpcError};
use crate::client::discover::Instance;
use crate::context;
use crate::net::Address;
use crate::server::Serve;
use crate::transport::codec::*;
use crate::transport::tcp;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Settings that control the behavior of the RPC client.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RpcConfig {
    /// service instance.
    pub instance: Arc<Instance>,
    /// transport codec type.
    pub transport_codec: Codec,
    /// Settings that control the behavior of the underlying client.
    pub core_config: Config,
    /// Maximum frame length, default is usize::MAX.
    pub max_frame_len: usize,
}

impl RpcConfig {
    /// Returns a default RpcConfig.
    pub fn new(instance: Arc<Instance>) -> Self {
        Self {
            instance,
            transport_codec: Default::default(),
            core_config: Config::default(),
            max_frame_len: usize::MAX,
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
    /// Set maximum frame length, zero is usize::MAX.
    pub fn with_max_frame_len(mut self, max_frame_len: usize) -> Self {
        if max_frame_len <= 0 {
            self.max_frame_len = usize::MAX;
        } else {
            self.max_frame_len = max_frame_len;
        }
        self
    }
    /// Set the underlying client config.
    #[allow(dead_code)]
    pub(crate) fn with_core_config(mut self, core_config: Config) -> Self {
        self.core_config = core_config;
        self
    }
    // init config.
    fn init(&mut self) {
        if self.max_frame_len <= 0 {
            self.max_frame_len = usize::MAX;
        }
    }
}

/// RPC channel which is client stub
pub struct RpcChannel<S: Serve> {
    inner: Arc<InnerRpcChannel<S::Req, S::Resp>>,
}

impl<S: Serve> Clone for RpcChannel<S> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

struct InnerRpcChannel<Req, Resp> {
    config: RpcConfig,
    channel: Arc<RwLock<Option<Channel<Req, Resp>>>>,
}

impl<S> Debug for RpcChannel<S>
where
    S: Serve,
    S::Req: Debug,
    S::Resp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcChannel").field("inner", &self.inner).finish()
    }
}

impl<Req, Resp> Debug for InnerRpcChannel<Req, Resp>
where
    Req: Debug,
    Resp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerRpcChannel").field("config", &self.config).field("channel", &self.channel).finish()
    }
}

impl<S: Serve> Stub for RpcChannel<S> {
    type Req = S::Req;
    type Resp = S::Resp;

    async fn call(&self, ctx: context::Context, request: Self::Req) -> Result<Self::Resp, RpcError> {
        let res = if let Some(channel) = &*self.inner.channel.read().await {
            channel.call(ctx, request).await
        } else {
            Err(RpcError::Shutdown)
        };
        if let Err(RpcError::Shutdown) = res {
            self.inner.channel.write().await.take();
        }
        res
    }
}

impl<S: Serve> RpcChannel<S> {
    /// Returns config
    pub fn config(&self) -> &RpcConfig {
        &self.inner.config
    }
}

impl<S> RpcChannel<S>
where
    S: Serve,
    S::Req: crate::serde::Serialize + Send + 'static,
    S::Resp: for<'de> crate::serde::Deserialize<'de> + Send + 'static,
{
    pub(crate) async fn new(mut config: RpcConfig) -> Result<Self, anyhow::Error> {
        config.init();
        let channe = Self::new_channel(&config).await?;
        Ok(Self {
            inner: Arc::new(InnerRpcChannel {
                config,
                channel: Arc::new(RwLock::new(Some(channe))),
            }),
        })
    }

    async fn new_channel(config: &RpcConfig) -> Result<Channel<S::Req, S::Resp>, anyhow::Error> {
        let Address::Ip(address) = &config.instance.address else {
            anyhow::bail!("invalid address {}", &config.instance.address)
        };
        match config.transport_codec {
            Codec::Bincode => {
                // Bincode codec using [bincode](https://docs.rs/bincode) crate.
                let mut conn = tcp::connect(address, Bincode::default);
                conn.config_mut().max_frame_length(config.max_frame_len);
                Ok(tarpc::client::new(config.core_config.clone(), conn.await?).spawn())
            },
            Codec::Json => {
                // JSON codec using [serde_json](https://docs.rs/serde_json) crate.
                let mut conn = tcp::connect(address, Json::default);
                conn.config_mut().max_frame_length(config.max_frame_len);
                Ok(tarpc::client::new(config.core_config.clone(), conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-messagepack")]
            Codec::MessagePack => {
                /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
                let mut conn = tcp::connect(address, MessagePack::default);
                conn.config_mut().max_frame_length(config.max_frame_len);
                Ok(tarpc::client::new(config.core_config.clone(), conn.await?).spawn())
            },
            #[cfg(feature = "serde-transport-cbor")]
            Codec::Cbor => {
                /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
                let mut conn = tcp::connect(address, Cbor::default);
                conn.config_mut().max_frame_length(config.max_frame_len);
                Ok(tarpc::client::new(config.core_config.clone(), conn.await?).spawn())
            },
        }
    }

    pub(crate) async fn reconnent(&self) -> Result<(), anyhow::Error> {
        let channel = Self::new_channel(&self.inner.config).await?;
        self.inner.channel.write().await.replace(channel);
        Ok(())
    }

    pub(crate) fn clone_update_instance(&self, instance: Arc<Instance>) -> Self {
        let mut inner = InnerRpcChannel {
            config: self.config().clone(),
            channel: self.inner.channel.clone(),
        };
        inner.config.instance = instance;
        Self { inner: Arc::new(inner) }
    }
}
