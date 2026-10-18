//! Private performance probe and its shared, bounded Wire control protocol.

pub mod protocol;

#[cfg(target_arch = "wasm32")]
mod canister;
