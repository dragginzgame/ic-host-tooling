//! Integration checks for benchmark behavior.
mod support;

use ic_testkit::benchmark::{
    BenchmarkCounter, BenchmarkCounters, BenchmarkEventKind, BenchmarkEventSource,
    BenchmarkParseReport, BenchmarkParserConfig, BenchmarkScope, DEFAULT_PREFIX, SuiteDerivation,
    aggregate_benchmark_spans, compare_benchmark_aggregates, format_marker, pair_benchmark_spans,
    parse_benchmark_events, parse_benchmark_events_from_captured_output,
    parse_benchmark_events_from_source,
};
use std::fmt::Write as _;

#[test]
fn parses_compact_markers_and_ignores_non_marker_lines() {
    let input = "\
ordinary debug line
ICTK|app/myfunc/something:start|100|200|300
ICTK|app/myfunc/something:end|150|260|390
";

    let report = parse_benchmark_events(input, &BenchmarkParserConfig::default());

    assert_eq!(report.ignored_line_count, 1);
    assert_eq!(
        report.malformed_markers,
        [] as [ic_testkit::benchmark::MalformedBenchmarkMarker; 0]
    );
    assert_eq!(report.events.len(), 2);
    assert_eq!(report.events[0].suite, "app");
    assert_eq!(report.events[0].span_label, "app/myfunc/something");
    assert_eq!(report.events[0].kind, BenchmarkEventKind::Start);
    assert_eq!(report.events[1].kind, BenchmarkEventKind::End);
}

#[test]
fn marker_formatter_uses_compact_tuple_shape() {
    let line = format_marker(
        DEFAULT_PREFIX,
        "app/myfunc/something:start",
        BenchmarkCounters {
            instructions: 1,
            heap_bytes: 2,
            memory_bytes: 3,
        },
    );

    assert_eq!(line, "ICTK|app/myfunc/something:start|1|2|3");
}

#[test]
fn records_source_stream_for_stdout_and_stderr_capture() {
    let stdout = parse_benchmark_events_from_source(
        "ICTK|app/a:start|1|2|3",
        &BenchmarkParserConfig::default(),
        BenchmarkEventSource::Stdout,
    );
    let stderr = parse_benchmark_events_from_source(
        "ICTK|app/a:end|2|3|4",
        &BenchmarkParserConfig::default(),
        BenchmarkEventSource::Stderr,
    );

    assert_eq!(stdout.events[0].source, BenchmarkEventSource::Stdout);
    assert_eq!(stderr.events[0].source, BenchmarkEventSource::Stderr);
}

#[test]
fn parses_combined_captured_stdout_and_stderr() {
    let bad_number = "ICTK|app/b:start|bad|2|3";
    let bad_shape = "ICTK|app/b:end|5|6";
    let stdout = format!("noise\n\nICTK|app/a:start|1|2|3\n{bad_number}\n");
    let stderr = format!("\nICTK|app/a:end|5|6|7\n{bad_shape}\nstderr noise\n");
    for strict in [false, true] {
        let config = BenchmarkParserConfig {
            strict,
            ..BenchmarkParserConfig::default()
        };
        let report = parse_benchmark_events_from_captured_output(&stdout, &stderr, &config);

        assert_eq!(report.ignored_line_count, if strict { 2 } else { 4 });
        assert_eq!(report.events.len(), 2);
        assert_eq!(report.events[0].source, BenchmarkEventSource::Stdout);
        assert_eq!(report.events[0].source_line, 3);
        assert_eq!(report.events[1].source, BenchmarkEventSource::Stderr);
        assert_eq!(report.events[1].source_line, 2);
        let expected = if strict {
            vec![
                (BenchmarkEventSource::Stdout, 1, "noise"),
                (BenchmarkEventSource::Stdout, 4, bad_number),
                (BenchmarkEventSource::Stderr, 3, bad_shape),
                (BenchmarkEventSource::Stderr, 4, "stderr noise"),
            ]
        } else {
            vec![
                (BenchmarkEventSource::Stdout, 4, bad_number),
                (BenchmarkEventSource::Stderr, 3, bad_shape),
            ]
        };
        assert_eq!(
            report
                .malformed_markers
                .iter()
                .map(|marker| (marker.source, marker.source_line, marker.line.as_str()))
                .collect::<Vec<_>>(),
            expected,
        );
    }
    assert_eq!(
        parse_benchmark_events_from_captured_output("", "", &BenchmarkParserConfig::default()),
        BenchmarkParseReport::default(),
    );
}

