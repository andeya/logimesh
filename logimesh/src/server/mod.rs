// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Server component.

use tokio::net::ToSocketAddrs;

pub use core::*;

mod core {
    pub use ::tarpc::server::*;
}

/// TCP server config.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct TcpConfig<A: ToSocketAddrs> {
    /// listen address.
    pub(crate) listen_address: A,
    /// Maximum frame length, default is usize::MAX.
    pub(crate) max_frame_len: usize,
    /// Controls the buffer size of the in-process channel over which a server's handlers send
    /// responses to the [`Channel`]. In other words, this is the number of responses that can sit
    /// in the outbound queue before request handlers begin blocking.
    /// Default is 100.
    pub(crate) pending_response_buffer: usize,
    /// Enforces channel per-key limits.
    pub(crate) max_channels_per_key: u32,
    /// An adaptor for creating a buffered list of pending futures (unordered).
    /// Default is 10, and zero means 10.
    pub(crate) buffer_unordered: usize,
}

impl<A: ToSocketAddrs> TcpConfig<A> {
    /// Create a new TCP config.
    pub fn new(listen_address: A) -> Self {
        let server_config = Config::default();
        Self {
            listen_address,
            max_frame_len: usize::MAX,
            pending_response_buffer: server_config.pending_response_buffer,
            max_channels_per_key: Default::default(),
            buffer_unordered: 10,
        }
    }
    /// listen address.
    pub fn listen_address(&self) -> &A {
        &self.listen_address
    }
    /// Set maximum frame length, default is usize::MAX.
    pub fn with_max_frame_len(mut self, max_frame_len: usize) -> Self {
        if max_frame_len <= 0 {
            self.max_frame_len = usize::MAX;
        } else {
            self.max_frame_len = max_frame_len;
        }
        self
    }
    /// Maximum frame length, default is usize::MAX.
    pub fn max_frame_len(&self) -> usize {
        self.max_frame_len
    }
    /// Set the buffer size of the in-process channel over which a server's handlers send
    /// responses to the [`Channel`]. In other words, this is the number of responses that can sit
    /// in the outbound queue before request handlers begin blocking.
    /// Default is 100.
    pub fn with_pending_response_buffer(mut self, pending_response_buffer: usize) -> Self {
        if pending_response_buffer <= 0 {
            self.pending_response_buffer = usize::MAX;
        } else {
            self.pending_response_buffer = pending_response_buffer;
        }
        self
    }
    /// Controls the buffer size of the in-process channel over which a server's handlers send
    /// responses to the [`Channel`]. In other words, this is the number of responses that can sit
    /// in the outbound queue before request handlers begin blocking.
    /// Default is 100.
    pub fn pending_response_buffer(&self) -> usize {
        self.pending_response_buffer
    }
    /// Set up enforces channel per-key limits.
    pub fn with_max_channels_per_key(mut self, max_channels_per_key: u32) -> Self {
        self.max_channels_per_key = max_channels_per_key;
        self
    }
    /// Enforces channel per-key limits.
    pub fn max_channels_per_key(&self) -> u32 {
        self.max_channels_per_key
    }
    /// Set an adaptor for creating a buffered list of pending futures (unordered).
    /// Default is 10, and zero means 10.
    pub fn with_buffer_unordered(mut self, buffer_unordered: usize) -> Self {
        if buffer_unordered <= 0 {
            self.buffer_unordered = 10;
        } else {
            self.buffer_unordered = buffer_unordered;
        }
        self
    }
    /// An adaptor for creating a buffered list of pending futures (unordered).
    /// Default is 10, and zero means 10.
    pub fn buffer_unordered(&self) -> usize {
        self.buffer_unordered
    }
}

/// Listen a TCP server.
/// # Example:
/// ```
/// extern crate tokio;
/// extern crate anyhow;
/// extern crate logimesh;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     logimesh::tokio_tcp_listen!(CompHello, logimesh::server::TcpConfig::new("[::1]:8888".parse::<std::net::SocketAddrV6>().unwrap()));
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! tokio_tcp_listen {
    ($component:expr, $tcp_config:expr $(,)?) => {{
        use ::logimesh::futures::prelude::*;
        use ::logimesh::server::incoming::Incoming as _;
        use ::logimesh::server::Channel as _;
        let serve = $component.logimesh_serve();
        let mut listener = ::logimesh::transport::tcp::listen($tcp_config.listen_address(), $component.__logimesh_codec().to_fn()).await.unwrap();
        ::logimesh::tracing::info!("[LOGIMESH] Listening on {}", listener.local_addr());
        listener.config_mut().max_frame_length($tcp_config.max_frame_len());
        listener
            // Ignore accept errors.
            .filter_map(|r| future::ready(r.ok()))
            .map(|transport| {
                ::logimesh::server::BaseChannel::new(
                    ::logimesh::server::Config {
                        pending_response_buffer: $tcp_config.pending_response_buffer(),
                    },
                    transport,
                )
            })
            // Limit channels to ${max_channels_per_key} per IP.
            .max_channels_per_key($tcp_config.max_channels_per_key(), |t| t.transport().peer_addr().unwrap().ip())
            // serve is generated by the component attribute. It takes as input any type implementing
            // the generated World trait.
            .map(|channel| {
                channel.execute(serve.clone()).for_each(|fut| async {
                    ::tokio::spawn(fut);
                })
            })
            // Max 10 channels.
            .buffer_unordered($tcp_config.buffer_unordered())
            .for_each(|_| async {})
            .await;
    }};
}
