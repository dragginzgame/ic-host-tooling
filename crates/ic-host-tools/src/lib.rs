//! IC-specific interface facts and bounded ICP responses.
pub mod candid;
#[cfg(feature = "certificate")]
pub mod certificate;
#[cfg(feature = "encoding")]
pub mod encoding;
pub mod icp;
/// Bounded effective project facts; ICP retains configuration resolution.
#[cfg(feature = "icp-project")]
pub mod icp_project;
pub mod interface;
#[cfg(feature = "ledger-evidence")]
pub mod ledger_evidence;
pub mod response;
