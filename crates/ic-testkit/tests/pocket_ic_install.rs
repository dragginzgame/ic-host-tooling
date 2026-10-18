mod support;

use ic_platform::Principal;
use ic_testkit::pic::{
    CanisterDiagnosticsRequest, CanisterInstallError, CanisterInstallExt, CanisterLogRenderLimits,
    InstallSpec, PocketIcDiagnosticsExt, StandaloneCanisterFixture,
};

#[test]
fn fallible_install_preserves_the_original_failure_for_caller_diagnostics() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let error = pocket_ic
        .try_create_and_install(InstallSpec {
            wasm: vec![0xde, 0xad],
            init_bytes: vec![],
            cycles: 0,
            install_sender: None,
            label: Some("invalid-wasm".into()),
        })
        .expect_err("invalid Wasm should fail installation");

    let CanisterInstallError::InstallCode {
        canister_id,
        label,
        source,
    } = &error
    else {
        panic!("invalid Wasm must fail at installation: {error}");
    };
    assert_eq!(label.as_deref(), Some("invalid-wasm"));
    assert!(
        source.message().contains("CanisterInvalidWasm"),
        "original PocketIC rejection should be preserved: {}",
        source.message()
    );
    let report = pocket_ic.collect_canister_diagnostics(CanisterDiagnosticsRequest {
        canister_id: *canister_id,
        status_sender: Principal::anonymous(),
        log_sender: Principal::anonymous(),
        log_limits: CanisterLogRenderLimits::default(),
    });
    assert!(
        report.status.is_ok(),
        "the failed install's canister remains inspectable"
    );
    assert!(source.message().contains("CanisterInvalidWasm"));
}

#[test]
fn failed_standalone_install_returns_the_caller_instance() {
    let Err(error) = StandaloneCanisterFixture::try_install(
        support::pocket_ic(),
        InstallSpec {
            wasm: vec![0xde, 0xad],
            init_bytes: vec![],
            cycles: 0,
            install_sender: None,
            label: Some("invalid-standalone-wasm".into()),
        },
    ) else {
        panic!("invalid Wasm should fail standalone installation");
    };

    let CanisterInstallError::InstallCode {
        canister_id, label, ..
    } = error.install_error()
    else {
        panic!("invalid Wasm must fail at installation: {error}");
    };
    assert_eq!(label.as_deref(), Some("invalid-standalone-wasm"));
    error
        .pocket_ic()
        .canister_status(*canister_id, None)
        .expect("caller-created instance should remain inspectable after install failure");
}
