# Example

Example service to demonstrate how to set up `logimesh`, run the following with `RUST_LOG=trace`.

## Server

```bash
cargo run --bin server -- --port 50051
```

## Client

```bash
cargo run --bin client -- --server-addr "[::1]:50051" --name "Bob"
```
