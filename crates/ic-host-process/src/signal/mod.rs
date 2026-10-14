//! Explicit process-global Unix signal actions, independent of cancellation policy.

use std::{fmt, io};

#[cfg(test)]
mod tests;

/// Failed installation retains its signal, native cause and rollback failure.
#[derive(Debug)]
pub struct SignalActionError {
    /// Signal whose admission or installation failed.
    pub signal: libc::c_int,
    /// Original input or native installation failure.
    pub source: io::Error,
    /// First failure restoring earlier actions after partial installation.
    pub rollback_error: Option<io::Error>,
}

impl fmt::Display for SignalActionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "signal {} action failed: {}",
            self.signal, self.source
        )
    }
}

impl std::error::Error for SignalActionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Saved dispositions for an explicitly selected set of Unix signals.
///
/// Actions are process-global. The caller must exclusively coordinate changes to
/// these signals for the guard's whole lifetime, restore overlapping guards in
/// reverse installation order, and supply an async-signal-safe static handler.
/// This guard chooses no signals, flags, cancellation, threads or shutdown policy.
/// Drop attempts restoration; call [`Self::restore`] to observe native failures.
pub struct SignalActions {
    previous: Vec<(libc::c_int, libc::sigaction)>,
}

impl SignalActions {
    /// Install the same caller-supplied handler for each selected signal.
    ///
    /// The handler uses an empty additional mask and no optional action flags.
    /// Empty selection has no effects. Duplicate signals refuse before native
    /// effects; failed installation attempts rollback of earlier changes.
    ///
    /// # Errors
    /// Returns the original installation failure separately from a rollback error.
    ///
    /// # Safety
    /// The caller must supply an async-signal-safe handler and exclusively
    /// coordinate process-global changes to the selected dispositions through
    /// restoration/Drop, including reverse-order restoration of overlapping guards.
    #[allow(unsafe_code)]
    pub unsafe fn install(
        signals: &[libc::c_int],
        handler: extern "C" fn(libc::c_int),
    ) -> Result<Self, SignalActionError> {
        for (index, &signal) in signals.iter().enumerate() {
            if signals[..index].contains(&signal) {
                return Err(SignalActionError {
                    signal,
                    source: io::Error::new(io::ErrorKind::InvalidInput, "duplicate signal"),
                    rollback_error: None,
                });
            }
        }
        let mut guard = Self {
            previous: Vec::new(),
        };
        for &signal in signals {
            // SAFETY: zero initializes these C structures, and both pointers
            // remain writable for the native call. The static callback has C ABI.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            action.sa_sigaction = handler as *const () as libc::sighandler_t;
            let mut previous = unsafe { std::mem::zeroed() };
            if unsafe { libc::sigemptyset(&raw mut action.sa_mask) } != 0
                || unsafe { libc::sigaction(signal, &raw const action, &raw mut previous) } != 0
            {
                let source = io::Error::last_os_error();
                return Err(SignalActionError {
                    signal,
                    source,
                    rollback_error: guard.restore().err(),
                });
            }
            guard.previous.push((signal, previous));
        }
        Ok(guard)
    }

    /// Restore all saved dispositions in reverse order, retaining failures for retry.
    ///
    /// Successful restoration is idempotent. Every saved action is attempted even
    /// if another fails; Drop makes one further best-effort attempt for failures.
    ///
    /// # Errors
    /// Returns the first native restoration failure.
    #[allow(unsafe_code)]
    pub fn restore(&mut self) -> io::Result<()> {
        let mut first_error = None;
        let previous = std::mem::take(&mut self.previous);
        for (signal, action) in previous.into_iter().rev() {
            // SAFETY: this is the disposition returned by installation; the
            // caller retains exclusive process-global registration custody.
            if unsafe { libc::sigaction(signal, &raw const action, std::ptr::null_mut()) } != 0 {
                first_error.get_or_insert_with(io::Error::last_os_error);
                self.previous.push((signal, action));
            }
        }
        self.previous.reverse();
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for SignalActions {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