#[test]
fn malformed_markers_capture_bad_shape_and_bad_numbers() {
    let input = "\
ICTK|app/a:start|1|2
ICTK|app/a:end|1|two|3
ICTK||1|2|3
";

    let report = parse_benchmark_events(input, &BenchmarkParserConfig::default());

    assert_eq!(
        report.events,
        [] as [ic_testkit::benchmark::RawBenchmarkEvent; 0]
    );
    assert_eq!(report.malformed_markers.len(), 3);
    assert!(
        report
            .malformed_markers
            .iter()
            .any(|marker| marker.reason.contains("five"))
    );
    assert!(
        report
            .malformed_markers
            .iter()
            .any(|marker| marker.reason.contains("unsigned integer"))
    );
    assert!(
        report
            .malformed_markers
            .iter()
            .any(|marker| marker.reason.contains("label is empty"))
    );
}

#[test]
fn malformed_extra_columns_preserve_diagnostics_and_following_markers() {
    let valid = "ICTK|app/a:start|1|2|3";
    let malformed = [
        format!("{valid}|"),
        format!("{valid}|extra|tail"),
        format!("{valid}{}", "|".repeat(65_536)),
    ];
    let input = format!("{}\n{valid}\n", malformed.join("\n"));
    let report = parse_benchmark_events_from_source(
        &input,
        &BenchmarkParserConfig::default(),
        BenchmarkEventSource::FetchedLog,
    );
    assert_eq!(report.malformed_markers.len(), malformed.len());
    for (index, (marker, line)) in report.malformed_markers.iter().zip(&malformed).enumerate() {
        assert_eq!(marker.source_line, index + 1);
        assert_eq!(marker.source, BenchmarkEventSource::FetchedLog);
        assert_eq!(&marker.line, line);
        assert_eq!(marker.reason, "expected five pipe-separated columns");
    }
    assert_eq!(report.events.len(), 1);
    assert_eq!(report.events[0].source_line, 4);
    assert_eq!(report.events[0].source, BenchmarkEventSource::FetchedLog);
    assert_eq!(
        report.events[0].counters,
        BenchmarkCounters {
            instructions: 1,
            heap_bytes: 2,
            memory_bytes: 3,
        },
    );
}

#[test]
fn strict_parser_reports_non_marker_log_lines() {
    let config = BenchmarkParserConfig {
        strict: true,
        ..BenchmarkParserConfig::default()
    };
    let report = parse_benchmark_events(
        "\
ordinary debug line
ICTK|app/a:start|1|2|3
",
        &config,
    );

    assert_eq!(report.events.len(), 1);
    assert_eq!(report.ignored_line_count, 0);
    assert_eq!(report.malformed_markers.len(), 1);
    assert_eq!(
        report.malformed_markers[0].reason,
        "line does not use a configured marker prefix"
    );
}

#[test]
fn pairs_repeated_same_label_spans_by_stack_order() {
    let report = parse_benchmark_events(
        "\
ICTK|app/a:start|10|10|10
ICTK|app/a:start|20|20|20
ICTK|app/a:end|30|30|30
ICTK|app/a:end|50|50|50
",
        &BenchmarkParserConfig::default(),
    );

    let spans = pair_benchmark_spans(&report.events);

    assert_eq!(spans.spans.len(), 2);
    assert_eq!(
        spans.unpaired_markers,
        [] as [ic_testkit::benchmark::RawBenchmarkEvent; 0]
    );
    assert_eq!(spans.spans[0].start_line, 2);
    assert_eq!(spans.spans[0].end_line, 3);
    assert_eq!(spans.spans[0].delta.instructions, 10);
    assert_eq!(spans.spans[1].start_line, 1);
    assert_eq!(spans.spans[1].end_line, 4);
    assert_eq!(spans.spans[1].delta.instructions, 40);
}

