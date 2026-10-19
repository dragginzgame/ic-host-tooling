//! Local host filesystem reads, durable publication and descriptor locks.
//!
//! Optional Unix APIs: `capability` selects held directory/file capabilities;
//! `temporary-file` selects scoped named scratch files. Defaults select neither.
pub mod cache_tag;
#[cfg(all(unix, feature = "capability"))]
pub mod capability;
pub mod directory;
pub mod durable;
#[cfg(unix)]
pub mod identity;
pub mod path;
pub mod read;
pub mod scratch;
#[cfg(test)]
mod test_support;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

pub mod owned_directory;
