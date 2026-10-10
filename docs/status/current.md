# Current handoff

## Pending 0.12.0 — Shared Tooling 0.3.0 adoption

Reviewed remote main `88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d` and exported
the same 89-file selection from a clean isolated checkout. The dirty sibling
dashboard script was excluded. Source/tool pin catalogs, Rust APIs, Cargo.lock
and package versions are unchanged; the package version remains 0.11.0.

The shared setup/check aggregates now require host, IC and Cargo tools in order.
Host CI and selected release preflight use the complete aggregate, replacing
CI's separate global cargo-sort installation. Ordinary CI and standalone release
verification only check prepared tools. Actual Host Make fixtures cover parallel
ordering and stopping at each failed toolset; the release fixture covers setup
only in preflight. Setup guidance and the retained-evidence collector are aligned.
The next notes select 0.12.0 because the setup-command and prerequisite contracts
change; see [the migration notes](../changelog/0.12.md) and
[Host #47](https://github.com/dragginzgame/ic-host-tooling/issues/47).

Focused Linux checks pass: snapshot integrity, documentation links, ShellCheck,
Actionlint, actual Host Make routing, shared aggregate ordering, host/Rust tool
installer fixtures (including evidence retention), and the release-adapter
fixtures. Host installer/evidence, aggregate, Host Make and release-adapter
fixtures also pass with Bash 3.2 on Linux. Installers, setup, Git and qualification effects in fixtures are
substituted. Actual `make tools-check` passed the installed host set, then refused
the absent IC bundle; the Cargo bundle is also not prepared. No tool download,
real setup, release, full delivery gate or new native macOS qualification ran.
Evidence and the preserved incoming handoff are in `/tmp/host-shared-030.dCQY4y/`.
The prior release's green CI below does not qualify these local changes.
Exact-source [upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38044218125)
has passing lint/security, Linux regression running and both macOS jobs queued
at inspection; upstream results remain distinct from Host adoption qualification.

## Released 0.11.0 — approved maintenance decisions

Released source: 0.11.0 (`1d768c80a5bb87e3330a6b7bacfdfc543063968f`).
The maintainer reports publication complete. Package versions are 0.11.0;
the limits, process cleanup fields and example commands changed public contracts.
[Host #46](https://github.com/dragginzgame/ic-host-tooling/issues/46) owns this batch;
[the 0.11 notes](../changelog/0.11.md) contain consumer migration instructions.

Delivered in this release:

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
remains read-only for the current Host task; follow-up proposals go to their
owning issues. Current continuation does not authorize another release.

Focused Linux qualification passed: 80 process, eight named-publication, ten Candid
and ten response-only tests; strict Clippy and Rust 1.88 all-target checks for the
three affected library packages; strict Rustdoc; moved examples compile. The
artifact JSON example passes selected Clippy/MSRV and executes in the minimal
profile; the response example executes without Candid. Feature graphs, formatting,
manifest order, local links, ShellCheck and local release-adapter fixtures pass,
including Bash 3.2/GNU Make 3.81. Release setup/qualification/Git effects were
substituted. No retained receipt uses an unnumbered Draft; latest release plan is
complete. These are the original pre-release checks; they do not describe the
later release's version and lockfile changes.

Logs are retained under `/tmp/host-011-*`, including initial compiler failures,
over-broad migrated cleanup assertions, Clippy findings and the corrected passing
runs. Exact released-source [CI run 38040092044](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38040092044)
completed successfully on 2026-10-10: Linux x86-64, MSRV and native macOS 15 on
both Intel and Apple Silicon passed. This supersedes the earlier pending native
qualification for this source, without qualifying downstream consumer changes.

Consumer migration is recorded in [Testkit #47](https://github.com/dragginzgame/ic-testkit/issues/47),
[Query #38](https://github.com/dragginzgame/ic-query/issues/38),
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6095865185),
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307#issuecomment-6095865365) and
[Toko Miner #33](https://github.com/dragginzgame/toko-miner/issues/33#issuecomment-6095865492).
Host publication is complete. Consumer adoption has independent source and
qualification evidence in those issues; several checkouts were changing during
the follow-up audit. No sibling source was modified or compiled by this Host
task. Full-output consumer contracts must be reviewed before selecting
diagnostic-only retention. The audit found remaining full-output build retention
and separate Cargo/Git probe capture paths in Testkit and Canic. Their existing
process engine can be reused without adding another Host API or a build deadline.
Prepared consumer patches and required qualification are recorded in
[Testkit #49](https://github.com/dragginzgame/ic-testkit/issues/49) and
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6096150261).
They select bounded diagnostic prefixes and complete hard-bounded probe output;
they are unapplied proposals, checked with Rustfmt and `git apply --check`, not
compiled consumer changes. Testkit advanced to released local source 0.29.0
(`e15cc2acfd9324f6877f854415005f91d169a031`) during preparation; its follow-up
diagnostic-contract change therefore belongs at the next minor boundary.

## Shared baseline and evidence

The current baseline is Shared 0.3.0, as recorded above and in AGENTS.md.
The following evidence records the earlier 0.2.13 adoption in released 0.11.0:
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

The adopted baseline requires the full delivery suite before code is declared
ready; AGENTS, README and Make help are aligned. The original adoption used
focused checks under the user-provided session restriction on full gates. The
released Host source now has passing native CI on all three required hosts, as
linked above. Upstream Shared CI and downstream adoption remain distinct evidence.

The [decision audit](../reports/audits/2026/10/10/maintenance-decisions/01/report.md)
is preserved as the original inspection evidence. The prior 2,642-line handoff
is preserved byte-for-byte in [the historical handoff](archive-2026-10-10.md).
Historical pending claims describe their original inspection time, not current
release state. GitHub issues remain the follow-up tracker.
