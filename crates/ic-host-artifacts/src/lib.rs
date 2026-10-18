//! Bounded host artifact streams, raw identities, gzip and Wasm inspection.
pub mod artifact;
pub mod batch;
#[cfg(feature = "wasm")]
pub mod wasm;
