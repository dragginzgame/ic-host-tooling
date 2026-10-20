use std::{num::NonZeroUsize, time::Duration};

use ic_platform::pocket_ic::{ErrorCode, PocketIc, RejectResponse};
use ic_platform::principal::ExternalPrincipal as Principal;

use super::{CanisterInstallError, transport::capture_operation};

/// Inputs and diagnostic context for one generic canister installation.
pub struct InstallSpec {
    /// Wasm module installed into the newly created canister.
    pub wasm: Vec<u8>,
    /// Raw Candid or application-specific initialization bytes.
    pub init_bytes: Vec<u8>,
    /// Extra cycles added after PocketIC creates the canister and before install.
    pub cycles: u128,
    /// Optional sender for the management-canister install operation.
    pub install_sender: Option<Principal>,
    /// Optional human-readable context included in install errors.
    pub label: Option<String>,
}

/// Retry limits and simulated cooldown for install-code operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    /// Maximum attempts, including the initial call.
    pub max_attempts: NonZeroUsize,
    /// Simulated cooldown between retryable attempts.
    pub cooldown: Duration,
}

/// Generic canister installation and structured install-code retry policy.
///
/// The extension creates canisters using PocketIC defaults. `InstallSpec::cycles`
/// is an additional top-up, not the complete initial balance.
pub trait CanisterInstallExt {
    /// Create and install one canister from a reusable specification.
    ///
    /// Captures PocketIC panics during creation, cycle funding, and installation.
    /// The error identifies the stage and retains an id when creation succeeded.
    /// Callers choose whether to collect diagnostics; this performs no diagnostic
    /// requests or error logging.
    fn try_create_and_install(&self, spec: InstallSpec) -> Result<Principal, CanisterInstallError>;

    /// Advance simulated time once, then tick twice for an install-code cooldown.
    /// Zero duration skips time reads/writes while retaining both ticks.
    fn wait_out_install_code_rate_limit(&self, cooldown: Duration);

    /// Retry only while PocketIC reports install-code rate limiting.
    ///
    /// `RetryPolicy::max_attempts` includes the initial call. Before each
    /// retry, this advances simulated time by the configured cooldown and
    /// executes two PocketIC ticks. Other rejections are returned unchanged.
    fn retry_install_code<T, F>(&self, policy: RetryPolicy, op: F) -> Result<T, RejectResponse>
    where
        F: FnMut() -> Result<T, RejectResponse>;
}

impl CanisterInstallExt for PocketIc {
    /// Install one arbitrary wasm module from a generic install specification.
    fn try_create_and_install(&self, spec: InstallSpec) -> Result<Principal, CanisterInstallError> {
        let canister_id = capture_operation(|| self.create_canister()).map_err(|source| {
            CanisterInstallError::CreateCanister {
                label: spec.label.clone(),
                source,
            }
        })?;
        if spec.cycles > 0 {
            capture_operation(|| self.add_cycles(canister_id, spec.cycles)).map_err(|source| {
                CanisterInstallError::AddCycles {
                    canister_id,
                    label: spec.label.clone(),
                    source,
                }
            })?;
        }

        // Move the Wasm and init payload while retaining the label for failures.
        let install = capture_operation(|| {
            self.install_canister(canister_id, spec.wasm, spec.init_bytes, spec.install_sender);
        });
        if let Err(source) = install {
            return Err(CanisterInstallError::InstallCode {
                canister_id,
                label: spec.label,
                source,
            });
        }

        Ok(canister_id)
    }

    /// Wait out the PocketIC `install_code` cooldown window inside the same instance.
    fn wait_out_install_code_rate_limit(&self, cooldown: Duration) {
        super::time::advance_and_tick(self, cooldown, 2);
    }

    fn retry_install_code<T, F>(&self, policy: RetryPolicy, op: F) -> Result<T, RejectResponse>
    where
        F: FnMut() -> Result<T, RejectResponse>,
    {
        retry_install_code_with(policy, op, || {
            self.wait_out_install_code_rate_limit(policy.cooldown);
        })
    }
}

