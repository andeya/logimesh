// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Server component.

use tokio::net::ToSocketAddrs;

use crate::transport::codec::Codec;
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
    /// transport codec type.
    pub(crate) transport_codec: Codec,
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
            transport_codec: Default::default(),
            max_frame_len: usize::MAX,
            pending_response_buffer: server_config.pending_response_buffer,
            max_channels_per_key: Default::default(),
            buffer_unordered: 10,
        }
    }
    /// Set listen address.
    pub fn with_listen_address(mut self, listen_address: A) -> Self {
        self.listen_address = listen_address;
        self
    }
    /// listen address.
    pub fn listen_address(&self) -> &A {
        &self.listen_address
    }
    /// Set transport codec type.
    pub fn with_transport_codec(mut self, transport_codec: Codec) -> Self {
        self.transport_codec = transport_codec;
        self
    }
    /// transport codec type.
    pub fn transport_codec(&self) -> Codec {
        self.transport_codec
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
