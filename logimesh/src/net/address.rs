// Modifications Copyright Andeya Lee 2024
// Based on original source code from Volo Contributors licensed under MIT OR Apache-2.0
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Unified address

use std::fmt;
use std::hash::Hash;
use std::net::{AddrParseError, Ipv6Addr, SocketAddr};
#[cfg(target_os = "linux")]
use std::os::linux::net::SocketAddrExt;
#[cfg(target_family = "unix")]
use std::os::unix::net::SocketAddr as StdUnixSocketAddr;
use std::str::FromStr;

#[cfg(target_family = "unix")]
use tokio::net::unix::SocketAddr as TokioUnixSocketAddr;

use super::probe;

/// Unified address
#[derive(Clone, Debug)]
pub enum Address {
    /// IP address
    Ip(SocketAddr),
    /// Unix address
    #[cfg(target_family = "unix")]
    Unix(StdUnixSocketAddr),
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ip(self_ip), Self::Ip(other_ip)) => self_ip == other_ip,
            #[cfg(target_family = "unix")]
            (Self::Unix(self_uds), Self::Unix(other_uds)) => {
                match (self_uds.as_pathname(), other_uds.as_pathname()) {
                    (Some(self_pathname), Some(other_pathname)) => self_pathname == other_pathname,
                    (None, None) => {
                        // Both uds are unnamed, so they cannot be compared.
                        //
                        // We noticed that the `PartialEq`, `Eq` and `Hash` are only used for load
                        // balance, and load balace can only be used for TCP connection.  So we can
                        // treat the unnamed uds as the same.
                        true
                    },
                    // named and unnamed must be different
                    _ => false,
                }
            },
            #[cfg(target_family = "unix")]
            _ => false,
        }
    }
}

impl Eq for Address {}

impl Hash for Address {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Ip(ip) => {
                state.write_u8(0);
                Hash::hash(ip, state);
            },
            #[cfg(target_family = "unix")]
            Self::Unix(uds) => {
                #[cfg(target_os = "linux")]
                if let Some(abs_name) = uds.as_abstract_name() {
                    state.write_u8(1);
                    Hash::hash(abs_name, state);
                    return;
                }
                if let Some(pathname) = uds.as_pathname() {
                    state.write_u8(2);
                    Hash::hash(pathname, state);
                } else {
                    state.write_u8(3);
                }
            },
        }
    }
}

impl Address {
    /// Returns the favor address
    pub fn favor_dual_stack(self) -> Self {
        match self {
            Address::Ip(addr) => {
                if addr.ip().is_unspecified() && should_favor_ipv6() {
                    Address::Ip((Ipv6Addr::UNSPECIFIED, addr.port()).into())
                } else {
                    self
                }
            },
            #[cfg(target_family = "unix")]
            _ => self,
        }
    }
}

fn should_favor_ipv6() -> bool {
    let probed = probe::probe();
    !probed.ipv4 || probed.ipv4_mapped_ipv6
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Ip(addr) => write!(f, "{addr}"),
            #[cfg(target_family = "unix")]
            Address::Unix(addr) => {
                #[cfg(target_os = "linux")]
                if let Some(abs_name) = addr.as_abstract_name() {
                    return write!(f, "{}", abs_name.escape_ascii());
                }
                if let Some(pathname) = addr.as_pathname() {
                    write!(f, "{}", pathname.to_string_lossy())
                } else {
                    f.write_str("(unnamed)")
                }
            },
        }
    }
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Address::Ip(addr)
    }
}

#[cfg(target_family = "unix")]
impl From<StdUnixSocketAddr> for Address {
    fn from(value: StdUnixSocketAddr) -> Self {
        Address::Unix(value)
    }
}

#[cfg(target_family = "unix")]
impl From<TokioUnixSocketAddr> for Address {
    fn from(value: TokioUnixSocketAddr) -> Self {
        // SAFETY: `std::mem::transmute` can ensure both struct has the same size, so there is no
        // need for checking it.
        Address::Unix(unsafe { std::mem::transmute::<tokio::net::unix::SocketAddr, std::os::unix::net::SocketAddr>(value) })
    }
}

impl FromStr for Address {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr: Result<SocketAddr, AddrParseError> = s.parse();
        match addr {
            Ok(addr) => Ok(addr.into()),
            Err(e) => Err(e),
        }
    }
}
