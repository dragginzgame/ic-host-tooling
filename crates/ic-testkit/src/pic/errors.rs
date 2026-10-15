use candid::Principal;
use pocket_ic::{PocketIc, RejectResponse};

use super::{PocketIcInstance, PocketIcOperationError};

/// Structured failure from a [`super::CandidCallExt`] operation.
///
/// Match the variant to inspect its required context and original failure details.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidCallError {
    /// Arguments could not be Candid-encoded.
    Encode {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Original codec failure text.
        message: String,
    },
    /// Successful response bytes could not be Candid-decoded.
    Decode {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Number of reply bytes presented to the decoder.
        bytes: usize,
        /// Original codec failure text.
        message: String,
    },
    /// PocketIC returned a structured canister rejection.
    CanisterReject {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Complete upstream rejection.
        response: Box<RejectResponse>,
    },
    /// The PocketIC instance became unreachable.
    Transport {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Original upstream failure text.
        message: String,
    },
}

/// Stable call metadata attached to a contextual call failure.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidCallContext {
    /// Underlying PocketIC operation, such as `update_call` or `query_call`.
    pub operation: &'static str,
    /// Target canister.
    pub canister_id: Principal,
    /// Effective message caller.
    pub caller: Principal,
    /// Canister method name.
    pub method: String,
}

/// Stage at which a canister creation and installation failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanisterInstallPhase {
    /// PocketIC could not create the canister.
    CreateCanister,
    /// PocketIC could not add the requested cycles.
    AddCycles,
    /// PocketIC could not install the Wasm module.
    InstallCode,
}

impl CanisterInstallPhase {
    /// Upstream operation attempted at this stage.
    #[must_use]
    pub const fn operation(self) -> &'static str {
        match self {
            Self::CreateCanister => "create_canister",
            Self::AddCycles => "add_cycles",
            Self::InstallCode => "install_canister",
        }
    }
}

/// Failed creation, funding, or installation with its original operation error.
#[derive(Debug, Eq, PartialEq)]
pub struct CanisterInstallError {
    phase: CanisterInstallPhase,
    canister_id: Option<Principal>,
    label: Option<String>,
    source: PocketIcOperationError,
}

/// Failed standalone install retaining the caller-created instance and optional server.
pub struct StandaloneCanisterInstallError {
    instance: Box<PocketIcInstance<PocketIc>>,
    install_error: CanisterInstallError,
}

impl CandidCallContext {
    /// Capture the stable call metadata attached to one call failure.
    #[must_use]
    pub fn new(
        operation: &'static str,
        canister_id: Principal,
        caller: Principal,
        method: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            canister_id,
            caller,
            method: method.into(),
        }
    }
}

impl std::fmt::Display for CandidCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let context = match self {
            Self::Encode { context, message } => {
                write!(f, "candid encode_args failed: {message}")?;
                context
            }
            Self::Decode {
                context,
                bytes,
                message,
            } => {
                write!(f, "candid decode failed (bytes={bytes}): {message}")?;
                context
            }
            Self::CanisterReject { context, response } => {
                write!(
                    f,
                    "pocket_ic {} was rejected: {response}",
                    context.operation
                )?;
                context
            }
            Self::Transport { context, message } => {
                write!(f, "pocket_ic {} failed: {message}", context.operation)?;
                context
            }
        };
        write!(
            f,
            " (operation={}, canister={}, caller={}, method={})",
            context.operation, context.canister_id, context.caller, context.method
        )
    }
}

impl std::error::Error for CandidCallError {}

impl CanisterInstallError {
    /// Capture an operation failure and all context available at that stage.
    #[must_use]
    pub const fn new(
        phase: CanisterInstallPhase,
        canister_id: Option<Principal>,
        label: Option<String>,
        source: PocketIcOperationError,
    ) -> Self {
        Self {
            phase,
            canister_id,
            label,
            source,
        }
    }

    /// Read the failed stage.
    #[must_use]
    pub const fn phase(&self) -> CanisterInstallPhase {
        self.phase
    }

    /// Read the created canister id, absent when creation failed.
    #[must_use]
    pub const fn canister_id(&self) -> Option<Principal> {
        self.canister_id
    }

    /// Read the original upstream panic message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.source.message()
    }

    /// Inspect the original operation failure and its transport classification.
    #[must_use]
    pub const fn operation_error(&self) -> &PocketIcOperationError {
        &self.source
    }

    /// Read the optional caller-provided install label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

impl std::fmt::Display for CanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} failed", self.phase.operation())?;
        if let Some(canister_id) = self.canister_id {
            write!(f, " for canister {canister_id}")?;
        }
        if let Some(label) = &self.label {
            write!(f, " ({label})")?;
        }
        write!(f, ": {}", self.source)
    }
}

impl std::error::Error for CanisterInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
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
        &self.instance.instance
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
    use candid::Principal;

    use super::CanisterInstallError;

    #[test]
    fn labeled_install_error_display_includes_label() {
        let err = CanisterInstallError::new(
            super::CanisterInstallPhase::InstallCode,
            Some(Principal::anonymous()),
            Some("authority".into()),
            super::PocketIcOperationError::new("trap"),
        );

        assert_eq!(err.label(), Some("authority"));
        assert!(err.to_string().contains("(authority): trap"));
    }
}
