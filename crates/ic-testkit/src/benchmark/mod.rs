//! Parse, pair, aggregate, compare, and report compact benchmark markers.
//!
//! The normal pipeline is [`parse_benchmark_events`],
//! [`pair_benchmark_spans`], [`aggregate_benchmark_spans`], and optionally
//! [`compare_benchmark_aggregates`]. Native report output lives in
//! the repository's `benchmark_report` example. Marker
//! producers can use [`format_marker`] on the host or
//! `ic_testkit::performance` in canister code.
//!
//! Totals and run counts remain exact integers. Means use Testkit's exact
//! scaled integer projection, rounded down. Callers choose the scale and handle
//! projection errors; reports use scale one in the original counter units.
//! Comparisons retain current and previous means instead of percentage estimates.

use std::collections::{BTreeMap, btree_map::Entry};

/// Why a benchmark's scaled average cannot be represented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkAverageError {
    /// A mean is undefined when the run count is zero, even at scale zero.
    ZeroDenominator,
    /// The scaled average exceeds `u128::MAX`.
    Overflow,
}

impl std::fmt::Display for BenchmarkAverageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ZeroDenominator => "benchmark run count is zero",
            Self::Overflow => "scaled benchmark average exceeds u128",
        })
    }
}

impl std::error::Error for BenchmarkAverageError {}

fn checked_scaled_average(
    total: u128,
    runs: u64,
    scale: u64,
) -> Result<u128, BenchmarkAverageError> {
    if runs == 0 {
        return Err(BenchmarkAverageError::ZeroDenominator);
    }
    let runs = u128::from(runs);
    let scale = u128::from(scale);
    // Split before multiplying: only the final average must fit. The remainder
    // and scale are at most u64::MAX, so their product always fits in u128.
    let fraction = (total % runs) * scale / runs;
    (total / runs)
        .checked_mul(scale)
        .and_then(|whole| whole.checked_add(fraction))
        .ok_or(BenchmarkAverageError::Overflow)
}

/// A comparison refuses ambiguous row identities or unrepresentable means.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BenchmarkComparisonError {
    /// More than one current row has the same scope and span.
    DuplicateCurrent {
        /// Repeated scope.
        scope: BenchmarkScope,
        /// Repeated span label.
        span_label: String,
    },
    /// More than one previous row has the same scope and span.
    DuplicatePrevious {
        /// Repeated scope.
        scope: BenchmarkScope,
        /// Repeated span label.
        span_label: String,
    },
    /// A row's selected mean cannot be represented.
    Average(BenchmarkAverageError),
}

impl std::fmt::Display for BenchmarkComparisonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCurrent { scope, span_label } => write!(
                formatter,
                "duplicate current benchmark row: {scope:?}/{span_label}"
            ),
            Self::DuplicatePrevious { scope, span_label } => write!(
                formatter,
                "duplicate previous benchmark row: {scope:?}/{span_label}"
            ),
            Self::Average(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for BenchmarkComparisonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Average(error) => Some(error),
            Self::DuplicateCurrent { .. } | Self::DuplicatePrevious { .. } => None,
        }
    }
}
impl From<BenchmarkAverageError> for BenchmarkComparisonError {
    fn from(error: BenchmarkAverageError) -> Self {
        Self::Average(error)
    }
}

/// Default prefix for benchmark marker lines.
pub const DEFAULT_PREFIX: &str = "ICTK";

/// Structural identity of one suite or the cross-suite aggregate.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BenchmarkScope {
    /// One caller-authored suite, including a suite literally named `ALL`.
    Suite(String),
    /// Aggregate across every suite.
    All,
}

/// Marker prefixes, suite naming and noise handling selected by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkParserConfig {
    /// Accepted marker prefixes.
    pub prefixes: Vec<String>,
    /// Rule used to derive each event’s suite.
    pub suite_derivation: SuiteDerivation,
    /// When enabled, non-empty lines without a configured marker prefix are
    /// reported as malformed markers instead of ignored log noise.
    pub strict: bool,
}

impl Default for BenchmarkParserConfig {
    fn default() -> Self {
        Self {
            prefixes: vec![DEFAULT_PREFIX.to_string()],
            suite_derivation: SuiteDerivation::FirstPathSegment,
            strict: false,
        }
    }
}

