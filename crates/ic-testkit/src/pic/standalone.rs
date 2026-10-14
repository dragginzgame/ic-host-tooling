use candid::Principal;
use pocket_ic::PocketIc;

use super::{CanisterInstallExt, InstallSpec, StandaloneCanisterInstallError};

/// One caller-built PocketIC instance and one installed canister.
///
/// This fixture owns the instance directly. It does not impose process-wide
/// synchronization or hide PocketIC methods behind a wrapper.
pub struct StandaloneCanisterFixture {
    pocket_ic: PocketIc,
    canister_id: Principal,
}

impl StandaloneCanisterFixture {
    /// Install one canister into a caller-configured PocketIC instance.
    ///
    /// # Panics
    ///
    /// Panics with the contextual install error when installation fails.
    #[must_use]
    pub fn install(pocket_ic: PocketIc, spec: InstallSpec) -> Self {
        Self::try_install(pocket_ic, spec)
            .unwrap_or_else(|err| panic!("failed to install standalone canister fixture: {err}"))
    }

    /// Fallible counterpart to [`install`](Self::install).
    ///
    /// On failure, the error retains both the caller's PocketIC instance and
    /// the structured install failure.
    pub fn try_install(
        pocket_ic: PocketIc,
        spec: InstallSpec,
    ) -> Result<Self, StandaloneCanisterInstallError> {
        let canister_id = match pocket_ic.try_create_and_install(spec) {
            Ok(canister_id) => canister_id,
            Err(error) => return Err(StandaloneCanisterInstallError::new(pocket_ic, error)),
        };

        Ok(Self {
            pocket_ic,
            canister_id,
        })
    }

    /// Borrow the PocketIC instance that owns this standalone fixture.
    #[must_use]
    pub const fn pocket_ic(&self) -> &PocketIc {
        &self.pocket_ic
    }

    /// Read the installed canister id for this standalone fixture.
    #[must_use]
    pub const fn canister_id(&self) -> Principal {
        self.canister_id
    }

    /// Consume the fixture and return the owned PocketIC instance and canister id.
    #[must_use]
    pub fn into_parts(self) -> (PocketIc, Principal) {
        (self.pocket_ic, self.canister_id)
    }
}
