//! Local host filesystem reads, durable publication and descriptor locks.
pub mod cache_tag;
pub mod durable;
pub mod path;
pub mod read;
pub mod scratch;
#[cfg(test)]
mod test_support;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

pub mod owned_directory;
