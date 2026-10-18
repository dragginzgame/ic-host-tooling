//! IC-specific interface facts and bounded ICP responses.
#[cfg(all(feature = "agent", not(target_family = "wasm")))]
pub mod agent;
pub mod candid;
#[cfg(feature = "certificate")]
pub mod certificate;
pub mod icp;
pub mod interface;
pub mod response;
