use ic_build::{
    artifacts::{WasmBuildSpec, build_wasm_canisters_cached},
    cargo::{observation::workspace_root_for, selection::CargoSelection},
};
use ic_host_fs::read::read_file;
use ic_host_process::{
    child::{CleanupPolicy, OwnedChild},
    tool::{OutputLimit, OutputLimits},
};
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::{
    benchmark::{
        BenchmarkEventSource, BenchmarkParserConfig, pair_benchmark_spans,
        parse_benchmark_events_from_source,
    },
    pic::{
        InstallSpec, PocketIcCallFailure, StandaloneCanisterFixture, WireCallError, query_wire,
        update_wire,
    },
};
use ic_testkit_perf_probe::protocol::{
    self, BenchmarkOnce, BenchmarkTrap, InspectState, MutateState, Ping, QueryCaller, SeedState,
    State, UpdateCaller,
};
use ic_wire::Method;
use std::process::Command;
const PERF_PROBE_PACKAGE: &str = "ic-testkit-perf-probe";

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
            .join("crates/ic-testkit-perf-probe/Cargo.toml")
            .is_file(),
        "benchmark qualification requires the workspace perf probe"
    );
    let target_dir = ic_host_fs::owned_directory::OwnedDirectory::new(
        ic_host_fs::directory::create_unique_directory(
            &workspace.join("target"),
            "ic-testkit-perf-probe-target",
            0o700,
            None,
        )
        .expect("create private test directory"),
    );

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
    let result = update_wire::<BenchmarkOnce>(
        fixture.pocket_ic(),
        fixture.canister_id(),
        Principal::anonymous(),
        &(),
    )
    .expect("benchmark_once update call");

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
    assert_callers(&fixture);
}

fn assert_probe_trap(fixture: &StandaloneCanisterFixture) {
    let error = update_wire::<BenchmarkTrap>(
        fixture.pocket_ic(),
        fixture.canister_id(),
        Principal::anonymous(),
        &(),
    )
    .expect_err("the probe deliberately traps");
    let WireCallError::Call { source, .. } = error else {
        panic!("structured rejection required")
    };
    assert!(
        matches!(*source, PocketIcCallFailure::Rejected(response) if response.reject_message.contains("intentional perf probe trap"))
    );
    assert_eq!(
        query_wire::<Ping>(
            fixture.pocket_ic(),
            fixture.canister_id(),
            Principal::anonymous(),
            &()
        )
        .unwrap()
        .as_str(),
        "pong"
    );
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
        update_wire::<SeedState>(pic, id, caller, &size).unwrap();
        let expected = State {
            mutations: 0,
            bytes: u64::from(size),
            first: (size > 0).then_some(42),
        };
        assert_eq!(
            query_wire::<InspectState>(pic, id, caller, &()).unwrap(),
            expected
        );
    }
    update_wire::<MutateState>(pic, id, caller, &()).unwrap();
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
        assert_eq!(
            query_wire::<InspectState>(pic, id, caller, &()).unwrap(),
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

fn assert_callers(fixture: &StandaloneCanisterFixture) {
    for caller in [Principal::anonymous(), Principal::from_slice(&[7, 3])] {
        let query =
            query_wire::<QueryCaller>(fixture.pocket_ic(), fixture.canister_id(), caller, &())
                .unwrap();
        assert_eq!(query.as_slice(), caller.as_slice());
        let update =
            update_wire::<UpdateCaller>(fixture.pocket_ic(), fixture.canister_id(), caller, &())
                .unwrap();
        assert_eq!(update.as_slice(), caller.as_slice());
    }
}
