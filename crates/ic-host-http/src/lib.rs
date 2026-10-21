//! Bounded native HTTP response reading, without request or status policy.
//!
//! Callers create and send requests, select redirects/retries/timeouts and TLS,
//! admit status before or after reading, and own decoding, redaction and storage.
//! The limit applies to bytes yielded by reqwest, after any enabled decompression.
//! It does not bound reqwest's own chunk/decompression allocations or wire bytes.
//! This crate does not enable TLS or compression; consumers select those features.

#[cfg(not(target_family = "wasm"))]
mod response;
#[cfg(not(target_family = "wasm"))]
pub use response::{Body, ReadError, ReadFailure, ReadOptions, read_body, read_body_into};
