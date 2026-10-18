//! IC-specific interface facts and bounded ICP responses.
pub mod candid;
#[cfg(feature = "certificate")]
pub mod certificate;
pub mod icp;
pub mod interface;
pub mod response;
