//! Render captured benchmark markers using this example's CSV/Markdown layout.
//! Usage: benchmark_report INPUT OUTPUT_DIRECTORY [PREVIOUS_INPUT]

mod output;

use ic_testkit::benchmark::{
    BenchmarkParserConfig, aggregate_benchmark_spans, compare_benchmark_aggregates,
    pair_benchmark_spans, parse_benchmark_events,
};
use output::BenchmarkRunReport;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::Command::new("benchmark_report")
        .about("Render captured benchmark markers as CSV and Markdown")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            clap::Arg::new("input")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf))
                .allow_hyphen_values(true),
        )
        .arg(
            clap::Arg::new("output_directory")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf))
                .allow_hyphen_values(true),
        )
        .arg(
            clap::Arg::new("previous_input")
                .value_parser(clap::value_parser!(PathBuf))
                .allow_hyphen_values(true),
        )
        .get_matches();
    let input = matches.get_one::<PathBuf>("input").expect("required input");
    let directory = matches
        .get_one::<PathBuf>("output_directory")
        .expect("required output directory");
    let previous = matches.get_one::<PathBuf>("previous_input");
    let mut report = read_measurements(input)?;
    if let Some(previous) = previous {
        let previous = read_measurements(previous)?;
        report.comparison = Some(compare_benchmark_aggregates(
            &report.aggregates,
            &previous.aggregates,
            1,
        )?);
    }
    output::write_benchmark_report_dir(&report, directory)?;
    Ok(())
}

fn read_measurements(path: &Path) -> Result<BenchmarkRunReport, Box<dyn Error>> {
    let parse = parse_benchmark_events(
        &fs::read_to_string(path)?,
        &BenchmarkParserConfig::default(),
    );
    let spans = pair_benchmark_spans(&parse.events);
    let aggregates = aggregate_benchmark_spans(&spans.spans)?;
    Ok(BenchmarkRunReport {
        parse,
        spans,
        aggregates,
        comparison: None,
    })
}
