use ic_build::{
    artifacts::{WasmBuildSpec, build_wasm_canisters_cached},
    cargo::{observation::workspace_root_for, selection::CargoSelection},
};
use ic_host_fs::read::read_file;
use ic_host_process::{
    child::{CleanupPolicy, OwnedChild},
    tool::{OutputLimit, OutputLimits},
};
use ic_platform::Principal;
use ic_testkit::{
    benchmark::{
        BenchmarkEventSource, BenchmarkParserConfig, pair_benchmark_spans,
        parse_benchmark_events_from_source,
    },
    pic::{InstallSpec, StandaloneCanisterFixture},
};
use ic_testkit_perf_probe::protocol::{
    self, BenchmarkOnce, BenchmarkTrap, InspectState, MutateState, Ping, SeedState, State,
};
use ic_wire::Method;
use std::process::Command;
const PERF_PROBE_PACKAGE: &str = "ic_testkit_perf_probe";

fn install_probe() -> StandaloneCanisterFixture {
    let workspace = workspace_root_for(
        env!("CARGO_MANIFEST_DIR"),
        OutputLimits {
            stdout: OutputLimit::Terminate(64 * 1024),
            stderr: OutputLimit::Terminate(64 * 1024),
            timeout: None,
        },
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| env!("CARGO").into())),
    )
    .expect("resolve Cargo workspace");
    assert!(
        workspace
            .join("crates/ic_testkit_perf_probe/Cargo.toml")
            .is_file(),
        "benchmark qualification requires the workspace perf probe"
    );
    let target_dir = tempfile::Builder::new()
        .prefix("ic-testkit-perf-probe-target")
        .tempdir_in(workspace.join("target"))
        .expect("create private test directory");

    let env = std::env::vars_os();
    let spec = WasmBuildSpec::new(&workspace, target_dir.path(), &[PERF_PROBE_PACKAGE], env)
        .with_cargo_selection(CargoSelection {
            locked: true,
            ..CargoSelection::default()
        })
        .with_shared_incremental_target(workspace.join("target/pic-wasm"));
    let outcome = build_wasm_canisters_cached(&spec, |command| {
        OwnedChild::spawn(command, CleanupPolicy::KillAndWait)
    })
    .expect("build perf probe Wasm");
    let wasm_path = outcome
        .record()
        .artifacts()
        .first()
        .expect("perf probe build returns its retained Wasm");
    let wasm = read_file(wasm_path, 128 * 1024 * 1024).expect("read retained bounded Wasm");
    StandaloneCanisterFixture::try_install(
        ic_testkit::pic::PocketIcStartupConfig::from_env(std::time::Duration::from_secs(30))
            .expect("configure prepared PocketIC")
            .build_instance(ic_testkit::pic::PocketIcBuilder::new().with_application_subnet())
            .expect("construct explicitly configured test instance"),
        InstallSpec {
            wasm,
            init_bytes: vec![],
            cycles: 0,
            install_sender: None,
            label: None,
        },
    )
    .expect("install standalone fixture")
}

#[test]
fn perf_probe_canister_emits_parseable_benchmark_markers() {
    let fixture = install_probe();
    let reply = fixture
        .pocket_ic()
        .update_call(
            fixture.canister_id(),
            ic_platform::Principal::anonymous(),
            BenchmarkOnce::NAME,
            ic_wire::encode_request::<BenchmarkOnce>(&())
                .unwrap()
                .into_bytes(),
        )
        .expect("benchmark_once update call");
    let result = ic_wire::decode_response::<BenchmarkOnce>(&reply).unwrap();

    assert_eq!(result, 1_498_500);

    let logs = fixture
        .pocket_ic()
        .fetch_canister_logs(fixture.canister_id(), Principal::anonymous())
        .expect("fetch perf probe logs");
    let log_text = logs
        .iter()
        .map(|record| String::from_utf8_lossy(&record.content))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed = parse_benchmark_events_from_source(
        &log_text,
        &BenchmarkParserConfig::default(),
        BenchmarkEventSource::FetchedLog,
    );
    let spans = pair_benchmark_spans(&parsed.events);

    assert!(
        parsed.malformed_markers.is_empty(),
        "unexpected malformed markers: {:?}",
        parsed.malformed_markers
    );
    assert!(
        spans.invalid_spans.is_empty(),
        "unexpected invalid spans: {:?}",
        spans.invalid_spans
    );
    assert!(
        spans.unpaired_markers.is_empty(),
        "unexpected unpaired markers: {:?}",
        spans.unpaired_markers
    );
    assert!(
        spans
            .spans
            .iter()
            .any(|span| span.span_label == "probe/benchmark_once" && span.delta.instructions > 0),
        "expected a positive instruction delta in spans: {:?}",
        spans.spans
    );

    assert_probe_contract(&fixture);
    assert_probe_trap(&fixture);
}

