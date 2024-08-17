#[cfg(feature = "serde1")]
pub use lrcall_plugins::derive_serde;
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// The main macro that creates RPC services.
///
/// Rpc methods are specified, mirroring trait syntax:
///
/// ```
/// #[lrcall::service]
/// trait Service {
/// /// Say hello
/// async fn hello(name: String) -> String;
/// }
/// ```
///
/// Attributes can be attached to each rpc. These attributes
/// will then be attached to the generated service traits'
/// corresponding `fn`s, as well as to the client stubs' RPCs.
///
/// The following items are expanded in the enclosing module:
///
/// * `trait Service` -- defines the RPC service.
///   * `fn serve` -- turns a service impl into a request handler.
/// * `Client` -- a client stub with a fn for each RPC.
///   * `fn new_stub` -- creates a new Client stub.
pub use lrcall_plugins::service;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}