fn retry_install_code_with<T, F, W>(
    policy: RetryPolicy,
    mut op: F,
    mut wait_out_cooldown: W,
) -> Result<T, RejectResponse>
where
    F: FnMut() -> Result<T, RejectResponse>,
    W: FnMut(),
{
    let mut retries = 1..policy.max_attempts.get();
    loop {
        match op() {
            Err(err)
                if err.error_code == ErrorCode::CanisterInstallCodeRateLimited
                    && retries.next().is_some() =>
            {
                wait_out_cooldown();
            }
            result => return result,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        num::NonZeroUsize,
        time::Duration,
    };

    use ic_platform::pocket_ic::{ErrorCode, RejectCode, RejectResponse};

    use super::{RetryPolicy, retry_install_code_with};

    fn rejection(error_code: ErrorCode, message: &str) -> RejectResponse {
        RejectResponse {
            reject_code: RejectCode::SysTransient,
            reject_message: message.to_string(),
            error_code,
            certified: false,
        }
    }

    #[test]
    fn retry_policy_counts_the_first_attempt() {
        for max_attempts in [1, 3] {
            let attempts = Cell::new(0);
            let waits = Cell::new(0);
            let result = retry_install_code_with(
                RetryPolicy {
                    max_attempts: NonZeroUsize::new(max_attempts).unwrap(),
                    cooldown: Duration::from_secs(1),
                },
                || {
                    attempts.set(attempts.get() + 1);
                    Err::<(), _>(rejection(
                        ErrorCode::CanisterInstallCodeRateLimited,
                        &format!("rate limit on attempt {}", attempts.get()),
                    ))
                },
                || waits.set(waits.get() + 1),
            );

            assert_eq!(
                result,
                Err(rejection(
                    ErrorCode::CanisterInstallCodeRateLimited,
                    &format!("rate limit on attempt {max_attempts}"),
                ))
            );
            assert_eq!(attempts.get(), max_attempts);
            assert_eq!(waits.get(), max_attempts - 1);
        }
    }

    #[test]
    fn retry_policy_waits_only_between_retryable_attempts() {
        for success_attempt in [1, 3] {
            let events = RefCell::new(Vec::new());
            let attempts = Cell::new(0);
            let result = retry_install_code_with(
                RetryPolicy {
                    max_attempts: NonZeroUsize::new(3).unwrap(),
                    cooldown: Duration::from_secs(1),
                },
                || {
                    events.borrow_mut().push("attempt");
                    attempts.set(attempts.get() + 1);
                    if attempts.get() == success_attempt {
                        Ok(42)
                    } else {
                        Err(rejection(
                            ErrorCode::CanisterInstallCodeRateLimited,
                            "rate limit",
                        ))
                    }
                },
                || events.borrow_mut().push("wait"),
            );
            assert_eq!(result, Ok(42));
            assert_eq!(attempts.get(), success_attempt);
            assert_eq!(
                events.into_inner(),
                if success_attempt == 1 {
                    vec!["attempt"]
                } else {
                    vec!["attempt", "wait", "attempt", "wait", "attempt"]
                }
            );
        }
    }

    #[test]
    fn retry_policy_stops_on_non_rate_limit_failure() {
        let attempts = Cell::new(0);
        let not_retryable = rejection(ErrorCode::CanisterRejectedMessage, "not retryable");
        let result = retry_install_code_with(
            RetryPolicy {
                max_attempts: NonZeroUsize::new(3).unwrap(),
                cooldown: Duration::from_secs(1),
            },
            || {
                attempts.set(attempts.get() + 1);
                Err::<(), _>(not_retryable.clone())
            },
            || panic!("non-rate-limit failure must not wait"),
        );

        assert_eq!(result, Err(not_retryable));
        assert_eq!(attempts.get(), 1);
    }
}