#[test]
fn reports_unpaired_and_invalid_spans() {
    let report = parse_benchmark_events(
        "\
ICTK|app/a:end|10|10|10
ICTK|app/b:start|20|20|20
ICTK|app/c:start|50|50|50
ICTK|app/c:end|40|60|60
",
        &BenchmarkParserConfig::default(),
    );

    let spans = pair_benchmark_spans(&report.events);

    assert_eq!(spans.spans, [] as [ic_testkit::benchmark::BenchmarkSpan; 0]);
    assert_eq!(spans.unpaired_markers.len(), 2);
    assert_eq!(spans.invalid_spans.len(), 1);
    assert_eq!(spans.invalid_spans[0].start.counters.instructions, 50);
    assert_eq!(spans.invalid_spans[0].end.counters.instructions, 40);
}

#[test]
fn pairing_preserves_suite_boundaries_and_owned_diagnostics() {
    let report = {
        let mut parsed = parse_benchmark_events(
            "ICTK|work:start|10|0|0\n\
             ICTK|work:end|20|0|0\n\
             ICTK|work:end|30|0|0\n\
             ICTK|work:start|50|0|0\n\
             ICTK|work:end|40|0|0\n\
             ICTK|work:start|70|0|0\n\
             ICTK|work:start|80|0|0\n",
            &BenchmarkParserConfig::default(),
        );
        for (event, suite) in parsed
            .events
            .iter_mut()
            .zip(["b", "a", "b", "a", "a", "b", "a"])
        {
            event.suite = suite.into();
        }
        pair_benchmark_spans(&parsed.events)
    };

    assert_eq!(report.spans.len(), 1);
    assert_eq!(report.spans[0].suite, "b");
    assert_eq!(report.spans[0].start_line, 1);
    assert_eq!(report.spans[0].end_line, 3);
    assert_eq!(report.spans[0].delta.instructions, 20);
    assert_eq!(report.invalid_spans.len(), 1);
    assert_eq!(report.invalid_spans[0].start.source_line, 4);
    assert_eq!(report.invalid_spans[0].end.source_line, 5);
    assert_eq!(
        report
            .unpaired_markers
            .iter()
            .map(|marker| (marker.suite.as_str(), marker.source_line, marker.kind))
            .collect::<Vec<_>>(),
        [
            ("a", 2, BenchmarkEventKind::End),
            ("a", 7, BenchmarkEventKind::Start),
            ("b", 6, BenchmarkEventKind::Start),
        ],
    );
}

#[test]
fn aggregates_by_suite_and_all_suites() {
    let parse = parse_benchmark_events(
        "\
ICTK|app/a:start|0|0|0
ICTK|app/a:end|100|20|30
ICTK|app/a:start|100|20|30
ICTK|app/a:end|300|60|90
",
        &BenchmarkParserConfig::default(),
    );
    let spans = pair_benchmark_spans(&parse.events);
    let aggregates = aggregate_benchmark_spans(&spans.spans).expect("aggregate benchmark spans");

    let app = aggregates
        .iter()
        .find(|row| {
            matches!(&row.scope, BenchmarkScope::Suite(suite) if suite == "app")
                && row.span_label == "app/a"
        })
        .expect("app aggregate");
    let all = aggregates
        .iter()
        .find(|row| matches!(row.scope, BenchmarkScope::All) && row.span_label == "app/a")
        .expect("ALL aggregate");

    assert_eq!(app.runs, 2);
    assert_eq!(app.total.instructions, 300);
    assert_eq!(app.average(1).unwrap().instructions, 150);
    assert_eq!(app.min.instructions, 100);
    assert_eq!(app.max.instructions, 200);
    assert_eq!(all.runs, 2);
}

#[test]
fn updated_aggregate_totals_and_runs_drive_averages_and_comparisons() {
    let parse = parse_benchmark_events(
        "ICTK|app/a:start|0|0|0\nICTK|app/a:end|100|20|30\n",
        &BenchmarkParserConfig::default(),
    );
    let previous = aggregate_benchmark_spans(&pair_benchmark_spans(&parse.events).spans).unwrap();
    let mut current = previous.clone();
    for row in &mut current {
        row.total.instructions = 300;
        row.runs = 2;
        assert_eq!(row.average(1).unwrap().instructions, 150);
    }
    let comparison = compare_benchmark_aggregates(&current, &previous, 1).unwrap();
    assert!(
        comparison
            .iter()
            .all(|row| row.current.unwrap().average.instructions == 150
                && row.previous.unwrap().average.instructions == 100)
    );
}

