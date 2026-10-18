//! Shared host-side Cargo/Wasm build and retained artifact-cache mechanics.
//!
//! Callers select recipes, tools, output budgets, source custody and maintenance.
//! The engine never owns a `PocketIC` lifecycle or a consumer's release identity.
//! Retain returned records through artifact consumption; paths alone carry no
//! cache-entry ownership. Builds have no default elapsed-time deadline.

pub mod artifacts;
pub mod cargo;

pub mod batch;

#[cfg(test)]
#[path = "../test-support/cargo/mod.rs"]
pub(crate) mod cargo_fixture;

#[cfg(test)]
#[path = "../test-support/fs/mod.rs"]
pub(crate) mod fs_fixture;
