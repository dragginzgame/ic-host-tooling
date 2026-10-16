mod support;

use candid::Principal;
use ic_testkit::pic::{
    CanisterDiagnosticsRequest, CanisterInstallError, CanisterInstallExt, InstallSpec,
    PocketIcDiagnosticsExt, StandaloneCanisterFixture,
};

#[test]
fn fallible_install_preserves_the_original_failure_for_caller_diagnostics() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let error = pocket_ic
        .try_create_and_install(InstallSpec::new(vec![0xde, 0xad], vec![], 0).label("invalid-wasm"))
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
    let report = pocket_ic.collect_canister_diagnostics(CanisterDiagnosticsRequest::new(
        *canister_id,
        Principal::anonymous(),
        Principal::anonymous(),
    ));
    assert!(
        report.status().is_ok(),
        "the failed install's canister remains inspectable"
    );
    assert!(source.message().contains("CanisterInvalidWasm"));
}

#[test]
fn failed_standalone_install_returns_the_caller_instance() {
    let Err(error) = StandaloneCanisterFixture::try_install(
        support::pocket_ic(),
        InstallSpec::new(vec![0xde, 0xad], vec![], 0).label("invalid-standalone-wasm"),
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
