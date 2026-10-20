//! One phase vocabulary for Wasm execution, progress and failure attribution.

/// Phase of a Wasm acquisition, including commands, coordination and recovery.
///
/// Progress and failures use the same phase identity. Command errors identify
/// the invoked Cargo/rustc operation; timing summaries may aggregate phases.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBuildPhase {
    /// Validating the caller's specification before effects.
    Specification,
    /// Waiting for the exact-cache lock or preparing its directory.
    ExactCacheCoordination,
    /// Reading the Cargo executable identity.
    CargoIdentity,
    /// Reading the Rust compiler identity.
    RustcIdentity,
    /// Resolving or decoding Cargo's package graph.
    CargoMetadata,
    /// Discovering selected source and configuration inputs.
    InputDiscovery,
    /// Hashing selected and conservative input contents.
    ContentHashing,
    /// Waiting for or preparing a shared incremental target.
    SharedTargetCoordination,
    /// Inspecting or clearing a shared incremental target.
    SharedTargetMaintenance,
    /// Compiling the selected Wasm packages.
    CargoBuild,
    /// Validating, copying, hashing or stamping exact artifacts.
    ArtifactPublication,
    /// Applying retention to immutable exact-cache entries.
    ExactCacheMaintenance,
    /// Removing an incomplete exact-cache entry after failure.
    Cleanup,
}

impl std::fmt::Display for WasmBuildPhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Specification => "specification",
            Self::ExactCacheCoordination => "exact cache coordination",
            Self::CargoIdentity => "Cargo identity",
            Self::RustcIdentity => "Rust compiler identity",
            Self::CargoMetadata => "cargo metadata",
            Self::InputDiscovery => "input discovery",
            Self::ContentHashing => "content hashing",
            Self::SharedTargetCoordination => "shared target coordination",
            Self::SharedTargetMaintenance => "shared target maintenance",
            Self::CargoBuild => "cargo build",
            Self::ArtifactPublication => "artifact publication",
            Self::ExactCacheMaintenance => "exact cache maintenance",
            Self::Cleanup => "cleanup",
        })
    }
}
