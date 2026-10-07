//! IC-specific Candid extraction and bounded ICP CLI response interpretation.
#[cfg(all(unix, feature = "candid-extraction"))]
pub mod candid;
pub mod response;
#[cfg(all(test, unix, feature = "candid-extraction"))]
mod test_support;
