mod support;
use candid::Principal;
use ic_build::artifacts::{
    ArtifactCacheMaintenance, ArtifactCachePrunePolicy, WasmBuildOutcome, WasmBuildSpec,
    build_wasm_canisters_cached, wasm_path, workspace_root_for,
};
use ic_testkit::{
    benchmark::{
        BenchmarkEventSource, BenchmarkParserConfig, pair_benchmark_spans,
        parse_benchmark_events_from_source,
    },
    ic_host_fs::read::read_file,
    pic::{InstallSpec, StandaloneCanisterFixture},
};
use std::fs;
use support::unique_temp_directory as unique_temp_dir;
const PERF_PROBE_PACKAGE: &str = "ic_testkit_perf_probe";

#[test]
fn perf_probe_canister_emits_parseable_benchmark_markers() {
    let workspace =
        workspace_root_for(env!("CARGO_MANIFEST_DIR")).expect("resolve Cargo workspace");
    if !workspace
        .join("crates/ic_testkit_perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping perf probe canister test: fixture canister is not packaged");
        return;
    }
    let target_dir = unique_temp_dir("ic-testkit-perf-probe-target");

    let spec = WasmBuildSpec::new(&workspace, &target_dir, &[PERF_PROBE_PACKAGE], "debug")
        .with_prune_policy_at_most_every(
            ArtifactCachePrunePolicy::new(),
            std::time::Duration::from_secs(60 * 60),
        );
    let first = build_wasm_canisters_cached(&spec).expect("first exact Wasm build");
    assert!(matches!(first, WasmBuildOutcome::Built(_)));
    assert!(first.record().timings().cargo_build().is_some());
    assert_cache_observability(&first);

    let reused = build_wasm_canisters_cached(&spec).expect("reuse exact Wasm build");
    assert!(matches!(reused, WasmBuildOutcome::Reused(_)));
    assert_eq!(first.record().fingerprint(), reused.record().fingerprint());
    assert!(reused.record().timings().cargo_build().is_none());
    assert!(
        reused.record().maintenance().is_none(),
        "scheduled maintenance should skip a second immediate cache hit"
    );
    assert!(reused.record().timings().cache_maintenance().is_some());

    let wasm_path = wasm_path(&target_dir, PERF_PROBE_PACKAGE, "debug");
    fs::write(&wasm_path, b"tampered").expect("overwrite cached Wasm artifact");
    let repaired = build_wasm_canisters_cached(&spec)
        .expect("artifact content mismatch should invalidate Wasm build");
    assert!(matches!(repaired, WasmBuildOutcome::Reused(_)));
    assert_ne!(
        fs::read(&wasm_path).expect("read repaired Wasm artifact"),
        b"tampered",
    );

    let changed_environment = spec
        .clone()
        .with_extra_env([("IC_TESTKIT_CACHE_PROBE", "changed")]);
    let rebuilt = build_wasm_canisters_cached(&changed_environment)
        .expect("declared environment should invalidate Wasm build");
    assert!(matches!(rebuilt, WasmBuildOutcome::Built(_)));
    assert_ne!(first.record().fingerprint(), rebuilt.record().fingerprint());
    assert_eq!(
        first.record().input_digest(),
        rebuilt.record().input_digest(),
        "declared environment changes the build fingerprint, not source content",
    );

    let restored = build_wasm_canisters_cached(&spec)
        .expect("original content-addressed artifact should remain reusable");
    assert!(matches!(restored, WasmBuildOutcome::Reused(_)));
    assert!(wasm_path.is_file());

    let wasm = read_file(&wasm_path, 128 * 1024 * 1024).expect("read bounded Wasm");
    let fixture =
        StandaloneCanisterFixture::install(support::pocket_ic(), InstallSpec::new(wasm, vec![], 0));
    let result: u64 = fixture
        .update_candid("benchmark_once", ())
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

    fs::remove_dir_all(target_dir).expect("clean temp target dir");
}

fn assert_cache_observability(outcome: &WasmBuildOutcome) {
    assert!(matches!(
        outcome.record().maintenance(),
        Some(ArtifactCacheMaintenance::Pruned(_))
    ));
    let timings = outcome.record().timings();
    let input = timings.input_resolution();
    assert!(input.total() >= input.tool_identity());
    assert!(input.total() >= input.cargo_metadata());
    assert!(input.total() >= input.input_discovery());
    assert!(input.total() >= input.content_hashing());
    assert!(timings.cache_maintenance().is_some());
}