/// How a marker label determines its suite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuiteDerivation {
    /// Use the first slash-delimited label segment, falling back to the whole label.
    FirstPathSegment,
    /// Use the supplied suite for every marker.
    Fixed(String),
}

impl SuiteDerivation {
    /// Derive the suite name for a marker label.
    #[must_use]
    pub fn derive_suite(&self, span_label: &str) -> String {
        match self {
            Self::FirstPathSegment => span_label
                .split('/')
                .next()
                .filter(|part| !part.is_empty())
                .unwrap_or(span_label)
                .to_string(),
            Self::Fixed(suite) => suite.clone(),
        }
    }
}

/// Whether a marker opens or closes a measured span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkEventKind {
    /// Opening counter sample.
    Start,
    /// Closing counter sample.
    End,
}

/// Stream from which a marker was collected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkEventSource {
    /// Stream identity was not supplied.
    Unknown,
    /// Captured standard output.
    Stdout,
    /// Captured standard error.
    Stderr,
    /// Fetched canister log.
    FetchedLog,
}

impl BenchmarkEventSource {
    /// Return the stable stream label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::FetchedLog => "fetched_log",
        }
    }
}

/// Exact instruction and byte counters reported by a marker.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkCounters {
    /// Instruction counter in the representation described by this type.
    pub instructions: u128,
    /// Heap byte counter in the representation described by this type.
    pub heap_bytes: u128,
    /// Memory byte counter in the representation described by this type.
    pub memory_bytes: u128,
}

impl BenchmarkCounters {
    fn checked_delta(self, start: Self) -> Option<Self> {
        Some(Self {
            instructions: self.instructions.checked_sub(start.instructions)?,
            heap_bytes: self.heap_bytes.checked_sub(start.heap_bytes)?,
            memory_bytes: self.memory_bytes.checked_sub(start.memory_bytes)?,
        })
    }

    fn checked_add(self, other: Self) -> Result<Self, BenchmarkCounter> {
        Ok(Self {
            instructions: self
                .instructions
                .checked_add(other.instructions)
                .ok_or(BenchmarkCounter::Instructions)?,
            heap_bytes: self
                .heap_bytes
                .checked_add(other.heap_bytes)
                .ok_or(BenchmarkCounter::HeapBytes)?,
            memory_bytes: self
                .memory_bytes
                .checked_add(other.memory_bytes)
                .ok_or(BenchmarkCounter::MemoryBytes)?,
        })
    }

    fn min_assign(&mut self, other: Self) {
        self.instructions = self.instructions.min(other.instructions);
        self.heap_bytes = self.heap_bytes.min(other.heap_bytes);
        self.memory_bytes = self.memory_bytes.min(other.memory_bytes);
    }

    fn max_assign(&mut self, other: Self) {
        self.instructions = self.instructions.max(other.instructions);
        self.heap_bytes = self.heap_bytes.max(other.heap_bytes);
        self.memory_bytes = self.memory_bytes.max(other.memory_bytes);
    }
}

/// Parsed marker with its original stream and line provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawBenchmarkEvent {
    /// Matched marker prefix.
    pub prefix: String,
    /// Derived suite name.
    pub suite: String,
    /// Marker label used to group matching spans.
    pub span_label: String,
    /// Opening or closing marker.
    pub kind: BenchmarkEventKind,
    /// Counters observed at this marker.
    pub counters: BenchmarkCounters,
    /// One-based line number in the original input stream.
    pub source_line: usize,
    /// Originating stream.
    pub source: BenchmarkEventSource,
}

/// Rejected marker and its parse diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MalformedBenchmarkMarker {
    /// One-based line number in the original input stream.
    pub source_line: usize,
    /// Originating stream.
    pub source: BenchmarkEventSource,
    /// Original rejected line.
    pub line: String,
    /// Explanation of the rejected marker.
    pub reason: String,
}

/// Parsed events, rejected markers and ignored log noise.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkParseReport {
    /// Successfully parsed markers in input order.
    pub events: Vec<RawBenchmarkEvent>,
    /// Rejected marker lines with diagnostics.
    pub malformed_markers: Vec<MalformedBenchmarkMarker>,
    /// Lines ignored as non-marker noise.
    pub ignored_line_count: usize,
}

