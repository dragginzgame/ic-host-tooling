# Current handoff

## Pending 0.11.0 — approved maintenance decisions

Released base: 0.10.2 (`6b755763aca71b7ea8c3de40edf032093dfdc62c`).
Package versions remain 0.10.2. The next notes select 0.11.0 because the limits,
process cleanup fields and example commands change public contracts.
[Host #46](https://github.com/dragginzgame/ic-host-tooling/issues/46) owns this batch;
[the 0.11 notes](../changelog/0.11.md) contain consumer migration instructions.

Implemented locally:

- One output-limits API with optional deadlines and per-stream hard overflow or
  truncation. Observers can forward all output with bounded or zero retention.
- Complete-output checks before version, Git and Candid interpretation.
- Composed process cleanup evidence and visible secondary cleanup diagnostics
  for process execution and staged filesystem publication.
- Generic examples moved to artifact/filesystem/process owners; response example usable
  without Candid. Production crate boundaries remain unchanged.
- Numbered release notes required by the local release adapter; retained receipt
  validation and numbered interrupted-release recovery remain supported.

The user explicitly left blocking child-drop behavior unchanged. Sibling source
is read-only for this batch. No commit, push, release, publication or package
version change is authorized by this repair request.

Focused Linux qualification passed: 80 process, eight named-publication, ten Candid
and ten response-only tests; strict Clippy and Rust 1.88 all-target checks for the
three affected library packages; strict Rustdoc; moved examples compile. The
artifact JSON example passes selected Clippy/MSRV and executes in the minimal
profile; the response example executes without Candid. Feature graphs, formatting,
manifest order, local links, ShellCheck and local release-adapter fixtures pass,
including Bash 3.2/GNU Make 3.81. Release setup/qualification/Git effects were
substituted. No retained receipt uses an unnumbered Draft; latest release plan is
complete. Cargo.lock and package versions are unchanged.

Logs are retained under `/tmp/host-011-*`, including initial compiler failures,
over-broad migrated cleanup assertions, Clippy findings and the corrected passing
runs. Native macOS qualification and full CI/release gates remain unrun for this
source. Linux results do not qualify either required native macOS architecture.

Consumer migration is recorded in [Testkit #47](https://github.com/dragginzgame/ic-testkit/issues/47),
[Query #38](https://github.com/dragginzgame/ic-query/issues/38),
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6095865185),
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307#issuecomment-6095865365) and
[Toko Miner #33](https://github.com/dragginzgame/toko-miner/issues/33#issuecomment-6095865492).
These are pending adoption after Host publication; no sibling source was modified
or compiled. Full-output consumer contracts must be reviewed before selecting
diagnostic-only retention.

## Shared baseline and evidence

The adopted Shared baseline is now 0.2.13,
`5864f468d39f8f9d1bd26fca1afe0e20f25f1b5e`, verified against remote main and exported
from a clean isolated checkout. The existing 89-file selection is unchanged.
[Host #45](https://github.com/dragginzgame/ic-host-tooling/issues/45) owns this local
adoption. Supplied/resolved LF/CR directories are rejected before snapshot export
or verification; selected Cargo-tool failures now identify the exact selection.
Host does not use the installer's versioned selected-CLI mode in its library gates;
the optional installer is qualified with substituted Cargo. Existing host setup,
early offline gate checks and independently selected tool versions are retained.

Focused Linux adoption checks pass: exact snapshot integrity, actual Host ordinary
path and LF/CR/alias controls, upstream distribution fixtures, Rust-tool installer
fixtures, actual Make routing, local release adapter, ShellCheck and links.
The Host path, installer, Make and release checks also pass with Bash 3.2/GNU Make
3.81. Setup/download/release effects in fixtures are substitutes. Evidence is in
`/tmp/host-shared-0213/`; incoming Rust source, artifacts, release-adapter changes,
historical handoff and audit evidence were preserved by digest checks.

The adopted baseline now requires the full delivery suite before code is declared
ready; AGENTS, README and Make help are aligned. This update used focused checks
under the user-provided session restriction on full gates. Full consumer delivery
and native macOS checks remain outstanding. Exact-source upstream
[CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38039035514)
was queued when inspected; upstream execution does not qualify this consumer.

The [decision audit](../reports/audits/2026/10/10/maintenance-decisions/01/report.md)
is preserved as the original inspection evidence. The prior 2,642-line handoff
is preserved byte-for-byte in [the historical handoff](archive-2026-10-10.md).
Historical pending claims describe their original inspection time, not current
release state. GitHub issues remain the follow-up tracker.
