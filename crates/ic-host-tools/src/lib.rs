//! IC interface facts, Candid extraction, ICP protocol commands and bounded responses.
pub mod candid;
#[cfg(feature = "certificate")]
pub mod certificate;
pub mod icp;
#[cfg(feature = "ic-limits")]
pub mod install_limits;
#[cfg(feature = "ic-interface")]
pub mod interface;
pub mod response;
#[cfg(all(test, unix, feature = "candid-extraction"))]
mod test_support;