/// Matched start/end markers with nonnegative counter differences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkSpan {
    /// Derived suite name.
    pub suite: String,
    /// Marker label used to group matching spans.
    pub span_label: String,
    /// One-based line of the opening marker.
    pub start_line: usize,
    /// One-based line of the closing marker.
    pub end_line: usize,
    /// Counter values observed at the closing marker.
    pub end: BenchmarkCounters,
    /// Closing counters minus opening counters.
    pub delta: BenchmarkCounters,
}

/// Matched markers whose counters decreased.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidBenchmarkSpan {
    /// Opening marker.
    pub start: RawBenchmarkEvent,
    /// Closing marker.
    pub end: RawBenchmarkEvent,
}

/// Pairing result retaining unmatched and invalid markers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkSpanReport {
    /// Successfully paired spans.
    pub spans: Vec<BenchmarkSpan>,
    /// Markers without a matching start or end.
    pub unpaired_markers: Vec<RawBenchmarkEvent>,
    /// Matched pairs whose counters decreased.
    pub invalid_spans: Vec<InvalidBenchmarkSpan>,
}

/// Exact totals and extrema for one scope and span label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkAggregateRow {
    /// Marker label used to group matching spans.
    pub span_label: String,
    /// Number of measured spans.
    pub runs: u64,
    /// Sum of the per-span counter differences.
    pub total: BenchmarkCounters,
    /// Smallest per-counter span difference.
    pub min: BenchmarkCounters,
    /// Largest per-counter span difference.
    pub max: BenchmarkCounters,
    /// Largest per-counter closing value.
    pub peak_end: BenchmarkCounters,
    /// Suite identity or the cross-suite aggregate.
    pub scope: BenchmarkScope,
}

/// Floor-rounded scaled means of exact aggregate totals and run counts.
///
/// Fields are `floor(total * scale / runs)`; choose a positive scale to
/// interpret them in the original counter unit. Scale zero projects zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchmarkAverages {
    /// Multiplier applied before dividing by the run count.
    pub scale: u64,
    /// Instruction counter in the representation described by this type.
    pub instructions: u128,
    /// Heap byte counter in the representation described by this type.
    pub heap_bytes: u128,
    /// Memory byte counter in the representation described by this type.
    pub memory_bytes: u128,
}

/// Counter whose aggregate cannot be represented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkCounter {
    /// Measured span count.
    Runs,
    /// Instruction total.
    Instructions,
    /// Heap byte total.
    HeapBytes,
    /// Memory byte total.
    MemoryBytes,
}

/// Overflow rejects the complete aggregation, without returning partial totals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkAggregateError {
    /// Marker label used to group matching spans.
    pub span_label: String,
    /// Counter that overflowed.
    pub counter: BenchmarkCounter,
    /// Suite identity or the cross-suite aggregate.
    pub scope: BenchmarkScope,
}

impl std::fmt::Display for BenchmarkAggregateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "benchmark aggregate overflow: scope={:?}, span={}, counter={:?}",
            self.scope, self.span_label, self.counter
        )
    }
}

impl std::error::Error for BenchmarkAggregateError {}

/// Run count and scaled means for one present comparison side.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchmarkMeasurement {
    /// Number of measured spans.
    pub runs: u64,
    /// Scaled, floor-rounded means.
    pub average: BenchmarkAverages,
}

/// Current and previous measurements, preserving missing rows and measured zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkComparisonRow {
    /// Marker label used to group matching spans.
    pub span_label: String,
    /// Current measurement, absent when that side has no row.
    pub current: Option<BenchmarkMeasurement>,
    /// Previous measurement, absent when that side has no row.
    pub previous: Option<BenchmarkMeasurement>,
    /// Suite identity or the cross-suite aggregate.
    pub scope: BenchmarkScope,
}

impl BenchmarkAggregateRow {
    /// Project exact totals at the caller's scale, rounding down.
    ///
    /// # Errors
    /// Zero runs and unrepresentable scaled results return [`BenchmarkAverageError`].
    pub fn average(&self, scale: u64) -> Result<BenchmarkAverages, BenchmarkAverageError> {
        Ok(BenchmarkAverages {
            scale,
            instructions: checked_scaled_average(self.total.instructions, self.runs, scale)?,
            heap_bytes: checked_scaled_average(self.total.heap_bytes, self.runs, scale)?,
            memory_bytes: checked_scaled_average(self.total.memory_bytes, self.runs, scale)?,
        })
    }
}

