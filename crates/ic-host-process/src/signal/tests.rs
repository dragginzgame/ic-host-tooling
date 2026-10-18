#![allow(unsafe_code)]

use super::SignalActions;
use std::{
    process::Command,
    sync::atomic::{AtomicI32, Ordering},
};

static OBSERVED: AtomicI32 = AtomicI32::new(0);

extern "C" fn observe(signal: libc::c_int) {
    OBSERVED.store(signal, Ordering::Relaxed);
}

extern "C" fn nested(_signal: libc::c_int) {
    OBSERVED.store(-1, Ordering::Relaxed);
}

fn disposition(signal: libc::c_int) -> libc::sighandler_t {
    // SAFETY: the output is writable and no disposition is modified.
    let mut action = unsafe { std::mem::zeroed() };
    assert_eq!(
        unsafe { libc::sigaction(signal, std::ptr::null(), &raw mut action) },
        0
    );
    action.sa_sigaction
}

fn raise(signal: libc::c_int) {
    // SAFETY: the worker owns these dispositions, and the handlers only store
    // to a static lock-free atomic. Delivery to this thread finishes before return.
    assert_eq!(unsafe { libc::raise(signal) }, 0);
}

#[test]
fn selected_actions_restore_and_failed_installation_rolls_back() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "signal::tests::dispositions_worker",
            "--nocapture",
        ])
        .env("IC_HOST_SIGNAL_WORKER", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn dispositions_worker() {
    if std::env::var_os("IC_HOST_SIGNAL_WORKER").is_none() {
        return;
    }
    // SAFETY: this isolated worker owns the dispositions; both static handlers
    // use only a lock-free atomic, and overlapping guards restore in LIFO order.
    let before = [disposition(libc::SIGUSR1), disposition(libc::SIGUSR2)];
    unsafe { SignalActions::install(&[], observe) }
        .unwrap()
        .restore()
        .unwrap();
    let mut outer =
        unsafe { SignalActions::install(&[libc::SIGUSR1, libc::SIGUSR2], observe) }.unwrap();
    for signal in [libc::SIGUSR1, libc::SIGUSR2] {
        raise(signal);
        assert_eq!(OBSERVED.load(Ordering::Relaxed), signal);
    }
    let duplicate = unsafe { SignalActions::install(&[libc::SIGUSR1, libc::SIGUSR1], nested) }
        .err()
        .unwrap();
    assert_eq!(duplicate.source.kind(), std::io::ErrorKind::InvalidInput);
    assert!(duplicate.rollback_error.is_none());
    raise(libc::SIGUSR1);
    assert_eq!(OBSERVED.load(Ordering::Relaxed), libc::SIGUSR1);
    {
        let _inner = unsafe { SignalActions::install(&[libc::SIGUSR1], nested) }.unwrap();
        raise(libc::SIGUSR1);
        assert_eq!(OBSERVED.load(Ordering::Relaxed), -1);
    }
    raise(libc::SIGUSR1);
    assert_eq!(OBSERVED.load(Ordering::Relaxed), libc::SIGUSR1);
    outer.restore().unwrap();
    outer.restore().unwrap();
    drop(outer);
    assert_eq!(
        [disposition(libc::SIGUSR1), disposition(libc::SIGUSR2)],
        before
    );
    for invalid in [libc::SIGKILL, -1] {
        let error = unsafe { SignalActions::install(&[libc::SIGUSR1, invalid], observe) }
            .err()
            .unwrap();
        assert_eq!(error.signal, invalid);
        assert!(error.rollback_error.is_none());
        assert_eq!(disposition(libc::SIGUSR1), before[0]);
    }
}
