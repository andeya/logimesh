// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu config.

/// A client stbu config.
#[non_exhaustive]
pub struct Config<ServiceLookup> {
    /// service name, recommended format is `Product.System.Module`
    pub service_name: String,
    /// Transport serde codec
    pub transport_codec: TransportCodec,
    /// service lookup engine
    pub service_lookup: ServiceLookup,
    /// load balance type
    pub load_balance: LoadBalance,
    /// Whether to enable the retry call function.
    pub enable_retry: bool,
    /// The number of requests that can be in flight at once.
    /// `max_in_flight_requests` controls the size of the map used by the client
    /// for storing pending requests.
    pub max_in_flight_requests: usize,
    /// The number of requests that can be buffered client-side before being sent.
    /// `pending_requests_buffer` controls the size of the channel clients use
    /// to communicate with the request dispatch task.
    pub pending_request_buffer: usize,
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

pub use crate::tokio_serde::formats;

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