/// Format one marker using the supplied prefix, label and counters.
#[must_use]
pub fn format_marker(prefix: &str, label: &str, counters: BenchmarkCounters) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        prefix, label, counters.instructions, counters.heap_bytes, counters.memory_bytes
    )
}

/// Parse markers without assigning a stream identity.
#[must_use]
pub fn parse_benchmark_events(input: &str, config: &BenchmarkParserConfig) -> BenchmarkParseReport {
    parse_benchmark_events_from_source(input, config, BenchmarkEventSource::Unknown)
}

/// Parse markers while retaining the supplied stream identity.
#[must_use]
pub fn parse_benchmark_events_from_source(
    input: &str,
    config: &BenchmarkParserConfig,
    source: BenchmarkEventSource,
) -> BenchmarkParseReport {
    let mut report = BenchmarkParseReport::default();
    append_benchmark_events(&mut report, input, config, source);
    report
}

fn append_benchmark_events(
    report: &mut BenchmarkParseReport,
    input: &str,
    config: &BenchmarkParserConfig,
    source: BenchmarkEventSource,
) {
    for (index, line) in input.lines().enumerate() {
        let source_line = index + 1;
        if !has_configured_prefix(line, &config.prefixes) {
            if config.strict && !line.trim().is_empty() {
                report.malformed_markers.push(malformed(
                    source_line,
                    source,
                    line,
                    "line does not use a configured marker prefix",
                ));
            } else {
                report.ignored_line_count += 1;
            }
            continue;
        }

        match parse_marker_line(line, source_line, source, config) {
            Ok(event) => report.events.push(event),
            Err(marker) => report.malformed_markers.push(marker),
        }
    }
}

/// Parse separately captured stdout and stderr.
///
/// Separate streams do not carry global ordering, so events are returned in
/// stdout-then-stderr order. If a benchmark span can start on one stream and end
/// on the other, capture combined process output and use [`parse_benchmark_events`].
#[must_use]
pub fn parse_benchmark_events_from_captured_output(
    stdout: &str,
    stderr: &str,
    config: &BenchmarkParserConfig,
) -> BenchmarkParseReport {
    let mut report = BenchmarkParseReport::default();
    append_benchmark_events(&mut report, stdout, config, BenchmarkEventSource::Stdout);
    append_benchmark_events(&mut report, stderr, config, BenchmarkEventSource::Stderr);
    report
}

/// Pair markers by suite and label, using the most recent unmatched start.
#[must_use]
#[expect(
    clippy::missing_panics_doc,
    reason = "only an internal nonempty-stack invariant can panic"
)]
pub fn pair_benchmark_spans(events: &[RawBenchmarkEvent]) -> BenchmarkSpanReport {
    let mut report = BenchmarkSpanReport::default();
    let mut open_starts: BTreeMap<(&str, &str), Vec<&RawBenchmarkEvent>> = BTreeMap::new();

    for event in events {
        let key = (event.suite.as_str(), event.span_label.as_str());
        match event.kind {
            BenchmarkEventKind::Start => open_starts.entry(key).or_default().push(event),
            BenchmarkEventKind::End => match open_starts.entry(key) {
                Entry::Occupied(mut entry) => {
                    let start = entry
                        .get_mut()
                        .pop()
                        .expect("open marker stacks must remain nonempty");
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                    push_paired_span(&mut report, start, event);
                }
                Entry::Vacant(_) => report.unpaired_markers.push(event.clone()),
            },
        }
    }

    for starts in open_starts.into_values() {
        for event in starts {
            report.unpaired_markers.push(event.clone());
        }
    }

    report
}

/// Aggregate spans by named suite and across all suites.
///
/// # Errors
/// Returns an error if any counter total or run count overflows. No partial
/// report is returned, and totals are never wrapped or saturated.
pub fn aggregate_benchmark_spans(
    spans: &[BenchmarkSpan],
) -> Result<Vec<BenchmarkAggregateRow>, BenchmarkAggregateError> {
    let mut rows = BTreeMap::new();

    for span in spans {
        add_span_to_aggregate(&mut rows, AggregateScope::Suite(&span.suite), span)?;
        add_span_to_aggregate(&mut rows, AggregateScope::All, span)?;
    }

    Ok(rows
        .into_iter()
        .map(|((scope, span_label), builder)| {
            builder.finish(scope.into_owned(), span_label.to_owned())
        })
        .collect())
}

