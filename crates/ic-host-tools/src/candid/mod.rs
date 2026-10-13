//! Pure Candid text and service interpretation, with optional extractor execution.

#[cfg(all(unix, feature = "candid-extraction"))]
pub mod extraction;
#[cfg(feature = "candid-service")]
pub mod service;
pub mod text;