#[test]
fn wide_means_preserve_integer_differences_and_explicit_projection_errors() {
    for values in [
        [1_u128 << 53, (1_u128 << 53) + 1],
        [u128::MAX - 1, u128::MAX],
    ] {
        for counter in 0..3 {
            let spans = boundary_spans(counter, values);
            let previous = aggregate_benchmark_spans(&spans[..1]).unwrap();
            let current = aggregate_benchmark_spans(&spans[1..]).unwrap();
            for row in compare_benchmark_aggregates(&current, &previous, 1).unwrap() {
                let before = row.previous.unwrap().average;
                let after = row.current.unwrap().average;
                let fields = |m: ic_testkit::benchmark::BenchmarkAverages| {
                    [m.instructions, m.heap_bytes, m.memory_bytes]
                };
                assert_eq!(fields(after)[counter], values[1]);
                assert_eq!(fields(before)[counter], values[0]);
            }
        }
    }
    let spans = boundary_spans(0, [3, 4]);
    let mut aggregate = aggregate_benchmark_spans(&spans).unwrap();
    let row = &mut aggregate[0];
    assert_eq!(row.average(10).unwrap().instructions, 35);
    assert_eq!(row.average(1).unwrap().instructions, 3);
    row.runs = 0;
    assert_eq!(
        row.average(1),
        Err(ic_testkit::benchmark::BenchmarkAverageError::ZeroDenominator)
    );
    assert!(compare_benchmark_aggregates(&aggregate, &[], 1).is_err());
    let row = &mut aggregate[0];
    row.runs = 1;
    row.total.instructions = u128::MAX;
    assert_eq!(
        row.average(2),
        Err(ic_testkit::benchmark::BenchmarkAverageError::Overflow)
    );
    assert_eq!(row.average(0).unwrap().instructions, 0);
    row.runs = u64::MAX;
    assert_eq!(row.average(u64::MAX).unwrap().instructions, u128::MAX);
    row.runs = 0;
    assert_eq!(
        row.average(0),
        Err(ic_testkit::benchmark::BenchmarkAverageError::ZeroDenominator)
    );
}

fn boundary_spans(
    counter_index: usize,
    values: [u128; 2],
) -> Vec<ic_testkit::benchmark::BenchmarkSpan> {
    let mut input = String::new();
    for value in values {
        let mut counters = [0_u128; 3];
        counters[counter_index] = value;
        writeln!(
            input,
            "ICTK|probe/run:start|0|0|0\nICTK|probe/run:end|{}|{}|{}",
            counters[0], counters[1], counters[2]
        )
        .unwrap();
    }
    let parsed = parse_benchmark_events(&input, &BenchmarkParserConfig::default());
    assert_eq!(
        parsed.malformed_markers,
        [] as [ic_testkit::benchmark::MalformedBenchmarkMarker; 0]
    );
    let paired = pair_benchmark_spans(&parsed.events);
    assert_eq!(
        paired.invalid_spans,
        [] as [ic_testkit::benchmark::InvalidBenchmarkSpan; 0]
    );
    assert_eq!(
        paired.unpaired_markers,
        [] as [ic_testkit::benchmark::RawBenchmarkEvent; 0]
    );
    assert_eq!(paired.spans.len(), 2);
    paired.spans
}

#[test]
fn aggregate_counter_overflow_is_rejected_for_every_counter_and_scope() {
    for (index, counter) in [
        BenchmarkCounter::Instructions,
        BenchmarkCounter::HeapBytes,
        BenchmarkCounter::MemoryBytes,
    ]
    .into_iter()
    .enumerate()
    {
        // Both a minimal overflow and the original two-MAX public reproduction.
        for values in [[u128::MAX, 1], [u128::MAX, u128::MAX]] {
            let mut spans = boundary_spans(index, values);
            // A named ALL suite must remain distinguishable from the global scope.
            for span in &mut spans {
                span.suite = "ALL".into();
            }
            let error = aggregate_benchmark_spans(&spans).expect_err("reject suite overflow");
            assert_eq!(error.counter, counter);
            assert_eq!(error.scope, BenchmarkScope::Suite("ALL".into()));
            assert_eq!(error.span_label, "probe/run");

            spans[1].suite = "other".into();
            let error = aggregate_benchmark_spans(&spans).expect_err("reject cross-suite overflow");
            assert_eq!(error.counter, counter);
            assert_eq!(error.scope, BenchmarkScope::All);
            assert_eq!(error.span_label, "probe/run");
        }
    }
}

