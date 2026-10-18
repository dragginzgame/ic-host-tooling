//! Native process observations, consumer-admitted execution and Git provenance.
#[cfg(unix)]
pub mod child;
#[cfg(unix)]
pub mod custody;
pub mod framing;
#[cfg(unix)]
pub mod provenance;
pub mod sampling;
#[cfg(unix)]
pub mod signal;
pub mod status;
#[cfg(all(test, unix))]
mod test_support;
#[cfg(unix)]
pub mod tool;

#[cfg(any(test, feature = "test-support"))]
pub mod testing;
