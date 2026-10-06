//! Local host filesystem reads, durable publication and descriptor locks.
pub mod durable;
pub mod read;
#[cfg(all(test, unix))]
mod test_support;