#[test]
fn aggregate_totals_at_the_numeric_boundary_remain_exact() {
    for index in 0..3 {
        let spans = boundary_spans(index, [u128::MAX - 1, 1]);
        let report = aggregate_benchmark_spans(&spans).expect("MAX total fits");
        assert_eq!(report.len(), 2);
        for row in report {
            assert_eq!(row.runs, 2);
            let totals = [
                row.total.instructions,
                row.total.heap_bytes,
                row.total.memory_bytes,
            ];
            assert_eq!(totals[index], u128::MAX);
            assert!(
                totals
                    .into_iter()
                    .enumerate()
                    .all(|(i, total)| i == index || total == 0)
            );
        }
    }
    assert_eq!(
        aggregate_benchmark_spans(&[]).unwrap(),
        [] as [ic_testkit::benchmark::BenchmarkAggregateRow; 0]
    );
}

#[test]
fn span_subtraction_underflow_is_rejected_for_every_counter() {
    for index in 0..3 {
        let mut end = [10_u128; 3];
        end[index] = 9;
        let input = format!(
            "ICTK|probe/run:start|10|10|10\nICTK|probe/run:end|{}|{}|{}\n",
            end[0], end[1], end[2]
        );
        let parsed = parse_benchmark_events(&input, &BenchmarkParserConfig::default());
        assert_eq!(
            parsed.malformed_markers,
            [] as [ic_testkit::benchmark::MalformedBenchmarkMarker; 0]
        );
        let paired = pair_benchmark_spans(&parsed.events);
        assert_eq!(
            paired.spans,
            [] as [ic_testkit::benchmark::BenchmarkSpan; 0]
        );
        assert_eq!(paired.invalid_spans.len(), 1);
        assert_eq!(
            paired.unpaired_markers,
            [] as [ic_testkit::benchmark::RawBenchmarkEvent; 0]
        );
    }
}

#[test]
fn authored_all_suite_does_not_collide_with_cross_suite_aggregate() {
    let config = BenchmarkParserConfig {
        suite_derivation: SuiteDerivation::Fixed("ALL".to_string()),
        ..BenchmarkParserConfig::default()
    };
    let parse = parse_benchmark_events(
        "\
ICTK|benchmark:start|0|0|0
ICTK|benchmark:end|100|20|30
",
        &config,
    );
    let mut spans = pair_benchmark_spans(&parse.events).spans;
    for suite in ["z", "", "a"] {
        let mut span = spans[0].clone();
        span.suite = suite.into();
        spans.push(span);
    }
    spans.reverse();
    let aggregates = aggregate_benchmark_spans(&spans).expect("aggregate benchmark spans");

    assert_eq!(
        aggregates
            .iter()
            .map(|row| row.scope.clone())
            .collect::<Vec<_>>(),
        [
            BenchmarkScope::Suite(String::new()),
            BenchmarkScope::Suite("ALL".into()),
            BenchmarkScope::Suite("a".into()),
            BenchmarkScope::Suite("z".into()),
            BenchmarkScope::All,
        ]
    );
    let authored = aggregates
        .iter()
        .find(|row| matches!(&row.scope, BenchmarkScope::Suite(suite) if suite == "ALL"))
        .expect("authored ALL suite");
    let all = aggregates
        .iter()
        .find(|row| matches!(row.scope, BenchmarkScope::All))
        .expect("cross-suite ALL aggregate");
    assert_eq!(authored.scope, BenchmarkScope::Suite("ALL".into()));
    assert_eq!(all.scope, BenchmarkScope::All);
    assert_eq!(authored.runs, 1);
    assert_eq!(all.runs, 4);
}

