// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Service load balance error

use crate::BoxError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadBalanceError {
    #[error("load balance retry reaches end")]
    Retry,
    #[error("load balance discovery error: {0:?}")]
    Discover(#[from] BoxError),
}

pub trait Retryable {
    fn retryable(&self) -> bool {
        false
    }
}