/// Pair current and previous means at the same caller-selected scale.
///
/// # Errors
/// Returns a typed error for duplicate scope/span rows on either side or an invalid mean.
pub fn compare_benchmark_aggregates(
    current: &[BenchmarkAggregateRow],
    previous: &[BenchmarkAggregateRow],
    scale: u64,
) -> Result<Vec<BenchmarkComparisonRow>, BenchmarkComparisonError> {
    let mut rows = BTreeMap::new();
    for row in current {
        if rows
            .insert((&row.scope, row.span_label.as_str()), (Some(row), None))
            .is_some()
        {
            return Err(BenchmarkComparisonError::DuplicateCurrent {
                scope: row.scope.clone(),
                span_label: row.span_label.clone(),
            });
        }
    }
    for row in previous {
        let previous = &mut rows
            .entry((&row.scope, row.span_label.as_str()))
            .or_insert((None, None))
            .1;
        if previous.replace(row).is_some() {
            return Err(BenchmarkComparisonError::DuplicatePrevious {
                scope: row.scope.clone(),
                span_label: row.span_label.clone(),
            });
        }
    }
    let measurement = |row: &BenchmarkAggregateRow| -> Result<_, BenchmarkAverageError> {
        Ok(BenchmarkMeasurement {
            runs: row.runs,
            average: row.average(scale)?,
        })
    };

    rows.into_iter()
        .map(|((scope, span_label), (current_row, previous_row))| {
            Ok(BenchmarkComparisonRow {
                span_label: span_label.to_owned(),
                current: current_row.map(measurement).transpose()?,
                previous: previous_row.map(measurement).transpose()?,
                scope: scope.clone(),
            })
        })
        .collect()
}

fn parse_marker_line(
    line: &str,
    source_line: usize,
    source: BenchmarkEventSource,
    config: &BenchmarkParserConfig,
) -> Result<RawBenchmarkEvent, MalformedBenchmarkMarker> {
    let mut parts = line.split('|');
    // Five fields and an absent sixth establish the complete marker shape
    // without collecting an arbitrary number of columns from malformed logs.
    let columns: [Option<&str>; 6] = std::array::from_fn(|_| parts.next());
    let [
        Some(prefix),
        Some(label),
        Some(instructions),
        Some(heap_bytes),
        Some(memory_bytes),
        None,
    ] = columns
    else {
        return Err(malformed(
            source_line,
            source,
            line,
            "expected five pipe-separated columns",
        ));
    };

    if !config.prefixes.iter().any(|known| known == prefix) {
        return Err(malformed(
            source_line,
            source,
            line,
            "prefix is not configured",
        ));
    }

    if label.is_empty() {
        return Err(malformed(source_line, source, line, "label is empty"));
    }

    let (span_label, kind) = split_label_kind(label).ok_or_else(|| {
        malformed(
            source_line,
            source,
            line,
            "label must end in :start or :end",
        )
    })?;

    let counters = BenchmarkCounters {
        instructions: parse_counter(instructions, source_line, source, line, "instructions")?,
        heap_bytes: parse_counter(heap_bytes, source_line, source, line, "heap_bytes")?,
        memory_bytes: parse_counter(memory_bytes, source_line, source, line, "memory_bytes")?,
    };
    let suite = config.suite_derivation.derive_suite(span_label);

    Ok(RawBenchmarkEvent {
        prefix: prefix.to_string(),
        suite,
        span_label: span_label.to_string(),
        kind,
        counters,
        source_line,
        source,
    })
}

fn parse_counter(
    value: &str,
    source_line: usize,
    source: BenchmarkEventSource,
    line: &str,
    name: &str,
) -> Result<u128, MalformedBenchmarkMarker> {
    if value.is_empty() {
        return Err(malformed(
            source_line,
            source,
            line,
            format!("{name} counter is empty"),
        ));
    }

    value.parse::<u128>().map_err(|_| {
        malformed(
            source_line,
            source,
            line,
            format!("{name} counter is not an unsigned integer"),
        )
    })
}

fn split_label_kind(label: &str) -> Option<(&str, BenchmarkEventKind)> {
    let start = label.strip_suffix(":start");
    let end = label.strip_suffix(":end");

    match (start, end) {
        (Some(span_label), None) if !span_label.is_empty() => {
            Some((span_label, BenchmarkEventKind::Start))
        }
        (None, Some(span_label)) if !span_label.is_empty() => {
            Some((span_label, BenchmarkEventKind::End))
        }
        _ => None,
    }
}

