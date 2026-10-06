//! Consumer-admitted executable resolution, bounded execution and Git observations.
#[cfg(unix)]
pub mod provenance;
#[cfg(all(test, unix))]
mod test_support;
#[cfg(unix)]
pub mod tool;
