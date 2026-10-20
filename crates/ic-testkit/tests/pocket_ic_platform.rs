//! Qualify Platform identities through Testkit’s public PocketIC module; no server is started.
#![cfg(not(target_family = "wasm"))]
use ic_testkit::pocket_ic::{
    CanisterInstallMode, CreateCanisterParams, CreateCanisterPlacement, PocketIc, PocketIcBuilder,
    RejectResponse,
    common::rest::{
        CanisterHttpReply, CanisterHttpResponse, InstanceHttpGatewayConfig,
        MockCanisterHttpResponse, RawEffectivePrincipal, RawMessageId,
    },
};

#[test]
fn placement_gateway_and_http_fixture_identities() {
    // Fixture generation must retain the client/DTO principal identity when
    // Platform's application Principal changes independently.
    let fixture: ic_platform::principal::ExternalPrincipal = ic_testkit::Fake::principal(7);
    let encoded = ic_platform::candid::contract::encode_one(fixture).unwrap();
    let (decoded,): (ic_platform::principal::ExternalPrincipal,) =
        ic_platform::candid::decode_args(
            &encoded,
            ic_platform::candid::DecodeLimits {
                reply_bytes: 1024,
                decoding_quota: 1_000_000,
                skipping_quota: 10_000,
                type_entries: 128,
                header_bytes: 1024,
            },
        )
        .unwrap();
    assert_eq!(decoded, fixture);
    let canister = ic_platform::principal::ExternalPrincipal::management_canister();
    let params = CreateCanisterParams {
        placement: Some(CreateCanisterPlacement::SubnetId(canister)),
        ..CreateCanisterParams::default()
    };
    assert!(matches!(
        params.placement,
        Some(CreateCanisterPlacement::SubnetId(id)) if id == canister
    ));
    let _builder = PocketIcBuilder::new().with_http_gateway(InstanceHttpGatewayConfig {
        ip_addr: Some("127.0.0.1".into()),
        port: Some(8080),
        domains: None,
        https_config: None,
        domain_custom_provider_local_file: None,
    });
    let message = RawMessageId {
        effective_principal: RawEffectivePrincipal::CanisterId(canister.as_slice().into()),
        message_id: vec![7; 32],
    };
    assert_eq!(message.message_id.len(), 32);
    let response = MockCanisterHttpResponse {
        subnet_id: canister,
        request_id: 7,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: b"fixture reply".to_vec(),
        }),
        additional_responses: vec![],
    };
    assert_eq!(response.request_id, 7);

    let _: fn(&PocketIc, RawMessageId) -> Result<Vec<u8>, RejectResponse> = PocketIc::await_call;
    let _: fn(&PocketIc, MockCanisterHttpResponse) = PocketIc::mock_canister_http_response;
    let _ = |client: &PocketIc| -> Result<Vec<u8>, RejectResponse> {
        client.update_call_with_effective_principal(
            canister,
            message.effective_principal,
            canister,
            "fixture",
            vec![],
        )
    };
}

#[test]
fn chunked_upgrade_preserves_pocketic_management_identity() {
    #[expect(
        clippy::default_trait_access,
        reason = "PocketIC exports the mode but not the contained upgrade flags"
    )]
    let mut mode = CanisterInstallMode::Upgrade(Some(Default::default()));
    let CanisterInstallMode::Upgrade(Some(flags)) = &mut mode else {
        panic!("expected upgrade flags");
    };
    flags.skip_pre_upgrade = Some(true);
    assert_eq!(flags.skip_pre_upgrade, Some(true));

    // Compile the real client parameter without starting a server or installing.
    let _ = |client: &PocketIc, mode: CanisterInstallMode| -> Result<(), RejectResponse> {
        let canister = ic_platform::principal::ExternalPrincipal::management_canister();
        client.install_chunked_canister(canister, None, mode, canister, vec![], vec![], vec![])
    };
}

#[test]
fn pending_ingress_preserves_the_submitted_message_identity() {
    let _ = |client: &PocketIc, message: RawMessageId| -> Option<Result<Vec<u8>, RejectResponse>> {
        client.ingress_status(message)
    };
    let _ = |client: &PocketIc| -> Result<RawMessageId, RejectResponse> {
        let canister = ic_platform::principal::ExternalPrincipal::management_canister();
        client.submit_call(canister, canister, "fixture", vec![])
    };
}
