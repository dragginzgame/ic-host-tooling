# Current handoff

## Pending 0.12.4 — Binaryen 133 and Shared Tooling 0.3.4

Released base: 0.12.3 (`aec863191b3c96ef879e59af701cbe1451595f25`).
Package versions remain 0.12.3. The next patch selects Binaryen 133 under the
existing compatible setup/check contract; library APIs, Cargo selections and
all other tool pins are unchanged. Explicit `make install-ic-tools` prepares
this selection; offline checks never install it.
[Host #53](https://github.com/dragginzgame/ic-host-tooling/issues/53) owns adoption
and remaining native qualification.

Canonically refreshed the unchanged 90-file selection from clean isolated Shared
`169d77b8440568c5200eede971625126181f7bb2`, verified against remote main.
Only the selected IC-tool and supported-host guidance changed, plus snapshot
provenance. AGENTS, README and local host guidance identify the new baseline.
That documentation-only refresh initially retained 132; the maintainer then
explicitly authorized moving Host's separately owned IC pin matrix to 133.
The original adoption evidence and incoming handoff are preserved at
`/tmp/host-shared034.LQxPTh/`.

All three retained official 133 archives match their reviewed digests. Actual
Linux setup downloaded and verified the new bundle, and complete offline
`make tools-check` passes. The old 132 selection was refused before setup;
subsequent setup reuse passes with curl blocked and keeps the same new bundle.
All files in the prior bundle remain unchanged by checksum.
Host's selected 133 executable passes optimization at -O3, -Os and -Oz and
Node 24.21.0 execution of the unchanged committed Shared fixture, including
IC-shaped imports/exports, integer boundaries and reply bytes. The O0 input
also executes correctly. Host has no product optimizer invocation or canister
build; archive member names containing 132 remain synthetic fixture inputs.
No Node dependency or optimizer gate was added to Host.

Evidence, selected binary/pin/fixture hashes and the incoming handoff are retained
in `/tmp/host-binaryen133.kpkq5f/`. Snapshot, dependency declaration, documentation
link and diff checks pass. Preserved digests confirm Cargo.toml, Cargo.lock and
the host/Cargo/release pin files are unchanged.
No Rust build, full delivery gate, commit, push or release ran. Native Host
macOS 15 Intel/Apple Silicon setup/check remains pending against delivered
source; verifying macOS archive bytes on Linux does not qualify execution.
[Shared #102](https://github.com/dragginzgame/shared-tooling/issues/102) retains
independent producer optimizer qualification. The existing Host CI matrix
prepares and admits the selected pins on all three native hosts.

## Released 0.12.3 — Shared Tooling 0.3.3 adoption

The following records original pre-release evidence; the maintainer subsequently
released this source as 0.12.3. Current adoption is recorded above.

Released base: 0.12.2 (`e1ef99e6a4c6d05f0b0d8364f8586c6cc358dadc`).
Package versions remain 0.12.2. The next patch adopts compatible jobserver and
fixture-completion fixes. Library APIs, Cargo.lock and all tool pins are unchanged.
[Host #52](https://github.com/dragginzgame/ic-host-tooling/issues/52) owns this batch.

Exported reviewed Shared 0.3.3 `d63f0cfaba8ab2961d6012064adbf051c1898bc1`
from a clean isolated checkout, excluding the sibling's dirty source and pins.
The 90-file snapshot adds the README freshness task referenced by the catalog;
this activates no schedule or gate. Shared formatting/LOC/Cargo recipes now
preserve jobserver descriptors and standalone tool includes enforce execution
admission. Shared fixtures require explicit completion before successful cleanup.
Host does not select the validation runner changed in Shared 0.3.3, so that
unused runner remains excluded. Host-owned fixture corrections remain local.

Real parallel command checks exposed an additional closed-descriptor warning
through Host's own validation wrappers. Those Cargo callers now preserve the
descriptors, and the actual Host Make fixture checks their handoff and unsafe-mode
refusal. The corrected parallel command/dependency/publication/release checks
pass without jobserver warnings. Actual parallel formatting prerequisites and
complete offline tool checks pass. Bash 3.2 Make, setup/installer, formatting,
LOC, release and retention fixtures pass; the canonical upstream retention
fixture also passes under Bash 5 and Bash 3.2 using the isolated source checkout.
Setup, Git and publication effects in fixtures are substitutes. ShellCheck,
snapshot integrity, local links and unchanged-input digests pass.

Evidence and the incoming handoff are retained in `/tmp/host-shared033.IGaURK/`,
including the initial warning and corrected runs. A standalone evidence-helper
invocation refused missing fixture arguments; its required host/IC caller runs
passed with disposable consumer fixtures. No tool download, build, full
delivery gate, commit, push, release or new native macOS qualification ran.
Released 0.12.1 [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38050076519)
now passes Linux/MSRV and both native macOS hosts, satisfying Host #48–#50.
Released 0.12.2 [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38051203749)
passes Linux/MSRV with both macOS jobs queued; #51 remains open for qualification.
Exact-source [Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38052409053)
is queued. These observations do not qualify pending Host source.

## Released 0.12.2 — Shared Tooling 0.3.1 setup admission

The following is original pre-release evidence; current delivery and qualification
are recorded above.

Released base: 0.12.1 (`e5ecfa06c14d144cfeb85ea89d65906b1bf81636`).
Package versions remain 0.12.1. The next patch adopts compatible setup checks
and diagnostics; library APIs, lockfile and all tool pins are unchanged.
[Host #51](https://github.com/dragginzgame/ic-host-tooling/issues/51) owns adoption.

Canonically exported the same 89-file selection from clean isolated Shared
`fa452afaa5012866eb1c20820dfa8038c106e7ec`, matching remote main. Nine selected
files changed. The moving sibling's uncommitted fixes were excluded. Setup now
checks IC platform/catalog and Rust/Cargo availability before common downloads,
then preserves host, IC, Rust and local release-tool order. Host's actual Make
fixture covers both preflight failures before installation, each later failure,
parallel ordering and the existing unsafe-mode refusal.

Focused Linux checks pass: actual IC/Rust preflight, complete installed-tool
offline admission, snapshot/dependency/link checks, ShellCheck, Host Make,
common aggregate, host/IC/Rust installer, selected release-tool and release
adapter fixtures. Host Make, aggregate, host/IC installer and both release
fixtures also pass on Bash 3.2 on Linux. Installer, Git and release effects in
fixtures are substitutes. Logs and the incoming handoff are retained in
`/tmp/host-shared031.bRHSeI/`. No download, build, full delivery gate, commit,
release or new native macOS qualification ran.

Released 0.12.1 [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38050076519)
has passing Linux and MSRV jobs; Apple Silicon is running and Intel queued at
inspection. Host #48–#50 remain open for that qualification. Exact-source
[Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38049622600)
is also incomplete. Neither result qualifies these pending Host bytes.
Shared #99 and #103 fixes are still uncommitted and remain upstream obligations.

## Released 0.12.1 — explicit release prerequisite and Cargo jobserver access

The following is the original pre-release evidence; delivery and current
qualification are recorded above.

Released base: 0.12.0 (`1ba4591868a83b367d56bae9e3c213418c66a192`).
Package versions remain 0.12.0. The next patch repairs the implicit release-tool
prerequisite and Make descriptor forwarding; library APIs and MSRV are unchanged.

Host pins cargo-edit 0.13.13 and installs only `cargo-set-version` through the
shared selected-Cargo installer. Ordered setup/check aggregates include this
local extension. The release adapter admits its receipt, digest and version,
then uses its absolute executable path without global PATH fallback.
[Host #48](https://github.com/dragginzgame/ic-host-tooling/issues/48) owns this fix.
Host-owned Cargo, release and publication recipes now preserve jobserver
descriptors while retaining the shared unsafe-Make-mode refusal
([Host #50](https://github.com/dragginzgame/ic-host-tooling/issues/50)).
The four Host fixtures require explicit completion before successful cleanup;
unset-variable failures on Bash 3.2 and other premature exits retain evidence
and return failure ([Host #49](https://github.com/dragginzgame/ic-host-tooling/issues/49)).

Focused Linux checks pass: actual offline installation from cached dependencies,
complete `make tools-check`, parallel release-tool admission, ShellCheck,
snapshot integrity, local links, release-tool/adapter and actual Make fixtures.
Release-tool and adapter fixtures also pass with Bash 3.2 on Linux. Fixtures
substitute installation, Git and release effects; separate real-executable
qualification changed a disposable workspace and lockfile with an empty Cargo
home and no global release CLI on PATH. Evidence is retained in
`/tmp/host-release-tool-fix/` and `/tmp/host-real-release-tool.VNmbcA/`.
No package release, commit, push, full delivery gate or new native macOS
qualification ran. This is focused local evidence, not full release readiness.

Follow-up evidence in `/tmp/host-0121-fixes/`: fault injection into actual fixture
copies passes for unset variables, failing commands, explicit nonzero exits and
premature zero exits. Normal Make, release-tool, release-adapter and publication
fixtures pass under Bash 5 and Bash 3.2 on Linux. Publication descriptor checks
and unsafe-Make-mode refusal use substitutes; no registry effects or builds ran.
ShellCheck, links, dependency declarations and snapshot integrity pass.

Shared remote main remains `88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d`
at the follow-up check. The sibling has uncommitted work; no sibling files were
changed and the adopted 89-file snapshot is intact. Shared formatting and Cargo
installer jobserver fixes remain owned by
[Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99), pending a
reviewable upstream commit. Do not treat the Host recipe fix as closing it.
The same Bash 3.2 completion defect was reproduced in Shared's unchanged
formatter fixture and reported in
[Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103).
Its correction belongs upstream before canonical adoption.

## Released 0.12.0 — Shared Tooling 0.3.0 adoption

The following records pre-release adoption evidence; the package version is now
0.12.0 and the installed complete toolset passes the current offline check above.
Released-source [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38045025680)
now reports success for Linux x86-64, MSRV and macOS 15 Intel/Apple Silicon.
This qualifies released 0.12.0, not pending 0.12.1. The nine older delivered Host
issues were reconciled against their acceptance notes, current source and this
native result; downstream obligations and older run outcomes remain separate.

Reviewed remote main `88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d` and exported
the same 89-file selection from a clean isolated checkout. The dirty sibling
dashboard script was excluded. Source/tool pin catalogs, Rust APIs, Cargo.lock
and package versions were unchanged during adoption, before release preparation.

The shared setup/check aggregates now require host, IC and Cargo tools in order.
Host CI and selected release preflight use the complete aggregate, replacing
CI's separate global cargo-sort installation. Ordinary CI and standalone release
verification only check prepared tools. Actual Host Make fixtures cover parallel
ordering and stopping at each failed toolset; the release fixture covers setup
only in preflight. Setup guidance and the retained-evidence collector are aligned.
The release selected 0.12.0 because the setup-command and prerequisite contracts
changed; see [the migration notes](../changelog/0.12.md) and
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
The maintainer reported publication complete. Package versions were 0.11.0;
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

The current baseline is Shared 0.3.4, as recorded above and in AGENTS.md.
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
