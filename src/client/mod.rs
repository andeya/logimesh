//! Provides a client that connects to a server and sends multiplexed requests.

pub mod stub;
pub use ::tarpc::client::{new, Channel, Config, NewClient, RequestDispatch, RpcError};
