use ic_platform::pocket_ic::PocketIc;
use ic_platform::principal::ExternalPrincipal as Principal;

use super::{PocketIcInstance, PocketIcOperationError};

/// Failed creation, funding, or installation with its original operation error.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum CanisterInstallError {
    /// Canister creation failed before an id was returned.
    CreateCanister {
        /// Optional caller-provided install label.
        label: Option<String>,
        /// Original upstream operation failure.
        source: PocketIcOperationError,
    },
    /// Cycle funding failed for the created canister.
    AddCycles {
        /// Created canister, retained for caller inspection.
        canister_id: Principal,
        /// Optional caller-provided install label.
        label: Option<String>,
        /// Original upstream operation failure.
        source: PocketIcOperationError,
    },
    /// Wasm installation failed for the created canister.
    InstallCode {
        /// Created canister, retained for caller inspection.
        canister_id: Principal,
        /// Optional caller-provided install label.
        label: Option<String>,
        /// Original upstream operation failure.
        source: PocketIcOperationError,
    },
}

/// Failed standalone install retaining the caller-created instance and optional server.
pub struct StandaloneCanisterInstallError {
    instance: Box<PocketIcInstance<PocketIc>>,
    install_error: CanisterInstallError,
}

impl std::fmt::Display for CanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (operation, canister_id, label, source) = match self {
            Self::CreateCanister { label, source } => ("create_canister", None, label, source),
            Self::AddCycles {
                canister_id,
                label,
                source,
            } => ("add_cycles", Some(canister_id), label, source),
            Self::InstallCode {
                canister_id,
                label,
                source,
            } => ("install_canister", Some(canister_id), label, source),
        };
        write!(f, "{operation} failed")?;
        if let Some(canister_id) = canister_id {
            write!(f, " for canister {canister_id}")?;
        }
        if let Some(label) = label {
            write!(f, " ({label})")?;
        }
        write!(f, ": {source}")
    }
}

impl std::error::Error for CanisterInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let (Self::CreateCanister { source, .. }
        | Self::AddCycles { source, .. }
        | Self::InstallCode { source, .. }) = self;
        Some(source)
    }
}

impl StandaloneCanisterInstallError {
    pub(super) fn new(
        instance: PocketIcInstance<PocketIc>,
        install_error: CanisterInstallError,
    ) -> Self {
        Self {
            instance: Box::new(instance),
            install_error,
        }
    }

    /// Borrow the caller-created instance retained after the failed install.
    #[must_use]
    pub fn pocket_ic(&self) -> &PocketIc {
        self.instance.instance()
    }

    /// Inspect the structured install failure.
    #[must_use]
    pub const fn install_error(&self) -> &CanisterInstallError {
        &self.install_error
    }

    /// Recover the instance/server ownership record and install failure.
    pub fn into_parts(self) -> (PocketIcInstance<PocketIc>, CanisterInstallError) {
        (*self.instance, self.install_error)
    }
}

impl std::fmt::Debug for StandaloneCanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StandaloneCanisterInstallError")
            .field("install_error", &self.install_error)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for StandaloneCanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.install_error.fmt(f)
    }
}

impl std::error::Error for StandaloneCanisterInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.install_error)
    }
}

#[cfg(test)]
mod tests {
    use ic_platform::principal::ExternalPrincipal as Principal;

    use super::CanisterInstallError;

    #[test]
    fn labeled_install_error_display_includes_label() {
        let err = CanisterInstallError::InstallCode {
            canister_id: Principal::anonymous(),
            label: Some("authority".into()),
            source: super::PocketIcOperationError::new("trap"),
        };
        assert!(err.to_string().contains("(authority): trap"));
    }
}
