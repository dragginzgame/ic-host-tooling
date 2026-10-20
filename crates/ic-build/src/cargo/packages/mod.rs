//! Qualification of exact coordinated Cargo archives in a fresh retained workspace.
//! Pure bounded external-lock subset admission is also available independently.
//!
//! Consumers supply package/version selection, tools, source custody and budgets.
//! This is local build evidence, never registry publication or release admission.
mod external_lock;
pub use external_lock::{ExternalLockError, verify_external_lock_subset};

#[cfg(unix)]
mod archive;
#[cfg(unix)]
mod source_inventory;
#[cfg(unix)]
pub use archive::{
    PackageArchiveSpec, PackageCommandError, PackageQualificationError, PackageQualificationLimits,
    qualify_package_archives,
};
