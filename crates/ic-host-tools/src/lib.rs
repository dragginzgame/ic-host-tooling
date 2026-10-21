//! IC-specific interface facts and bounded ICP responses.
#[cfg(all(feature = "agent", not(target_family = "wasm")))]
pub mod agent;
#[cfg(feature = "candid")]
pub mod candid;
#[cfg(feature = "certificate")]
pub mod certificate;
#[cfg(feature = "encoding")]
pub mod encoding;
pub mod icp;
/// Bounded ICP Ledger protobuf transfer fields; authentication stays with callers.
#[cfg(feature = "icp-ledger")]
pub mod icp_ledger;
/// Bounded effective project facts; ICP retains configuration resolution.
#[cfg(feature = "icp-project")]
pub mod icp_project;
pub mod interface;
#[cfg(feature = "ledger-evidence")]
pub mod ledger_evidence;
#[cfg(feature = "registry")]
pub mod registry;
pub mod response;
