//! IC-specific Candid extraction and bounded ICP CLI response interpretation.
#[cfg(all(unix, feature = "candid-extraction"))]
pub mod candid;
#[cfg(feature = "ic-limits")]
pub mod install_limits;
pub mod response;
#[cfg(all(test, unix, feature = "candid-extraction"))]
mod test_support;
