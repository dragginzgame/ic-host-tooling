//! IC-specific interface facts and bounded ICP responses.
pub mod candid;
#[cfg(feature = "certificate")]
pub mod certificate;
#[cfg(feature = "ic-limits")]
pub mod install_limits;
#[cfg(any(feature = "candid-service", feature = "ic-interface"))]
pub mod interface;
pub mod response;