fn has_configured_prefix(line: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| {
        line.strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('|'))
    })
}

fn malformed(
    source_line: usize,
    source: BenchmarkEventSource,
    line: &str,
    reason: impl Into<String>,
) -> MalformedBenchmarkMarker {
    MalformedBenchmarkMarker {
        source_line,
        source,
        line: line.to_string(),
        reason: reason.into(),
    }
}

fn push_paired_span(
    report: &mut BenchmarkSpanReport,
    start: &RawBenchmarkEvent,
    end: &RawBenchmarkEvent,
) {
    if let Some(delta) = end.counters.checked_delta(start.counters) {
        report.spans.push(BenchmarkSpan {
            suite: start.suite.clone(),
            span_label: start.span_label.clone(),
            start_line: start.source_line,
            end_line: end.source_line,
            end: end.counters,
            delta,
        });
    } else {
        report.invalid_spans.push(InvalidBenchmarkSpan {
            start: start.clone(),
            end: end.clone(),
        });
    }
}

// Borrow grouping identities until a row or error must own them. Keep the same
// suite-before-all ordering as the public report scope.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum AggregateScope<'a> {
    Suite(&'a str),
    All,
}

impl AggregateScope<'_> {
    fn into_owned(self) -> BenchmarkScope {
        match self {
            Self::Suite(suite) => BenchmarkScope::Suite(suite.to_owned()),
            Self::All => BenchmarkScope::All,
        }
    }
}

#[derive(Clone, Debug)]
struct AggregateBuilder {
    runs: u64,
    total: BenchmarkCounters,
    min: BenchmarkCounters,
    max: BenchmarkCounters,
    peak_end: BenchmarkCounters,
}

impl AggregateBuilder {
    const fn new(span: &BenchmarkSpan) -> Self {
        Self {
            runs: 1,
            total: span.delta,
            min: span.delta,
            max: span.delta,
            peak_end: span.end,
        }
    }

    fn push(&mut self, span: &BenchmarkSpan) -> Result<(), BenchmarkCounter> {
        let runs = self.runs.checked_add(1).ok_or(BenchmarkCounter::Runs)?;
        let total = self.total.checked_add(span.delta)?;
        self.runs = runs;
        self.total = total;
        self.min.min_assign(span.delta);
        self.max.max_assign(span.delta);
        self.peak_end.max_assign(span.end);
        Ok(())
    }

    const fn finish(self, scope: BenchmarkScope, span_label: String) -> BenchmarkAggregateRow {
        BenchmarkAggregateRow {
            span_label,
            runs: self.runs,
            total: self.total,
            min: self.min,
            max: self.max,
            peak_end: self.peak_end,
            scope,
        }
    }
}

fn add_span_to_aggregate<'a>(
    rows: &mut BTreeMap<(AggregateScope<'a>, &'a str), AggregateBuilder>,
    scope: AggregateScope<'a>,
    span: &'a BenchmarkSpan,
) -> Result<(), BenchmarkAggregateError> {
    match rows.entry((scope, span.span_label.as_str())) {
        Entry::Occupied(mut entry) => entry.get_mut().push(span).map_err(|counter| {
            let (scope, span_label) = entry.key();
            BenchmarkAggregateError {
                span_label: (*span_label).to_owned(),
                counter,
                scope: scope.into_owned(),
            }
        }),
        Entry::Vacant(entry) => {
            entry.insert(AggregateBuilder::new(span));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AggregateBuilder, BenchmarkCounter, BenchmarkCounters, BenchmarkSpan};

    #[test]
    fn aggregate_run_count_overflow_is_rejected_without_changing_totals() {
        let span = BenchmarkSpan {
            suite: "probe".into(),
            span_label: "probe/run".into(),
            start_line: 1,
            end_line: 2,
            end: BenchmarkCounters::default(),
            delta: BenchmarkCounters::default(),
        };
        let mut builder = AggregateBuilder::new(&span);
        builder.runs = u64::MAX;
        assert_eq!(builder.push(&span), Err(BenchmarkCounter::Runs));
        assert_eq!(builder.runs, u64::MAX);
        assert_eq!(builder.total, BenchmarkCounters::default());
    }
}
