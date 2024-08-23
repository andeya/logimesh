// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu config.

use crate::discover;
pub use crate::tokio_serde::formats;

/// A lrcall client stbu config.
#[non_exhaustive]
pub struct LRConfig<ServiceLookup> {
    /// service name, recommended format is `Product.System.Module`
    pub service_name: String,
    /// service lookup engine
    pub service_lookup: ServiceLookup,
    /// Transport serde codec
    pub transport_codec: TransportCodec,
    /// load balance type
    pub load_balance: LoadBalance,
    /// Whether to enable the retry call function.
    pub enable_retry: bool,
    /// The number of requests that can be in flight at once.
    /// `max_in_flight_requests` controls the size of the map used by the client
    /// for storing pending requests.
    /// Default is 1000.
    pub max_in_flight_requests: usize,
    /// The number of requests that can be buffered client-side before being sent.
    /// `pending_requests_buffer` controls the size of the channel clients use
    /// to communicate with the request dispatch task.
    /// Default is 100.
    pub pending_request_buffer: usize,
}

impl<ServiceLookup> LRConfig<ServiceLookup>
where
    ServiceLookup: discover::ServiceLookup,
{
    /// Create a rlcall's config
    pub fn new(service_name: String, service_lookup: ServiceLookup) -> Self {
        let conf = tarpc::client::Config::default();
        Self {
            service_name,
            service_lookup,
            transport_codec: Default::default(),
            load_balance: Default::default(),
            enable_retry: Default::default(),
            max_in_flight_requests: conf.max_in_flight_requests,
            pending_request_buffer: conf.pending_request_buffer,
        }
    }
    /// Set transport serde codec
    pub fn with_transport_codec(mut self, transport_codec: TransportCodec) -> Self {
        self.transport_codec = transport_codec;
        self
    }
    /// Set load balance type
    pub fn with_load_balance(mut self, load_balance: LoadBalance) -> Self {
        self.load_balance = load_balance;
        self
    }
    /// Whether to enable the retry call function.
    pub fn with_enable_retry(mut self, enable_retry: bool) -> Self {
        self.enable_retry = enable_retry;
        self
    }
    /// The number of requests that can be in flight at once.
    /// `max_in_flight_requests` controls the size of the map used by the client
    /// for storing pending requests.
    /// Default is 1000.
    pub fn with_max_in_flight_requests(mut self, max_in_flight_requests: usize) -> Self {
        self.max_in_flight_requests = max_in_flight_requests;
        self
    }
    /// The number of requests that can be buffered client-side before being sent.
    /// `pending_requests_buffer` controls the size of the channel clients use
    /// to communicate with the request dispatch task.
    /// Default is 100.
    pub fn with_pending_request_buffer(mut self, pending_request_buffer: usize) -> Self {
        self.pending_request_buffer = pending_request_buffer;
        self
    }
}

/// Load balance type
#[derive(Debug)]
pub enum LoadBalance {
    /// round-robin strategy
    RoundRobin,
    /// consistent hashing strategy
    ConsistentHash,
}

impl Default for LoadBalance {
    fn default() -> Self {
        LoadBalance::RoundRobin
    }
}

/// Transport serde codec
#[derive(Debug, Clone)]
pub enum TransportCodec {
    /// Bincode codec using [bincode](https://docs.rs/bincode) crate.
    Bincode,
    /// JSON codec using [serde_json](https://docs.rs/serde_json) crate.
    Json,
    /// MessagePack codec using [rmp-serde](https://docs.rs/rmp-serde) crate.
    #[cfg(feature = "serde-transport-messagepack")]
    MessagePack,
    /// CBOR codec using [serde_cbor](https://docs.rs/serde_cbor) crate.
    #[cfg(feature = "serde-transport-cbor")]
    Cbor,
}

impl Default for TransportCodec {
    fn default() -> Self {
        TransportCodec::Bincode
    }
}
