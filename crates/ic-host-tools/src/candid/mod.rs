//! Pure Candid text and service interpretation, with optional extractor execution.

#[cfg(all(unix, feature = "candid-extraction"))]
pub mod extraction;
#[cfg(feature = "ic-interface")]
pub mod service;
pub mod text;
