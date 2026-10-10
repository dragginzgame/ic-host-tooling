# Maintenance and API decision audit

Source: Host 0.10.2 `6b755763aca71b7ea8c3de40edf032093dfdc62c`, clean at entry.
Shared baseline: `83efac446348dea024798a331d77933b24b429dc`; local overlay:
AGENTS.md at the source commit. Methods: `audits/code-hygiene.md` and
`audits/complexity-and-technical-debt.md`. Trigger: maintainer requested things
that seem awkward, stale or contrary to good maintenance practice, for decisions.
This is a first broad decision baseline, not comparable to a prior numeric score.

Verdict: **PASS WITH FINDINGS** for the inspected maintenance/API boundaries.
No new runtime integrity failure was demonstrated. This does not qualify runtime
correctness, vulnerability freshness, performance, hardware crash durability or
native macOS behavior. Source inspection and existing design-issue review only;
no compilation, tests, benchmarks, downloads, release or sibling mutations ran.
The only local write is this immutable audit evidence.

The active decision owner is [Host #46](https://github.com/dragginzgame/ic-host-tooling/issues/46).
The previously identified snapshot-path defect remains
[Host #45](https://github.com/dragginzgame/ic-host-tooling/issues/45).
This report does not create a second issue/status queue.

## Scope and ownership trace

Inspected the four crate manifests/facades, public process admission and capture,
child ownership and cleanup, filesystem publication/read contracts, artifact
copy/error contracts, IC extraction/response example dependencies, Make/CI,
release/publication adapters and current handoff. Targeted sibling caller reads
were limited to Canic foreground communication and Testkit Cargo-output/error
consumers; these moving dirty checkouts are applicability evidence, not qualified
consumer versions. All external package APIs may have consumers outside this set.

| State or policy | Current owner | Important interaction |
| --- | --- | --- |
| Direct child versus owned group | OwnedChild | Waiting, handoff, failure and Drop have different cleanup obligations |
| Immediate kill versus TERM/grace/bounded reap | CleanupPolicy | Group creation permits selection; direct spawn fixes KillAndWait |
| Finite versus absent communication deadline | OutputLimits / CommunicationLimits | Capture/admission accept finite limits; communication supports None |
| Hard output quota and retained output | process::read_chunk | Observer receives only bytes also retained in memory |
| Primary failure and cleanup evidence | ExecutionError / CleanupError / NamedWriteError | Parallel cleanup failures need more than one linear source chain |
| Replace/create-only and staging permissions | WriteOptions / durable engine | Path, descriptor, named and closed-writer admission retain different custody contracts |
| Executable pin versus observed installed version | ToolSpec / VersionSpec | Both converge on one verification/capture implementation |
| Release intent, qualification and registry effects | Shared runner / Host adapters / Cargo | Same-release recovery and partial publication cannot be discarded as old machinery |

The generic artifact owner remains below filesystem/process and IC interpretation.
The feature graph separates response-only and IC-limit users from extraction.
No compatibility reexports or duplicate production publication engine were found
in the inspected owners. Large files were treated as inspection leads, not
standalone findings.

## Findings for decisions

| Severity | Evidence and consequence | Recommended disposition |
| --- | --- | --- |
| MEDIUM | `tool/process.rs:296–343` appends all observed bytes before calling the observer. Canic `icp/run.rs` and Testkit `artifacts/wasm_cache.rs` use usize::MAX. Live forwarding therefore retains whole output; allocation grows with output. No leak/OOM or regression is claimed. | Decide whether consumers still need complete output. If not, separate forwarding from retained diagnostics in the existing engine; preserve hard quota mode and unlimited elapsed build time. |
| MEDIUM | `durable/named/mod.rs:40–61` omits secondary cleanup_error from Display/source. `tool/mod.rs:263–287` similarly omits cleanup outcomes. Ordinary terminal formatting can conceal incomplete cleanup although fields remain available. | Improve cleanup visibility while retaining primary failures and redaction. Do not print raw command output/environment. |
| MEDIUM | `child/mod.rs:72–82`, `tool/mod.rs:248–260` and `tool/process.rs:233–254` repeat and unpack/repack four cleanup fields. A new cleanup category requires coordinated structural edits. | At an approved breaking boundary, consider composing the existing CleanupError within ExecutionError. Preserve status and primary failure; coordinate Testkit's public-field consumers. |
| LOW | `docs/status/current.md` has 2,642 lines and still labels delivered 0.10.2 pending, with older pending claims back through 0.3.1 and ephemeral evidence paths. This mandatory first read obscures current authority. | Keep a short current handoff and preserve historical evidence in the established archives; retain sole artifacts rather than deleting history. |
| LOW | `ic-host-tools/Cargo.toml:35–80` places generic examples behind candid-extraction; inspect_response uses response decoding/fs/hash but no extractor. Feature ownership in examples does not match the lean production graph. | Move examples with one clear owner; use explicit development dependencies for compositions. Do not remove required-features without fixing dependency availability. |

## Intentional choices to revisit only by decision

- **Two limits types:** `tool/mod.rs:25–65` duplicates the byte fields, with
  Duration versus Option<Duration>. [#31](https://github.com/dragginzgame/ic-host-tooling/issues/31)
  deliberately retained finite probes while allowing long builds. Arbitrary
  `AdmittedTool::run` and capture helpers remain finite-only. Convergence could
  simplify a future 0.11 API, but should explicitly preserve bounded admission
  policy; this is not evidence that the current build deadline fix failed.
- **Blocking Drop:** `child/mod.rs:327–375,479–484` can synchronously reap under
  KillAndWait; a capture deadline does not bound final cleanup. This is documented
  and prevents abandoned children. Bounded group cleanup exists. Revisit direct
  cleanup configurability only with an owner for unreaped-child recovery; do not
  silently make Drop leak or impose an arbitrary build deadline.
- **Old release spelling:** `scripts/release/adapter.sh:39–59` admits `## [Draft]`
  although current governance requires numbered pending notes. The Shared
  finalizer also supports it. Rejecting it locally is a possible small cleanup;
  review source-owned upstream behavior and interrupted-release obligations first.

## Change rehearsals

1. Forward output with bounded retained diagnostics: the semantic owner is
   read_chunk/exchange; limits, observer documentation, evidence and consumer
   error rendering must change together. Replacing the IO engine is unnecessary.
2. Add a cleanup failure category: today child, tool error representation,
   conversion and public-field consumers need matching changes. Composition
   removes the repeated structural declaration, not the obligation to report it.
3. Demonstrate response-only usage: adjust example ownership/development inputs
   and required-features, retaining the current normal dependency-graph check.
   Widening production dependencies just to run an example would be a regression.

## Retain and verification limits

Keep the four crate boundaries, descriptor/path custody distinctions, named and
closed-writer producer contracts, before/after-publication phases and separate
cleanup evidence. The small macOS FFI function has a fixed buffer and documented
safety boundary; its presence alone is not a reason to add a bindgen dependency.
The isolated expect asserts a fixed two-PID buffer fits c_int, not attacker input.
Keep source provenance and release/publication recovery evidence. Per-run tool
rehashing is documented drift detection; no measurement justifies removing it.
Polling and large modules were inspected but no observed failure or measured
bottleneck supports a new event framework or a size-only refactor.

Existing tests/CI results were not rerun or relabelled as this audit's evidence.
Reviewed existing issues #28/#31 to distinguish intentional observer/deadline
contracts from accidental leftovers. No API removal or next-version implementation
is selected by this report; those are maintainer decisions in #46.
