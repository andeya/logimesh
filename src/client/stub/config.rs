// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! A client stbu config.

/// A client stbu config.
pub struct Config<ServiceLookup> {
    /// service lookup engine
    pub service_lookup: ServiceLookup,
    /// load balance type
    pub load_balance: LoadBalance,
    /// Whether to enable the retry call function.
    pub enable_retry: bool,
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