fn assert_probe_trap(fixture: &StandaloneCanisterFixture) {
    let error = fixture
        .pocket_ic()
        .update_call(
            fixture.canister_id(),
            Principal::anonymous(),
            BenchmarkTrap::NAME,
            ic_wire::encode_request::<BenchmarkTrap>(&())
                .unwrap()
                .into_bytes(),
        )
        .expect_err("the probe deliberately traps");
    assert!(error.reject_message.contains("intentional perf probe trap"));
    let reply = fixture
        .pocket_ic()
        .query_call(
            fixture.canister_id(),
            Principal::anonymous(),
            Ping::NAME,
            ic_wire::encode_request::<Ping>(&()).unwrap().into_bytes(),
        )
        .expect("the same canister remains callable after the trap");
    assert_eq!(
        ic_wire::decode_response::<Ping>(&reply).unwrap().as_str(),
        "pong"
    );
    assert!(ic_wire::decode_response::<InspectState>(&reply).is_err());
    let request = ic_wire::encode_request::<Ping>(&()).unwrap();
    assert!(ic_wire::decode_response::<Ping>(request.as_bytes()).is_err());
    for malformed in [
        reply[..reply.len() - 1].to_vec(),
        [reply.as_slice(), &[0]].concat(),
        vec![0; protocol::FRAME_BYTES + 1],
    ] {
        assert!(ic_wire::decode_response::<Ping>(&malformed).is_err());
    }
}

fn assert_probe_contract(fixture: &StandaloneCanisterFixture) {
    let pic = fixture.pocket_ic();
    let id = fixture.canister_id();
    let caller = Principal::anonymous();
    for size in [0, 3] {
        let reply = pic
            .update_call(
                id,
                caller,
                SeedState::NAME,
                ic_wire::encode_request::<SeedState>(&size)
                    .unwrap()
                    .into_bytes(),
            )
            .unwrap();
        ic_wire::decode_response::<SeedState>(&reply).unwrap();
        let expected = State {
            mutations: 0,
            bytes: u64::from(size),
            first: (size > 0).then_some(42),
        };
        let reply = pic
            .query_call(
                id,
                caller,
                InspectState::NAME,
                ic_wire::encode_request::<InspectState>(&())
                    .unwrap()
                    .into_bytes(),
            )
            .unwrap();
        assert_eq!(
            ic_wire::decode_response::<InspectState>(&reply).unwrap(),
            expected
        );
    }
    let reply = pic
        .update_call(
            id,
            caller,
            MutateState::NAME,
            ic_wire::encode_request::<MutateState>(&())
                .unwrap()
                .into_bytes(),
        )
        .unwrap();
    ic_wire::decode_response::<MutateState>(&reply).unwrap();
    let request = ic_wire::encode_request::<SeedState>(&99)
        .unwrap()
        .into_bytes();
    let bad = [
        vec![],
        request[..request.len() - 1].to_vec(),
        ic_wire::encode_request::<MutateState>(&())
            .unwrap()
            .into_bytes(),
        ic_wire::encode_response::<SeedState>(&())
            .unwrap()
            .into_bytes(),
        [request.as_slice(), &[0]].concat(),
        vec![0; protocol::FRAME_BYTES + 1],
    ];
    for bytes in bad {
        let oversized = bytes.len() > protocol::FRAME_BYTES;
        let error = pic
            .update_call(id, caller, SeedState::NAME, bytes)
            .expect_err("invalid request refuses before seeding");
        if oversized {
            assert!(
                error
                    .reject_message
                    .contains("perf probe Wire request exceeds limit")
            );
        }
        let reply = pic
            .query_call(
                id,
                caller,
                InspectState::NAME,
                ic_wire::encode_request::<InspectState>(&())
                    .unwrap()
                    .into_bytes(),
            )
            .unwrap();
        assert_eq!(
            ic_wire::decode_response::<InspectState>(&reply).unwrap(),
            State {
                mutations: 1,
                bytes: 3,
                first: Some(255)
            }
        );
    }
    pic.query_call(
        id,
        caller,
        InspectState::NAME,
        ic_wire::encode_request::<Ping>(&()).unwrap().into_bytes(),
    )
    .expect_err("query must check its method identity too");
}