#[test]
fn compares_current_average_against_previous_average() {
    let previous = aggregate_benchmark_spans(
        &pair_benchmark_spans(
            &parse_benchmark_events(
                "\
ICTK|app/a:start|0|0|0
ICTK|app/a:end|100|100|100
",
                &BenchmarkParserConfig::default(),
            )
            .events,
        )
        .spans,
    )
    .expect("aggregate benchmark spans");
    let current = aggregate_benchmark_spans(
        &pair_benchmark_spans(
            &parse_benchmark_events(
                "\
ICTK|app/a:start|0|0|0
ICTK|app/a:end|150|50|100
",
                &BenchmarkParserConfig::default(),
            )
            .events,
        )
        .spans,
    )
    .expect("aggregate benchmark spans");

    let comparison = compare_benchmark_aggregates(&current, &previous, 1).unwrap();
    let app = comparison
        .iter()
        .find(|row| {
            matches!(&row.scope, BenchmarkScope::Suite(suite) if suite == "app")
                && row.span_label == "app/a"
        })
        .expect("comparison row");

    assert_eq!(app.current.unwrap().average.instructions, 150);
    assert_eq!(app.previous.unwrap().average.instructions, 100);
    assert_eq!(app.current.unwrap().average.heap_bytes, 50);
    assert_eq!(app.previous.unwrap().average.heap_bytes, 100);
}

#[test]
fn comparisons_preserve_order_missing_rows_and_refuse_duplicates() {
    let aggregate = |input| {
        let parsed = parse_benchmark_events(
            input,
            &BenchmarkParserConfig {
                suite_derivation: SuiteDerivation::Fixed("ALL".into()),
                ..BenchmarkParserConfig::default()
            },
        );
        aggregate_benchmark_spans(&pair_benchmark_spans(&parsed.events).spans).unwrap()
    };
    let previous = aggregate(
        "ICTK|common:start|0|0|0\nICTK|common:end|100|0|0\n\
         ICTK|previous:start|0|0|0\nICTK|previous:end|50|0|0\n",
    );
    let mut current = aggregate(
        "ICTK|common:start|0|0|0\nICTK|common:end|150|0|0\n\
         ICTK|current:start|0|0|0\nICTK|current:end|75|0|0\n",
    );
    current.reverse();
    let mut duplicate = current
        .iter()
        .find(|row| matches!(row.scope, BenchmarkScope::Suite(_)) && row.span_label == "common")
        .unwrap()
        .clone();
    duplicate.total.instructions = 300;
    current.push(duplicate);
    assert!(matches!(
        compare_benchmark_aggregates(&current, &previous, 1),
        Err(ic_testkit::benchmark::BenchmarkComparisonError::DuplicateCurrent { .. })
    ));
    current.pop();
    let comparison = compare_benchmark_aggregates(&current, &previous, 1).unwrap();

    assert_eq!(
        comparison
            .iter()
            .map(|row| (row.scope.clone(), row.span_label.as_str()))
            .collect::<Vec<_>>(),
        [
            (BenchmarkScope::Suite("ALL".into()), "common"),
            (BenchmarkScope::Suite("ALL".into()), "current"),
            (BenchmarkScope::Suite("ALL".into()), "previous"),
            (BenchmarkScope::All, "common"),
            (BenchmarkScope::All, "current"),
            (BenchmarkScope::All, "previous"),
        ],
    );
    assert_eq!(comparison[0].current.unwrap().average.instructions, 150);
    assert_eq!(comparison[3].current.unwrap().average.instructions, 150);
    for row in &comparison {
        assert_eq!(row.current.or(row.previous).unwrap().average.heap_bytes, 0);
        match row.span_label.as_str() {
            "current" => {
                assert_eq!(row.current.unwrap().runs, 1);
                assert_eq!(row.previous, None);
            }
            "previous" => {
                assert_eq!(row.previous.unwrap().runs, 1);
                assert_eq!(row.current, None);
            }
            _ => {}
        }
    }

    let mut duplicated_previous = previous;
    let mut duplicate = duplicated_previous
        .iter()
        .find(|row| matches!(row.scope, BenchmarkScope::Suite(_)) && row.span_label == "common")
        .unwrap()
        .clone();
    duplicate.total.instructions = 600;
    duplicate.runs = 2;
    duplicated_previous.push(duplicate);
    assert!(matches!(
        compare_benchmark_aggregates(&current, &duplicated_previous, 1),
        Err(ic_testkit::benchmark::BenchmarkComparisonError::DuplicatePrevious { .. })
    ));
    assert!(matches!(
        compare_benchmark_aggregates(&duplicated_previous, &current, 1),
        Err(ic_testkit::benchmark::BenchmarkComparisonError::DuplicateCurrent { .. })
    ));
    assert_eq!(
        compare_benchmark_aggregates(&[], &[], 1).unwrap(),
        [] as [ic_testkit::benchmark::BenchmarkComparisonRow; 0],
    );
}
