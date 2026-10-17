//! Render captured benchmark markers using this example's CSV/Markdown layout.
//! Usage: benchmark_report INPUT OUTPUT_DIRECTORY [PREVIOUS_INPUT]

mod output;

use ic_testkit::benchmark::{
    BenchmarkParserConfig, aggregate_benchmark_spans, compare_benchmark_aggregates,
    pair_benchmark_spans, parse_benchmark_events,
};
use output::BenchmarkRunReport;
use std::{error::Error, fs, io, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let usage = "usage: benchmark_report INPUT OUTPUT_DIRECTORY [PREVIOUS_INPUT]";
    let mut arguments = std::env::args_os().skip(1);
    let input = arguments.next().ok_or(usage)?;
    let directory = arguments.next().ok_or(usage)?;
    let previous = arguments.next();
    if arguments.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, usage).into());
    }
    let mut report = read_measurements(Path::new(&input))?;
    if let Some(previous) = previous {
        let previous = read_measurements(Path::new(&previous))?;
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
