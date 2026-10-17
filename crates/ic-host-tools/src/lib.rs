//! IC-specific interface facts and bounded ICP responses.
#[cfg(all(feature = "agent", not(target_arch = "wasm32")))]
pub mod agent;
pub mod candid;
#[cfg(feature = "certificate")]
pub mod certificate;
pub mod interface;
pub mod response;
