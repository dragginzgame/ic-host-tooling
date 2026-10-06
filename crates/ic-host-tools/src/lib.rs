//! IC-specific Candid extraction and bounded ICP CLI response interpretation.
#[cfg(unix)]
pub mod candid;
pub mod response;
#[cfg(all(test, unix))]
mod test_support;
