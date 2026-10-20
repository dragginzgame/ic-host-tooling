use super::{BenchmarkRunReport, write_benchmark_report_dir};
use ic_host_fs::{directory::create_unique_directory, owned_directory::OwnedDirectory};
use ic_testkit::benchmark::{
    BenchmarkParserConfig, SuiteDerivation, aggregate_benchmark_spans,
    compare_benchmark_aggregates, pair_benchmark_spans, parse_benchmark_events,
};
use std::fs;

#[test]
fn comparison_csv_distinguishes_named_all_suite_from_all_suites() {
    let parse = parse_benchmark_events(
        "ICTK|work:start|0|0|0\nICTK|work:end|100|20|30\n",
        &BenchmarkParserConfig {
            suite_derivation: SuiteDerivation::Fixed("ALL".into()),
            ..BenchmarkParserConfig::default()
        },
    );
    let spans = pair_benchmark_spans(&parse.events);
    let aggregates = aggregate_benchmark_spans(&spans.spans).expect("aggregate benchmark spans");
    let comparison = compare_benchmark_aggregates(&aggregates, &aggregates, 1).unwrap();
    let root_directory = OwnedDirectory::new(
        create_unique_directory(
            &std::env::temp_dir(),
            "testkit-benchmark-report",
            0o700,
            None,
        )
        .unwrap(),
    );
    let root = root_directory.path();
    let report = BenchmarkRunReport {
        parse,
        spans,
        aggregates,
        comparison: Some(comparison),
    };
    write_benchmark_report_dir(&report, root).expect("write comparison");
    assert_eq!(
        fs::read_to_string(root.join("raw-events.csv"))
            .unwrap()
            .lines()
            .next()
            .unwrap(),
        "source_line,source,prefix,suite,label,span_label,kind,instructions,heap_bytes,memory_bytes"
    );
    assert_eq!(
        fs::read_to_string(root.join("spans.csv"))
            .unwrap()
            .lines()
            .next()
            .unwrap(),
        "suite,span_label,start_line,end_line,instructions_delta,heap_bytes_delta,memory_bytes_delta"
    );
    assert_eq!(
        fs::read_to_string(root.join("suite-aggregates.csv"))
            .unwrap()
            .lines()
            .next()
            .unwrap(),
        "suite,span_label,runs,instructions_total,instructions_mean,heap_bytes_total,heap_bytes_mean,memory_bytes_total,memory_bytes_mean"
    );
    let csv = fs::read_to_string(root.join("comparison.csv")).unwrap();
    assert_eq!(
        csv.lines().next().unwrap(),
        "scope,suite,span_label,current_runs,previous_runs,current_mean_scale,previous_mean_scale,current_instructions_mean,previous_instructions_mean,current_heap_bytes_mean,previous_heap_bytes_mean,current_memory_bytes_mean,previous_memory_bytes_mean"
    );
    assert!(csv.lines().any(|row| row.starts_with("suite,ALL,work,")));
    assert!(csv.lines().any(|row| row.starts_with("all,ALL,work,")));
}

#[test]
fn writes_csv_and_markdown_report_files() {
    let parse = parse_benchmark_events(
        "\
ICTK|app/myfunc/something:start|0|0|0
ICTK|app/myfunc/something:end|234_200_000|1|4_194_304
ICTK|app/missing:end|0|0|0
ICTK|app/open:start|0|0|0
ICTK|app/current-only:start|0|0|0
ICTK|app/current-only:end|0|0|0
ICTK|app/quoted,\"thing:start|0|0|0
ICTK|app/quoted,\"thing:end|1|2|3
ICTK|app/decreasing:start|10|0|0
ICTK|app/decreasing:end|9|0|0
"
        .replace('_', "")
        .as_str(),
        &BenchmarkParserConfig::default(),
    );
    let spans = pair_benchmark_spans(&parse.events);
    let aggregates = aggregate_benchmark_spans(&spans.spans).expect("aggregate benchmark spans");
    let previous_parse = parse_benchmark_events(
        "\
ICTK|app/myfunc/something:start|0|0|0
ICTK|app/myfunc/something:end|174776119|1|5447148
ICTK|app/previous-only:start|0|0|0
ICTK|app/previous-only:end|0|0|0
",
        &BenchmarkParserConfig::default(),
    );
    let previous_spans = pair_benchmark_spans(&previous_parse.events);
    let previous_aggregates =
        aggregate_benchmark_spans(&previous_spans.spans).expect("aggregate benchmark spans");
    let comparison = compare_benchmark_aggregates(&aggregates, &previous_aggregates, 1).unwrap();
    let report = BenchmarkRunReport {
        parse,
        spans,
        aggregates,
        comparison: Some(comparison),
    };
    let root_directory = OwnedDirectory::new(
        create_unique_directory(
            &std::env::temp_dir(),
            "testkit-benchmark-report",
            0o700,
            None,
        )
        .unwrap(),
    );
    let root = root_directory.path();

    write_benchmark_report_dir(&report, root).expect("write report");

    assert!(root.join("spans.csv").exists());
    assert!(root.join("suite-aggregates.csv").exists());
    assert!(root.join("all-aggregates.csv").exists());

    let raw_csv = fs::read_to_string(root.join("raw-events.csv")).unwrap();
    assert!(raw_csv.lines().any(|line| line == "7,unknown,ICTK,app,\"app/quoted,\"\"thing:start\",\"app/quoted,\"\"thing\",start,0,0,0"));

    assert_eq!(
        fs::read_to_string(root.join("unpaired-markers.csv")).unwrap(),
        "source_line,source,kind,suite,span_label,label\n3,unknown,end,app,app/missing,app/missing:end\n4,unknown,start,app,app/open,app/open:start\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("invalid-spans.csv")).unwrap(),
        "start_line,end_line,suite,span_label,reason\n9,10,app,app/decreasing,end counter is lower than start counter\n"
    );
    let comparison_csv = fs::read_to_string(root.join("comparison.csv")).expect("read comparison");
    assert!(comparison_csv.contains("app/myfunc/something"));
    assert!(comparison_csv.contains("234200000,174776119"));
    for (label, expected) in [
        (
            "app/current-only",
            ["1", "", "1", "", "0", "", "0", "", "0", ""],
        ),
        (
            "app/previous-only",
            ["", "1", "", "1", "", "0", "", "0", "", "0"],
        ),
    ] {
        let rows = comparison_csv
            .lines()
            .skip(1)
            .map(|line| line.split(',').collect::<Vec<_>>())
            .filter(|columns| columns[2] == label)
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 2, "suite and all-suites rows for {label}");
        for columns in rows {
            assert_eq!(columns[3..], expected);
        }
    }

    let summary = fs::read_to_string(root.join("bench-summary.md")).expect("read summary");
    assert!(summary.contains("| app/myfunc/something | 1 | 234200000 | 1 | 4194304 |"));

    fs::remove_dir_all(root).expect("clean temp dir");
}
