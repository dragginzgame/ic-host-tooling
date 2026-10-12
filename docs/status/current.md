# Current handoff

## Pending 0.15.0 — Candid/interface ownership

Released base is 0.14.10 `886a8df0424d548e08126ef8dfab889e6d8e8071`.
The checkout was clean at the start of the Candid batch. Shared 0.9.3 is now pinned
at `3619de9dc36065411b8e6521f8695e9395a0e0ea`, matching GitHub main at inspection.
Canonical preview/refresh from a clean isolated source updates six payloads,
retaining all 87 roots and 99 files. The sibling's dirty release-runner and
snapshot changes are excluded. This adds bounded transient asset-download retries
to explicit Host/IC setup ([Shared #138](https://github.com/dragginzgame/shared-tooling/issues/138));
it adds no build timeout and changes no pins or existing installation requirements.
The pending Candid source, Cargo graph and local validation commands are preserved.
Adoption evidence is retained in `/tmp/host-shared093.BjjCeS/`. Full Linux
`CARGO_NET_OFFLINE=true make ci` passes for the combined source. The exact-source
download fixture passes with real curl and Bash 3.2.57 over substituted loopback
HTTP transport; it tests transient recovery, exhausted/permanent errors and
checksum refusal without downloading real tools. Host/IC installer fixtures,
snapshot verification and repeat preview also pass Bash 3.2; the preview reports
99 KEEP rows. ShellCheck and crate/Cargo/pin preservation checks pass. Existing
MSRV evidence applies to the unchanged Rust inputs. Native macOS qualification
remains separate and pending. No real tool installation or download ran.

The approved hard cut separates `candid::text` and `candid::extraction`, with no
old-path reexports. Text normalization is available without Unix/default features.
Optional `ic-interface` introduces parser-backed `candid::service::inspect` and
borrowed `interface::exported_methods` over the artifact owner's Wasm facts. It
retains names/modes, explicitly projects oneway to update exports and rejects
imports. It never expands argument types into a second presentation tree. Caller
source-byte/method bounds are not hard parser heap, stack or CPU quotas.

The parser is absent from response/text and default extraction graphs. With
defaults disabled, interface inspection adds no Host filesystem/process owner.
The new focused profile is part of `make ci`. Full Linux
`CARGO_NET_OFFLINE=true make ci`, focused interface tests/feature graphs, Clippy
and Rust 1.88 all-target/all-feature checks pass. Logs use `/tmp/host015-*.log`.
The optional `candid_parser 0.4.1` graph adds 112 locked packages while preserving
all 42 existing package versions/checksums (evidence in
`/tmp/host015-review/lock-preservation.log`). Package release versions remain
0.14.10. The parser is an opt-in dependency cost, not a default graph expansion.

Breaking notes select 0.15.0. In-repository callers are updated. Canic's
normalization calls and IcyDB's extraction imports need the documented new paths;
their artifact inventories can adopt the shared service/export owners. Consumer
source remains read-only. Expected methods, internal-export policy, endpoint
presentation and build/cache orchestration remain with their existing owners.
[Host #71](https://github.com/dragginzgame/ic-host-tooling/issues/71) owns
qualification. [Canic #458](https://github.com/dragginzgame/canic/issues/458) and
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307) remain the consumer
adoption owners. Migration comments are prepared in `/tmp/host015-review/` but
initially required explicit cross-repository issue approval. The maintainer
approved both and the comments are now posted:
[Canic migration](https://github.com/dragginzgame/canic/issues/458#issuecomment-6127109367),
[IcyDB migration](https://github.com/dragginzgame/icydb/issues/307#issuecomment-6127109791).
Sibling files remain unchanged; actual consumer deletions/qualification have not
run. Native macOS qualification is pending. Released 0.14.10
[CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38264225245)
is queued; it does not qualify these pending 0.15 inputs. No open Host PR was found.
No sibling edit, staging, package-version bump, commit, push or release ran.

## Historical preparation for 0.14.10 — Shared 0.9.2 admission and lock-test isolation

Adopted committed Shared Tooling 0.9.2
`c118062a070a2e238f1eb3989b6fd700f369ce1e`, matching GitHub main at inspection,
through canonical preview/refresh from a clean isolated checkout. This advances
the pending 0.9.1 adoption below; all 87 roots and 99 payloads remain. Five
payloads change relative to 0.9.1. Checksum manifests reject NUL before hashing;
Host, IC and Rust installers reject supplied/resolved CR/LF consumer paths before
configuration or setup. No pin change or reinstall is required.

Actual Host checksum fixtures and exact-source upstream path-admission fixtures
pass genuine Bash 3.2.57 on Linux. Snapshot verification and repeat preview pass
under that shell, with 99 KEEP rows. ShellCheck passes. Evidence is retained at
`/tmp/host-shared092.glI8OP/`.

The first full Linux gate passed tooling checks but exposed an unchanged private
lock test's immediate reopen failure (`WouldBlock`). The exact test passed alone.
As with the neighboring lock tests, its close/reacquisition assertions now run in
an isolated subprocess to avoid parallel fork/exec descriptor inheritance
([Host #70](https://github.com/dragginzgame/ic-host-tooling/issues/70)). All existing
assertions remain; production locking is unchanged. The parallel filesystem suite
and its Rust 1.88 check pass. Full Linux `CARGO_NET_OFFLINE=true make ci` now passes
(`ci-final.log`); the failed attempt remains in `ci.log` and focused results in
`fs-tests.log`/`fs-msrv.log`. Documentation links, diff and Cargo/pin preservation
checks pass. All other crate files retain their pre-adoption bytes.

Package versions remain 0.14.9; compatible contributor tooling/test changes keep
the pending notes at 0.14.10. Released 0.14.9 passes Linux, MSRV and native macOS
15 Intel/Apple Silicon in the linked CI below; pending-source native qualification
remains outstanding. The GitHub description remains accurate. No sibling edit,
tool installation, download, staging, commit, push or release ran.

## Earlier preparation for 0.14.10 — Shared 0.9.1 installer cleanup

Released Host 0.14.9 is `bea0f3e764915bff5b8ff0c61ed36dd5596f3f43`; the working
tree was clean before adoption. Canonical preview/refresh from a clean isolated
checkout adopts Shared Tooling 0.9.1
`e8a9ea6b660f216166eb00d0af6a32462bdbda4a`, matching GitHub main at inspection.
The corrected source VERSION now appears in the snapshot. All 87 roots and 99
payloads remain; six selected/derived payloads change, with no addition or removal.

Host and IC installation now share lock acquisition, verified-cache activation and
exit cleanup. Cleanup attempts shared-lock release even when checkout cleanup
fails, preserves any original installation error, retains failed candidates and
returns failure when cleanup alone fails. No pin change or reinstall is required.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes. The exact upstream Host/IC
installer fixtures pass genuine Bash 3.2.57, including substituted cleanup/download
failures and retry/retention checks. Actual Host snapshot verification and repeat
preview pass Bash 3.2 with 99 KEEP rows. ShellCheck, links, diff and captured
crate/Cargo/pin preservation checks pass. Evidence: `/tmp/shared091-review.Nomkcl/`.
These fixture checks do not install real tools or establish native macOS evidence.

Native macOS qualification of this pending source remains outstanding.
[Released 0.14.9 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38247180562)
now passes Linux x86-64, MSRV and native macOS 15 Intel/Apple Silicon. That
released-source result does not qualify this pending adoption. The GitHub
description remains accurate. Compatible
contributor-tooling notes select 0.14.10; package versions remain 0.14.9. Rust
inputs are unchanged, so no separate MSRV rerun was needed.

The request followed creation of ic-build; target clarification had no reply, so
this update used the current ic-host-tooling workspace and left ic-build at its
existing baseline. No sibling file edit, tool installation, download, staging,
commit, push or release ran.

## Historical preparation for 0.14.9 — Shared 0.9.0 manifest reader and JSON workspace counts

Released Host 0.14.8 is `c7b884f8bce7907c2c2670c1a8a6bc7bfad30628`.
Adopted Shared's 0.9.0 commit `019ce8236cc7cee14ea5540d468b939a2053ef30`, matching
GitHub main at inspection, through canonical preview/refresh from a clean isolated
checkout. Its committed VERSION remains 0.8.4 despite the 0.9.0 commit subject and
changelog. The snapshot preserves that canonical annotation and exact revision;
[Shared #137](https://github.com/dragginzgame/shared-tooling/issues/137) records the
upstream release metadata correction. Do not claim the annotation says 0.9.0.
The prior uncommitted 0.8.4 adoption is retained. The selection remains 87 explicit
roots and 99 payloads, with no additions or retirements.

Refresh and verification share the verifier's manifest syntax reader, also exposed
through `--read-manifest FILE`. It rejects NUL bytes and malformed input before
emitting any records. Payload integrity and committed-source ownership remain
separate checks; existing manifests need no conversion
([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).

`bash scripts/dev/cloc.sh --json` emits numeric crate rows and totals only after
all counts succeed. Existing `make cloc` text output is unchanged. Fleet aggregation
remains in Shared Tooling. The guide now consistently explains derived script
companions and explicitly selected governance roots and optional functionality.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes under the new reader.
Upstream snapshot-distribution and tooling-report integration fixtures pass genuine
Bash 3.2.57, including NUL/malformed-input refusal without emitted records and
the shared reader's integration with refresh. Actual Host manifest reading,
snapshot verification and repeat preview pass under Bash 3.2, with 99 KEEP rows.
ShellCheck, documentation links, diff and crate/Cargo/pin preservation checks pass.
Evidence: `/tmp/host-shared090.NTVuzu/`. Earlier LOC evidence remains at
`/tmp/host-shared084.Mf2edf/`: Bash 3.2 fixtures passed and actual Host JSON totals
matched the text report for all four crates. Those LOC inputs are unchanged.

Native macOS qualification of this pending adoption remains outstanding.
[Released 0.14.8 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38240226634)
was queued at inspection and does not qualify these working-tree changes.
The GitHub description remains accurate. Compatible contributor-tooling notes
select 0.14.9; package versions remain 0.14.8. Crate code, dependencies, MSRV and
pins are unchanged, so no separate MSRV rerun or tool reinstall was needed.
No sibling file edit, installation, download, staging, commit, push or release ran.

## Historical preparation for 0.14.8 — Shared 0.8.3 installer and snapshot fixes

Adopted Shared Tooling 0.8.3
`8cca7e67b03ce6e701b6e7dcec2e9753eb98337e`, matching GitHub main at inspection,
through canonical preview/refresh from a clean isolated checkout. The committed
VERSION is 0.8.3 despite the source commit subject saying 0.8.2.
The prior pending 0.8.2 guidance and selection cleanup remain: 87 roots derive
99 payloads, including the Cargo-tool freshness task. Its checker stays in Shared
Tooling and runs against an explicit consumer; no Host online CI gate is added.

Installer observation paths are admitted before diagnostic writes
([Shared #136](https://github.com/dragginzgame/shared-tooling/issues/136)),
and formatting uses the selected Rust pin catalog
([Shared #135](https://github.com/dragginzgame/shared-tooling/issues/135)).
Snapshot payload replacement is atomic per file, common Make companions include
their installers and LOC helper, and successful PCRE2 checks are quiet
([Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)).
The consumer ignores temporary payloads as the updated guide requires.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes. Genuine Bash 3.2.57 checks
pass for upstream snapshot distribution (including interrupted copy and concurrent
file/index changes), Host Rust-tool installation fixtures and Make integration.
These installer/formatter effects use substitutes. Actual Host snapshot verification
and repeat preview pass under Bash 3.2, with 99 KEEP rows. ShellCheck, links,
diff and preservation checks pass. Evidence: `/tmp/host-shared083.it29Xr/`;
prior guidance-only evidence remains at `/tmp/host-shared082.kuY403/`.

Native macOS qualification of this pending adoption remains outstanding.
Released Host 0.14.7 `52535a6cb88df9a84d1cda0a20d0887c6ba0ea8e`
passes Linux/MSRV in [its CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38236092920),
with macOS ARM running and Intel queued at inspection; that run does not qualify
this pending source. The GitHub description still matches the four-library scope.

Compatible contributor-tooling notes stay 0.14.8; package versions stay 0.14.7.
Crate source, Cargo metadata and pins are unchanged. No separate MSRV rerun is
needed for unchanged Rust inputs. No sibling edit, tool installation, freshness
query, download, staging, commit, push or release ran.

## Historical preparation for 0.14.7 — derive internal shared helpers

Released Host 0.14.6 is `11f8c31b599ea81f6106225c4068d78d22fed249`, with
Shared 0.8.1 still matching remote main at inspection. The working tree was clean
at the start of this continuation; no open Host issues were found.
[Released-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38233096876)
passes Linux and MSRV, with both macOS jobs queued at inspection.

A bounded flow-convergence review under the adopted Shared method and Host
AGENTS.md found 12 internal helper selections duplicating canonical companion
ownership. Canonical preview/refresh now records 86 explicit roots; the derived
98-file payload has identical paths, digests and modes. Direct commands, local
fixture inputs, governance and maintained optional tests remain explicit.
No file or check is removed. The method, ownership map, reviewed callers and
retention decisions are retained at `/tmp/host0147-selection.eaWWsO/review.txt`.
[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131) owns the
broader entrypoint-adoption acceptance.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes. Actual snapshot verification
and repeat preview pass Bash 3.2.57, with 98 KEEP rows and no changes. Snapshot,
links, diff and preservation checks pass; all crate/script/Make/workflow/pin inputs
and Cargo metadata match the captured released baseline. Evidence is retained
alongside the review. Compatible notes select 0.14.7; package versions stay 0.14.6.
Native qualification of the pending manifest remains separate from released CI.

Two new canonical findings were inspected at exact Shared 0.8.1 source:
[Shared #135](https://github.com/dragginzgame/shared-tooling/issues/135)'s alternate
Rust pin catalog mismatch has no current Host trigger because Host uses the
default common catalog. [Shared #136](https://github.com/dragginzgame/shared-tooling/issues/136)'s
redirected observed-evidence paths apply to the unchanged installer; this root
cleanup does not repair that upstream admission gap. No duplicate implementation
or local snapshot patch was introduced. No sibling edit, installation, staging,
version bump, commit, push or release ran.

## Historical preparation for 0.14.6 — Shared 0.8.1 snapshot and installer fixes

Adopted Shared Tooling 0.8.1 `c97f9f9c7b7c706fffdcfc5925cf58759b58093f`,
matching remote main at inspection, through canonical preview/refresh from a clean
isolated checkout. It advances the uncommitted 0.8.0 adoption without requiring
a Host commit. Six selected files change; all 98 entrypoints/payloads remain,
with no addition or retirement. Repeat preview reports only KEEP. The pending
batch retains explicit roots, automatic companion selection and verified automatic
retirement; future root narrowing still requires local caller review.

Snapshot readers reject malformed fields while preserving literal filename spaces
([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).
Cargo tool installation retains original and observed build identities, reports
changed fields and preserves the prior active selection when the environment
changes ([Shared #134](https://github.com/dragginzgame/shared-tooling/issues/134)).

Full Linux `CARGO_NET_OFFLINE=true make ci` passes. The initial gate stopped at
manifest sorting because an incoming edit renamed `ic-host-tools` to `ic-dls`.
After the maintainer indicated this was unexpected, the original name was restored;
Cargo.toml now matches committed HEAD. The incoming file and failed log are retained.
The actual Host Rust-tool fixture and upstream malformed-record integration pass
genuine Bash 3.2.57 on Linux, including both identity observations in archived
failure evidence and unchanged active selections. Compiler/Cargo effects in the
installer fixture are substitutes. Host verification/repeat preview, ShellCheck,
snapshot/link/diff and preservation checks pass. Evidence:
`/tmp/host-shared081.wkh3sc/`; earlier 0.8.0 and entrypoint evidence remains at
`/tmp/host-shared080.jFnEbd/` and `/tmp/host-shared-entrypoints.Z6wXb1/`.
Crate source, lockfile, pins and package versions remain unchanged. The accidental
0.14.5 notes remain intact; pending notes stay 0.14.6 and packages stay 0.14.5.

[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131) still owns
the stale manual IC-companion paragraph; canonical copies remain intact.
[Upstream 0.8.1 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38231862048)
passes lint, with Linux running and both macOS jobs queued at inspection.
Released Host 0.14.5 passes Linux, MSRV and both macOS architectures, completing
[Host #69](https://github.com/dragginzgame/ic-host-tooling/issues/69)'s earlier
hook-admission acceptance only. Native qualification of this adoption remains
outstanding. No sibling edit, installation, download, staging, version bump,
commit, push or release ran during this 0.8.1 update.

## Released base — 0.14.5

Local HEAD and GitHub main are `bb8f1e71f3d4599618d0f4084d27fa637408f1fb`
(`Release 0.14.5`), with tag `v0.14.5`. Shared 0.6.6 adoption shipped in
0.14.4 (`f93cd8854d500616a73eda713d215a20ff0617c7`). The maintainer confirmed
that 0.14.5 was an accidental extra release; it changes only release metadata.
The changelog now records that disposition. Keep the current workspace/package
version at 0.14.5, matching the pushed release; the next compatible patch is
0.14.6 when meaningful work is ready. The snapshot remains 0.6.6.
[Released-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38225165549)
passes Linux, MSRV and both macOS architectures;
the superseded 0.14.3/0.14.4 runs were cancelled.
These Git identities do not establish crates.io publication or native qualification.
The registry checks returned HTTP 403 for all four 0.14.5 packages, leaving
publication unverified; evidence is under `/tmp/host0145-registry-current`.

## Historical preparation for 0.14.4 — Shared 0.6.6 derive-sorter fix

Adopted Shared Tooling 0.6.6 `a15eb9d406d287697df9bc8fa96db8cda6721a71`
through canonical preview/refresh from a clean isolated checkout. Three files
change within the same 98-file selection: the common tool pins and setup/hook
guidance. cargo-sort-derives 0.14.1 preserves literal `$crate` paths in direct
and conditional derives ([Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113)).
Host keeps its cargo-sort/rustfmt flow; no Host macro workaround needs removal.
This compatible contributor update selects 0.14.4 notes without changing crate
APIs, runtime behavior, MSRV, Cargo metadata or package versions.

Explicit `make install-rust-tools` prepared the new selection, and complete
`make tools-check` passes. Direct and conditional stdin examples preserve and
sort both literal macro paths with the installed 0.14.1 binary. Full Linux
`CARGO_NET_OFFLINE=true make ci` passes, including formatting/hook preservation,
release/tool fixtures, feature profiles, Clippy, rustdoc and package tests.
Evidence is retained at `/tmp/host-shared066.lVYpiL/`. Shared's frontend macro
fixture remains upstream; Host does not adopt an unused frontend test suite.

Native qualification remains outstanding. [Shared 0.6.6 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38220477204)
has both macOS jobs queued at inspection. Released Host 0.14.3
`543b137458feba0f6dfd3e458b680fb2be3d9e6d` passes Linux/MSRV in
[its CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38219424683),
with both macOS jobs queued; [Host #69](https://github.com/dragginzgame/ic-host-tooling/issues/69)
still owns that earlier adoption's native acceptance. Neither run qualifies this
unpublished Host snapshot. No sibling edit, staging, commit, push or release ran.

## Historical preparation for 0.14.3 — Shared 0.6.5 staged-hook receipt admission

Adopted Shared Tooling 0.6.5 `42f2091768b5f71c98a684d3dab563f408ca2dd9`,
matching remote main, through canonical preview/refresh from a clean isolated
checkout. Seven files change within the same 98-file selection. The staged hook
can check prepared Rust receipts from the original checkout against exported
pins and compiler selections. Corrupt/mismatched receipts, export installation
and unrelated nested selection borrowing refuse. Host's formatter prerequisites
stay unchanged; the new canonical fixture uses real Git/receipt handling with
substituted compiler/Cargo binaries. Shared's optional validation-input checkpoint
runner is outside Host's selection; no validation-flow change is claimed.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes on the preserved incoming lock,
including actual formatting/staged-hook preservation, release/tool fixtures,
feature profiles, Clippy, rustdoc and package tests. Changed Cargo-tool and
Make-command fixtures also pass genuine Bash 3.2.57. ShellCheck passes with
sourced companions followed; the initial incomplete-source invocation's log is
retained. Snapshot, links, diff and input hash checks pass. Rust source, Cargo
files, pins and release adapter remain unchanged; prior audit evidence is retained.
Existing verified tools need no reinstall. Evidence: `/tmp/host-shared065.pv3rnO/`.

[Host #69](https://github.com/dragginzgame/ic-host-tooling/issues/69) owns delivery
and matching native acceptance. Upstream [0.6.5 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38218219902)
passes lint, with Linux running and both macOS jobs queued at inspection. Released
Host 0.14.2 native results qualify the prior snapshot only. No sibling edit,
staging, package bump, commit, push, installation, online dependency preparation
or release ran. The incoming lock changes now have local full-Linux evidence,
but no native qualification of these unpublished inputs.

## Released 0.14.2 — process and consumer convergence review

Released HEAD is `fed8724e79862d8273d01823619fe648c981c019`, with adopted
Shared 0.6.4 still matching remote main. [Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38159733484)
passes Linux, MSRV and both macOS 15 architectures; #65 and #68 are closed.
The incoming Cargo.lock updates to serde_json 1.0.154, syn 3.0.8 and
unicode-ident 1.0.27 remain untouched and are not qualified by that released run.

The [bounded process convergence review](../reports/audits/2026/10/12/process-convergence/01/report.md)
found no new Host source defect or competing execution owner. Testkit 0.34.0
now has complete native CI and carries completed results directly; Canic has
removed its observer cancellation/error-rewrite workaround. Producer #66 is
complete; downstream delivery and application graph acceptance stay separate.
Six of seven inspected current sibling lockfiles contain only Host 0.14.
Toko Miner still records Testkit 0.33.8 and Host 0.13.5 alongside 0.14.2, with a
different root Testkit requirement; its existing #6 owns graph reconciliation.
No sibling source, lock or artifact was changed or compiled.

Shared #130's staged-export receipt gap still has no committed correction to
adopt. This pass changes evidence/handoff only, with snapshot, link and diff
checks; no new release entry, code change, full gate or release effect is needed.

## Historical preparation for 0.14.2 — Shared 0.6.4 common Make entrypoint

Adopted Shared Tooling 0.6.4 `6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d`,
matching remote main at inspection, through canonical preview/refresh from a
clean isolated checkout. The 98-file selection adds the common Make entrypoint,
its guide and its fixture. Host now uses that entrypoint instead of three
component includes and duplicate hook/snapshot-check recipes. Ordered release-tool
setup, direct-delivery admission, package gates and publication remain Host-owned.
Actual hook/Make/release fixture exports include the new entrypoint; Host's routing
fixture also checks hook activation and snapshot checks with substitute effects.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes, including real formatting and
staged-hook preservation, release/tool fixtures, feature profiles, package Clippy,
rustdoc and tests. Host/common Make fixtures and the actual Makefile's isolated
release-routing check also pass under genuine Bash 3.2.57. Snapshot, documentation
links, ShellCheck and diff checks pass. Rust source, Cargo metadata, tool pins and
the release adapter retain captured hashes. Existing verified tools need no
reinstall. Evidence: `/tmp/host-shared064.stLzXm/`.

The same pending patch addresses [Host #68](https://github.com/dragginzgame/ic-host-tooling/issues/68):
README setup now links to the existing Host and shared owners. Host-specific
release-tool, direct-delivery, cache and recovery details live in `docs/publishing.md`;
feature-specific checks live in `docs/hosts.md`. Product reference content stays
in the README. This documentation-only follow-up passes all local link checks,
command/source consistency review and diff checks; the previous complete CI
evidence still applies to unchanged executable inputs. Logs:
`/tmp/host0142-readme-checks.log` and `/tmp/host0142-readme-help.log`.

Released 0.14.1 `f766121c7282a78753ca32bdb4d069ba8424b517` records Shared
0.6.3 and passes [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38149531883)
on Linux, MSRV and both macOS 15 architectures. That completes
[Host #67](https://github.com/dragginzgame/ic-host-tooling/issues/67), now closed.
The same released filesystem tests also complete
[Host #64](https://github.com/dragginzgame/ic-host-tooling/issues/64)'s private
sidecar native acceptance; independent consumer custody remains consumer-owned.
The new Make adoption remains under [Host #65](https://github.com/dragginzgame/ic-host-tooling/issues/65)
and [Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130), with
delivery and native qualification outstanding. Upstream 0.6.4
[CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38155574308)
passes lint, with Linux running and both macOS jobs queued at inspection. Older
native results do not qualify this pending adoption. No sibling edit, staging,
package bump, commit, push, installation or release ran.

## Historical preparation for 0.14.1 — Shared 0.6.3 and retained validation attempts

Adopted Shared Tooling 0.6.3 `4fd979d13c32adb3cb2f51f213ebfc4f98339b3d`
through canonical preview/refresh from a clean isolated checkout, retaining the
same 95-file selection. Direct preparation can restart from clean descendant
commits with fresh validation under the canonical recovery contract. Host's
[receipt fix](https://github.com/dragginzgame/ic-host-tooling/issues/67) first
archives existing validation receipts, saved notes and logs into unique attempt
directories, verifying identical bytes before validation can append or replace
them. Unsafe entries and failed preservation refuse before the gate. Failed
validation retains the prior active receipt and notes.

During this task the maintainer released 0.14.0 at
`4723befe8d65c760c00e05c42652759d691fbd5e`. That release and its process APIs
are preserved; this compatible tooling update selects pending 0.14.1. Rust
source, released Cargo metadata and pins retain their captured hashes. Existing
verified 0.6.1 tool installations need no reinstall. No sibling edit, staging,
package bump, commit, push or release was performed by this adoption task.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes. The Host adapter and canonical
runner fixtures pass under genuine Bash 3.2.57; the exact upstream real-Git
tracking/restart fixture also passes under Bash 3.2 in disposable repositories.
Host adapter fixtures substitute Git/setup/gates while exercising actual evidence
preservation for successful and failed same-candidate revalidation, unsafe paths
and failed copies. ShellCheck, snapshot and preservation checks pass. Evidence:
`/tmp/host-shared063-adopt.D8GMrT/`.

Native acceptance remains open in [Host #65](https://github.com/dragginzgame/ic-host-tooling/issues/65)
and #67. Released 0.14.0 [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38146341421)
passes Linux/MSRV with both macOS jobs queued at inspection. Upstream 0.6.3
[CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38145666246)
passes Linux/lint, with Intel macOS running and ARM queued. Neither run qualifies
the pending Host adoption. Producer recovery acceptance remains with
[Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126).

## Historical preparation for 0.14.0 — completed execution and fallible observers

Implemented the approved [process contract change](https://github.com/dragginzgame/ic-host-tooling/issues/66)
on released Host 0.13.5 `b32021c67515f0a0dcf35f1dd0b2029b9599ac75`, preserving
the incoming Shared 0.6.2 adoption. The combined unpublished batch selects
0.14.0 because capture, communication, admitted runs and completed Git/Candid
evidence now expose `CompletedExecution` with required status. Ordinary nonzero
and signalled exits return completion; explicit `require_success()` owns strict
admission. Version, Git, Candid and the named-producer fixture retain success
checks before interpretation/publication. Partial failure evidence stays optional-status.

Output observers return `io::Result<()>`; their first error retains its stream,
original sink cause, admitted bytes, truncation and separate cleanup failures.
One IO engine still owns pipe closure and child cleanup. Nonzero retained leaders
are cleaned/reaped and cannot hand off. Successful retained leaders, optional
deadlines, panic cleanup and caller-owned recovery retain their contracts.
The pending 0.13.6 notes moved into the sole 0.14.0 entry; published history,
package versions, dependencies, pins and staging are unchanged.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes, including 84 process tests,
22 IC-tool tests, package Clippy/rustdoc, all feature profiles, real formatting
and tooling fixtures. Affected process/tools checks also pass Rust 1.88. New
cases cover signalled completion, nonzero retained-leader descendant cleanup,
both observer streams, zero/hard retention, original sink causes and failed
cleanup evidence. Native 0.14 macOS qualification remains outstanding. Released
0.13.5 now passes Linux, MSRV and ARM macOS; Intel is running at inspection,
which does not qualify the unpublished source.

Concrete consumer source patches are recorded under
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6109845273),
[Testkit #61](https://github.com/dragginzgame/ic-testkit/issues/61) and
[Query #46](https://github.com/dragginzgame/ic-query/issues/46).
They are Rustfmt-clean and pass apply checks against the reviewed source, but
remain unapplied and uncompiled. Consumers own dependency selection after Host
publication and their own native qualification; Testkit's public Host reexports
require its minor boundary. No sibling source/artifact edit or build ran.
Evidence, initial compiler/lint failures and final passing logs are retained in
`/tmp/host014-work/`. No staging, manifest bump, commit, push or release ran.

## Historical preparation for 0.13.6 — carried into 0.14.0

Adopted Shared Tooling 0.6.2 `7f1ca16b0dfa98021d2671017ecb7de767409b0e`
through canonical preview/refresh from a clean isolated checkout. The same
95-file selection changes four files: release conflict diagnostics, its fixture,
release recovery guidance and Audit/Machete invocation examples. Notes select
compatible 0.13.6; Cargo versions, dependencies, pins and library APIs are unchanged.
Existing verified 0.6.1 installations pass admission without reinstalling.
Host does not journal every preparation mutation; missing journal evidence alone
cannot satisfy the new maintainer recovery procedure's no-effect proof.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes. The release-runner substitute
fixture also passes under genuine Bash 3.2.57. ShellCheck passes with the optional
upstream PR helper resolved from the exact source checkout; the initial focused
invocation reported only its absent optional source. Snapshot, links, formatting
and preservation checks pass. Evidence: `/tmp/host-shared062-adopt.DTEiG9/`.
The incoming handoff is preserved below and in that evidence directory. No
staging, package version, commit, release, installation or sibling edit ran.

Native qualification remains separate: at inspection, released Host 0.13.5 and
upstream Shared 0.6.2 pass Linux, with ARM macOS running and Intel queued.
[Host #65](https://github.com/dragginzgame/ic-host-tooling/issues/65) retains
native acceptance; those runs do not qualify the unpublished Host adoption.

## Released 0.13.5 — native acceptance and consumer follow-through

Local released HEAD is `b32021c67515f0a0dcf35f1dd0b2029b9599ac75`, clean at
inspection. Its [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38142886238)
passes MSRV, with Linux running and both macOS jobs queued. Shared main remains
adopted 0.6.1 `731909525038c58d6f80b498e80f8630b11c685d`; no refresh is needed.
The completed 0.13.4 native failure logs are now available: both Apple jobs stop
after the first common Cargo path in the old fixture, consistent with the padded
count defect corrected in 0.13.5. This is old-source failure evidence, not
qualification of the correction. Host #64/#65 retain native acceptance.

A bounded read-only `audits/code-hygiene.md` review under this Shared baseline
and the Host overlay traced private-sidecar admission into Canic. Canic now
delegates admission to Host 0.13.4 and reports nine focused Linux acceptance
cases, preserving command locking and descendant custody. Its dirty consumer
test still calls rustix's Apple-excluded `mkfifoat`; the selected rustix 1.1.5
source confirms the compile gap. [Canic #515](https://github.com/dragginzgame/canic/issues/515)
contains a minimal portable FIFO fixture patch, passing apply/format checks in
a disposable copy. No consumer source or artifact was changed or compiled.
This review covered the adoption boundary and fixture portability, not a full
sibling audit or runtime/lifecycle qualification. No new Host production defect
was established; no next release notes, build, setup or broad gate were needed.

## Historical preparation for 0.13.5 — Shared 0.6.1 failure and macOS fixture corrections

The sidecar batch is released as Host 0.13.4
`46483b96c21419aa41ee2cbcdfa2e8efbbfdab21`; the incoming checkout was clean.
Adopted latest committed Shared 0.6.1
`731909525038c58d6f80b498e80f8630b11c685d`, matching remote main, through canonical
preview/refresh from a clean isolated checkout. The same 95-file selection adds
explicit Cargo setup failure propagation, lock-guarded activation and the numeric
padded-count fixture correction. The upstream tag-maintenance fixture is outside
Host's selection. Notes select compatible 0.13.5; no package version, dependency,
pin or library API changed. Released sidecar source retains its incoming hashes.

Canonical `CARGO_NET_OFFLINE=true make install-tools` completed from the managed
cache under the changed build-policy digest. Old binary hashes are preserved.
A disposable second checkout reuses the exact same physical generations for all
four Cargo tools offline. Its first attempt refused a shared preparation lock;
the attempt is retained, and retry succeeded after observing the lock released
and completing CI. No lock was removed or bypassed. No downloads, manual binary
relocation or artifact cleanup ran. Staging remains empty; the index file's
metadata hash changed without a staged content change.

Full Linux `CARGO_NET_OFFLINE=true make ci` passes, including the 76 filesystem
tests, feature profiles, package Clippy/rustdoc, real formatting/hooks and tooling
fixtures. Host/IC/Cargo fixtures also pass under genuine Bash 3.2.57; the Cargo
fixture always exercises padded counts. Snapshot/declaration/link checks,
ShellCheck and diff checks pass. Evidence: `/tmp/host-shared061.iVAfU0/`.

The source correction for [Shared #124](https://github.com/dragginzgame/shared-tooling/issues/124)
is now adopted. [Host #65](https://github.com/dragginzgame/ic-host-tooling/issues/65)
remains open for delivery and native qualification. Upstream 0.6.1 Linux passes;
both native macOS jobs are queued. Released Host 0.13.4 uses the older snapshot:
Linux/MSRV pass, ARM failed and Intel is running. Its selected failed log is not
yet exposed while the run is incomplete, so this inspection does not attribute
its cause. These old-source results do not qualify the pending update. No sibling
source edit, staging, package bump, commit, push or release ran.

## Historical preparation for 0.13.4 — private sidecar admission

Released base is Host 0.13.3 `66c5df4c13ec500b4935d919420360bb0b8986ef`;
the incoming checkout was clean. Its CI passes Linux/MSRV, with both macOS jobs
queued at inspection. Those results do not qualify this unpublished change.

Implemented [Host #64](https://github.com/dragginzgame/ic-host-tooling/issues/64)
with the additive `durable::open_private_regular_lock_file`. New sidecars use
0600 before umask in existing parents; existing bytes, inode and permissions
are preserved. It returns an unlocked descriptor through the same no-follow,
nonblocking, close-on-exec regular-file admission as the existing durable opener.
The existing parent-creating/publication APIs retain their behavior. Direct
creation adds no durability syncs, retry policy or lifecycle owner. Callers retain
lock acquisition and descriptor/descendant custody. Pending 0.13.4 is compatible;
package versions, dependencies, pins, snapshot and real index are unchanged.

All 76 filesystem tests and examples pass on Linux. The new cases exercise
child-only permissive umask, missing parents, unsafe entries, original I/O errors,
descriptor admission, retained inode/bytes/mode, CLOEXEC and caller-owned
acquisition/contention/release. Rust 1.88 MSRV checking for ic-host-fs passes.
Complete `CARGO_NET_OFFLINE=true make ci` passes, including formatting/hooks,
tooling/publication fixtures, feature profiles, Clippy, rustdoc and package tests.
Evidence is retained through `/tmp/host-private-lock-current`.

[Host #65](https://github.com/dragginzgame/ic-host-tooling/issues/65) remains blocked
on [Shared #124](https://github.com/dragginzgame/shared-tooling/issues/124): remote
Shared main is still adopted `75180980a0c4e4252d8a915aaecfaefd92bf9a4e`, with no
committed padded-count correction to adopt. The snapshot is intact. Native
macOS qualification of both changes remains outstanding. Canic adoption is
tracked under [Canic #513](https://github.com/dragginzgame/canic/issues/513);
its source and lockfile remain untouched. No download, staging, package bump,
commit, push or release ran.

## Historical preparation for 0.13.3 — Shared 0.6.0 Cargo storage adoption

Released base is Host 0.13.2 `b7ef6ecff3be330c16c0f013e567a553b8517972`.
Its exact-source CI passes Linux, MSRV and both native macOS 15 architectures;
[Host #63](https://github.com/dragginzgame/ic-host-tooling/issues/63) is closed.
Those results qualify the preceding snapshot only.

Adopted requested Shared 0.6.0
`75180980a0c4e4252d8a915aaecfaefd92bf9a4e` from a clean isolated checkout through
canonical preview/refresh. The required Cargo-build companion brings the snapshot
to 95 files. Pending notes select compatible 0.13.3; no library API, dependency,
MSRV, package version or tool pin changed. Host's release adapter already uses
the installer's returned path. Its fixture now tests the Host-owned invocation,
returned path, version and failure contract, while adopted shared fixtures own
receipt/storage admission and retention. The retention harness follows that split.

Canonical `make install-tools` completed with pinned package downloads and
builds in the default per-user store. Offline setup in a disposable second
checkout selects the same physical generations for all three common Cargo tools
and cargo-set-version. Prior local executable/receipt hashes remain unchanged.
Cargo files, pins and real index retain their incoming hashes; no artifact cleanup
or manual binary relocation ran.

Complete Linux `CARGO_NET_OFFLINE=true make ci` passes, including tool admission,
formatting/hooks, tooling/publication fixtures, feature profiles, package Clippy,
rustdoc and all-target tests. Snapshot/declaration/link checks, ShellCheck and
actionlint pass. Host/IC/Cargo fixtures and the Host retention harness also pass
on Linux under genuine Bash 3.2.57. Evidence: `/tmp/host-shared060.yd8mhI/`.

The new canonical Cargo fixture fails with macOS-style padded `wc -l` output.
A disposable four-comparison numeric correction passes under Bash 3.2.57 with
the same padded-output substitute. The snapshot is intact; the proposed patch
and evidence are reported in [Shared #124](https://github.com/dragginzgame/shared-tooling/issues/124).
Native macOS readiness remains blocked on the committed correction and actual
consumer CI. [Host #65](https://github.com/dragginzgame/ic-host-tooling/issues/65)
tracks delivery/qualification. No sibling edit, staging, package bump, commit,
push or release ran.

## Historical preparation for 0.13.2 — Shared 0.5.0 host/IC storage adoption

Adopted requested committed Shared 0.5.0
`a306180ce534227fcb98b882fba9a526d5d747e0` from a clean isolated checkout.
The first preview correctly reported two required companions; adding the store
helper and its fixture through canonical refresh produces a complete 94-file
snapshot. The incoming unpublished 0.13.2 batch is preserved and updated. Library
APIs, MSRV, dependency selections, package versions and tool pins are unchanged.

Canonical `make install-tools` prepared checksum-pinned host/IC bundles in the
default per-user store, `/home/adam/.local/share/shared-tooling/tools`.
Checkout `.tools/host` and `.tools/ic` retain their command paths via references
to exact generations. A disposable second checkout selected the same physical
bundles with curl disabled. Complete offline admission passes. Original physical
local bundles remain present with matching executable hashes; no manual relocation
or cleanup occurred. Cargo slots/build output remain local under `.tools/rust`.
The old policy-only storage description below is historical.

Complete Linux `CARGO_NET_OFFLINE=true make ci` passed. The adopted IC installer
fixture was then added to Host's native tooling gate beside the host installer,
closing an observed coverage gap. Its Make routing fixture was updated, and the
complete affected `tooling-command-check` gate passed again. Unchanged package,
formatting and publication evidence is reused. Host/IC storage fixtures and the
amended Make fixture pass under genuine Bash 3.2.57. The storage tests include
reuse, pin/platform isolation, corruption, locks, retained failures and actual
collector behavior. Snapshot/declaration/link checks, ShellCheck, actionlint and
diff checks pass. Cargo files, pins and real index retain incoming hashes.
Evidence: `/tmp/host-shared050.qPhAha/`.

[Host #63](https://github.com/dragginzgame/ic-host-tooling/issues/63) remains open
for delivery and native acceptance. Shared 0.5.0's native matrix is still pending;
Host 0.13.1 passes Linux/MSRV/Apple Silicon with Intel running at inspection, and
does not qualify these pending bytes. Setup downloaded pinned host/IC assets
through the canonical installers. No sibling source edit, package version edit,
staging, commit, push or release ran.

## Historical preparation — Shared 0.4.2 and shared-storage ownership

Adopted the explicitly requested Shared 0.4.2
`8db24059613c63ba9be36d4225fb74f598f49df2` through preview and canonical refresh
from a clean isolated checkout. The same 92-file selection is complete. Compared
with the incoming 0.4 implementation, only five selected policy/documentation
files and snapshot metadata change; executable installer bytes are unchanged.
The canonical version annotation now correctly says 0.4.2, resolving Shared #119.

The maintainer clarified that contributor tooling does not break this library's
consumer contract. Pending notes are corrected to 0.13.2 and consolidated into
`docs/changelog/0.13.md`; the unpublished 0.14 detail file is retired. Published
0.13.1 history, package versions, dependencies, APIs and MSRV remain unchanged.
The earlier minor-release classification below is superseded by this decision.

Shared Tooling owns the planned per-user verified executable store under
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120).
It is not implemented in 0.4.2. Host retains the canonical repository-local
`.tools` storage and `.tools/rust/common/bin` selection until upstream delivers
and qualifies shared storage. No files were relocated or deleted from `.tools`,
no sibling PATH or independent installer/store was introduced, and no installation
was repeated for these unchanged installer bytes.

The complete local `CARGO_NET_OFFLINE=true make ci` gate passes on Linux: offline
tool admission, snapshot/dependency/link checks, formatting and real hook checks,
release/tool/publication fixtures, feature profiles, package Clippy/rustdoc and
all-target tests. This supersedes the earlier focused-only validation limit.
Cargo files, pins, installer bytes, original in-place tools/receipts and real index
retain incoming hashes. Evidence: `/tmp/host-shared042.pLii0F/`; earlier installed
slot/reuse and Bash 3.2 evidence remains `/tmp/host-shared040.9885UM/`.

[Host #63](https://github.com/dragginzgame/ic-host-tooling/issues/63) retains
delivery/native qualification. Released Host 0.13.1 passes Linux, MSRV and Apple
Silicon, with Intel queued at inspection. Shared's preceding 0.4.1 hosted native
retention fixture failed before producing its qualification marker; that unchanged
upstream-only fixture is not selected by Host, and its repair remains under
[Shared #118](https://github.com/dragginzgame/shared-tooling/issues/118).
No upstream or local Linux evidence is relabelled as new native macOS acceptance.
No sibling edit, download, staging, package-version edit, commit, push or release ran.

## Historical preparation — earlier Shared 0.4 adoption and version classification

At the maintainer's explicit adoption request, refreshed the same 92-file snapshot
from clean isolated Shared commit `e0acfdbb2cef9c29462686d3454c1bd993963627`,
matching remote main. Its read-only preview required no companion additions.
The commit is named 0.4.0 but committed VERSION remains 0.3.12; the exporter-owned
annotation is preserved, with [Shared #119](https://github.com/dragginzgame/shared-tooling/issues/119)
tracking correction. The incoming handoff edit is retained below.

Shared Make, hook/check callers, installer, documentation and evidence action now
share the versioned-slot common-tool contract. Host's local CI PATH selects
`.tools/rust/common/bin`; the existing selected cargo-edit adapter and pin remain
unchanged. The developer setup/PATH hard cut selects pending 0.14.0 notes; package
versions, Cargo selections, four library APIs and release defaults are unchanged.
No compatibility fallback or artifact cleanup was introduced.

Real Linux common-tool installation succeeded using cached locked dependencies
with network disabled. The full offline `tools-check` passes. Repeated setup with
a refusing Cargo substitute succeeds without dispatch, proving admitted-slot reuse.
Old in-place tool files/receipts, Cargo files, pins and the real index retain their
incoming hashes. Canonical installer and hook fixtures plus actual Host formatting
pass on current Bash and genuine Bash 3.2.57. Host's retained-failure fixture passes,
covering its selected cargo-edit adapter and real metadata/substituted effects.
The actual adopted evidence collector retains sample slot receipts/build files
without following an external symlink. Snapshot/declaration/link checks, ShellCheck,
actionlint and diff checks pass. Evidence: `/tmp/host-shared040.9885UM/`.

[Host #63](https://github.com/dragginzgame/ic-host-tooling/issues/63) remains open
for delivery and native acceptance. Shared's exact-source macOS jobs are queued;
Host 0.13.1 passes Linux/MSRV, with Apple Silicon running and Intel queued at the
final check. These earlier-source results do not qualify the new snapshot.
No full local gate, native macOS execution, network download, sibling edit, staging,
package version edit, commit, push or release ran. No named function/type was removed.

## Released 0.13.1 — next adoption preparation

Local HEAD and remote main are released 0.13.1
`95fa4828996a6116e84b5ed01909b6e8590a355d`; the incoming checkout was clean,
with no active Cargo/rustc process. Its
[CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38127809605)
passes Linux and MSRV; both macOS jobs are queued at inspection. The preceding
release's native acceptance does not qualify this source.

Shared main remains adopted 0.3.12 `52e846369c39a1da371eac47ff97d94da925217e`.
The installer/PATH hard cut is still dirty upstream. Prepared the concrete Host
owner map under [#63](https://github.com/dragginzgame/ic-host-tooling/issues/63#issuecomment-6107727678):
one consumer-owned CI PATH selection, coordinated canonical snapshot callers and
evidence collection, and retention of the already-delegating cargo-edit selector.
No dirty upstream helper was executed or adopted. Implementation awaits reviewed
committed upstream source and native acceptance; no next release is selected yet.

A bounded code-hygiene review covered executable resolution and exact version/byte
admission, including search precedence, typed errors, explicit environment,
pre-execution rejection and later identity rechecks. No new defect was established.
Nine resolution and five version-admission tests pass on Linux with locked/offline
dependencies and all features. This is module evidence, not a full process audit,
installer qualification or native macOS result. Evidence: `/tmp/host-post0131.rlWyuk/`.
Cargo files, tool pins, snapshot and real index retain incoming hashes. Only this
handoff changes; no library change, sibling edit, installation, download, full
local gate, commit, push or release occurred.

## Historical preparation for 0.13.1 — Host fixture path stability

The bounded code-hygiene review of Host-owned fixture roots reproduced four
failures with an existing caller-relative TMPDIR: Make loses its event path,
release tools lose their selector/log paths, the release adapter's nested metadata
probe cannot create its temporary directory, and publication loses substitute
command resolution. Physical fixture roots and an anchored inherited TMPDIR now
preserve the selected location through directory changes. The retention harness
uses the same anchoring and executes the real fixtures through relative symlinks,
including its existing wrong-status and premature-completion failures.
[Host #62](https://github.com/dragginzgame/ic-host-tooling/issues/62) owns delivery
and native qualification of this compatible addition to 0.13.1.

The extended retention regression passes on Linux under current Bash and genuine
Bash 3.2.57. ShellCheck, links and diff checks pass. Release/publication effects
are substituted; metadata uses the locked offline workspace. The deliberate
newline-named publication checkout remains covered. Failed pre-fix attempts and
passing logs are retained at `/tmp/host-relative-fixtures.QTPEV6/`.
The incoming Shared adoption, Cargo files, pins and real index are preserved.
No Rust build, full local gate, native macOS run, sibling edit, download, commit,
push or release occurred. No function, method or type was removed.

## Pending 0.13.1 — formatting-adoption temporary paths

Released Host 0.13.0 `7c58c92c72789139ec460475c429f9090581008f` is now
local HEAD and remote main; the incoming working tree was clean and no Cargo/rustc
process was active. Its [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38122069573)
passes Linux, MSRV and both native macOS 15 architectures. The native logs confirm
the affected tooling fixtures and 73 filesystem tests pass; issues #59 and #60
are closed with exact-source evidence. The earlier delivery observations below
are historical, not current checkout status.

Adopted committed Shared 0.3.12 `52e846369c39a1da371eac47ff97d94da925217e`
through its canonical exporter from a clean isolated checkout. Only the formatting
adoption checker and its regression fixture change among the same 92 selected
files; dirty sibling governance edits are excluded. The checker anchors inherited
TMPDIR before changing directories. The compatible tooling fix selects pending
0.13.1, with no library, Cargo selection or tool-pin change.
[Host #61](https://github.com/dragginzgame/ic-host-tooling/issues/61) owns delivery
and native acceptance separately from the released batch.

Canonical hook regressions pass on Linux under current Bash and genuine Bash 3.2.57.
The actual Host formatting adapter passes under both shells from an external caller
with a relative symlinked temporary root. Snapshot/declaration/link checks, ShellCheck and diff
checks pass. Cargo files, pins and the real index retain incoming hashes.
Evidence: `/tmp/host-shared0312.f8CgSp/`. No full local gate, native macOS execution,
download, sibling edit, commit, push or release ran.

A bounded read-only convergence review used the adopted
`audits/flow-convergence-and-duplication.md` method and `docs/extraction.md` ownership
contract. The six sampled sibling manifests select Host 0.13; no Rust caller of
the removed lock-progress wrapper remains in their crate trees. Canic's owning
[migration evidence](https://github.com/dragginzgame/canic/issues/458#issuecomment-6106919245)
reports passing focused checks; its adaptation remains dirty. Testkit's lock
waiting and Memory's publication paths already delegate to Host. Query's sampled
hashing uses sha2 directly or constructs application-specific identity encodings;
neither warrants a generic Host replacement. Host's hashing writer and chunk
digests already converge through their bounded writer/reader owners. No additional
API extraction or removal is justified by this scope. Sibling compilation, runtime
qualification and unrelated process/publication families were outside this pass.

## Historical adoption — Shared 0.3.11, hard cut prepared locally

Remote Host main remains released 0.12.10
`5074de1792f920112c6539629605a6624f573863`, whose native CI is fully green.
Local HEAD is still preparation commit `3326dfe`; the approved 0.13 hard cut
described below remains uncommitted. The maintainer's report of a push does not
match this remote's source identity. No additional API removal was needed.

Adopted committed Shared 0.3.11 `db143ae9ae140eb66826c413360723d23c078f66`,
verified against remote main and exported from an isolated clean checkout.
The 92 selected files are byte-for-byte unchanged from the previous snapshot;
only its revision/version metadata advances. The canonical annotation now correctly
reports 0.3.11, resolving [Shared #114](https://github.com/dragginzgame/shared-tooling/issues/114).
The other upstream change affects an unselected validation-runner test.

Snapshot, declaration, documentation-link and diff checks pass. The hard-cut
source/tests, Makefile, Cargo files, pins and real index retain incoming hashes,
so the preceding 73-test filesystem, Clippy and rustdoc evidence remains applicable.
No additional compilation or full local gate was needed. Canic's migration remains
prepared under its owning issue; no sibling files were changed. Pending release
notes remain 0.13.0. Evidence: `/tmp/host-shared0311.UEoMi1/`.
No install, download, commit, push or release ran.

## Pending 0.13.0 — caller-owned lock progress

The maintainer approved the proposed public-API hard cut. Local base is the clean
preparation commit `3326dfe` above released 0.12.10; no 0.12.11 release tag exists.
The undated 0.12.11 notes and Shared qualification work now belong to the single
pending 0.13.0 entry. Package versions, lockfile, tool pins and release defaults
remain unchanged; actual release preparation must select the minor increment.

The fixed-cadence pathname progress wrapper is removed without a compatibility
alias. Consumers compose the admitted regular descriptor opener with the existing
descriptor wait, choosing their own polling/report policy. The cross-process test
uses that composition and retains callback-error, nonblocking-contention, held-lock
and drop/reacquisition assertions. Existing descriptor-owner tests remain intact.
The migration notes describe error projection, timing and descriptor lifetime;
no lock-file, journal or recovery-state reset is required.
[Host #60](https://github.com/dragginzgame/ic-host-tooling/issues/60) tracks delivery
and native qualification; [#59](https://github.com/dragginzgame/ic-host-tooling/issues/59)
retains the separate Shared adoption acceptance in this minor-release batch.

All 73 filesystem library tests and its example targets pass on Linux with locked
offline dependencies. Selected-package Clippy and rustdoc pass with warnings denied;
snapshot integrity, local links, formatting and diff checks pass. No full local
CI/release gate or native macOS test ran. Evidence: `/tmp/host-lock-policy-cut.WsLViQ/`.

Canic's current build-reuse lock source was preserved, including unrelated dirty
work in its checkout. The prepared patch keeps its existing timing and cancellation
semantics, passes apply/format checks, and is posted under
[Canic #458](https://github.com/dragginzgame/canic/issues/458). It can use the existing
0.12 APIs before a dependency upgrade. It is not applied or consumer-build-qualified.
No sibling mutation, download, installation, commit, push or release was performed.

## Historical preparation — Shared 0.3.10 adoption (carried into 0.13.0)

Released Host 0.12.10 `5074de1792f920112c6539629605a6624f573863` matches remote
main. Its [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38120032068)
now passes Linux, MSRV and both native macOS 15 architectures. The preceding
0.12.9 run also completed on all hosts; its native logs were reviewed against
the earlier fixes. Issues #52–#56 and #58 are closed with source-bound acceptance.
No earlier cancelled run was relabelled as passing.

The clean checkout adopted Shared commit `37af29b2f8fb4532744b600c5f85ce716a8b57cf`
through the canonical exporter from an isolated clean source. This is the requested
0.3.10 commit, but its committed VERSION is still 0.3.9; the canonical snapshot
annotation preserves that value. [Shared #114](https://github.com/dragginzgame/shared-tooling/issues/114)
tracks the producer metadata discrepancy. The 92-file selection is unchanged.
The adopted host/release fixtures normalize alias/relative temporary paths;
new derive-macro guidance needs no Host source workaround because Host has no
affected macros and does not run that formatter in its formatting path.

Host's native tooling gate now invokes the already-selected host-tool fixture,
closing its previously missing CI route. The existing release-command route stays
in place. Both canonical checks pass on Linux with alias/relative temporary roots
under current Bash and genuine Bash 3.2.57; the amended Host Make routing fixture
also passes on both. The first relative-root probe used a nonexistent path and
was refused; its log is retained separately from the corrected passing probes.
Snapshot, dependency declarations, links, ShellCheck and diff checks pass.
[Host #59](https://github.com/dragginzgame/ic-host-tooling/issues/59) owns delivery
and native acceptance of this compatible 0.12.11 batch. No full local gate ran.

The requested hard-cut review inspected public lock/process/admission boundaries
and read-only callers in Canic, Testkit, Query, Memory, IcyDB and Toko Miner.
The fixed-cadence progress-lock helper has one observed production consumer in
Canic; moving its timing policy there is a possible future minor-release cut.
Byte-writer helpers and distinct process ownership/admission contracts have active
callers and already share their underlying engines. No public API was removed or
breaking change selected. Canic's in-progress edits were left untouched.

Evidence: `/tmp/host-shared0310.5KfKGs/`. Cargo files, tool pins and the real index
retain incoming checksums. No active Cargo/rustc process was present. No sibling
source, installation, download, commit, push, publication or release was performed.

## Historical preparation for released 0.12.10 — Shared Rust tool context

Released Host 0.12.9 `8cc85c53cdd59701a7352f566425b3faf195a37b` matches remote
main; the incoming checkout was clean, with no active Cargo/rustc processes.
Its [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38073002651)
passes Linux, MSRV and Apple Silicon, including the repaired macOS fixture checks.
Intel is still running at inspection. Earlier checks are not relabelled as green.

Adopted committed Shared 0.3.9 `1ed2ca523186641e0e0cdf77e3cb10814c22a430`,
verified against remote main, using the canonical exporter from an isolated clean
checkout. The 92-file selection is unchanged. The Rust installer now establishes
one consumer working directory for preflight, setup and offline checks; caller
catalog and consumer lockfile resolution remain explicit. Only its fixture and
setup guidance also change among selected files. The unrelated Shared validation
output runner is not selected. Compatible tooling behavior selects pending 0.12.10.
[Host #58](https://github.com/dragginzgame/ic-host-tooling/issues/58) tracks this
adoption independently of [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101).

The shared Rust installer fixture and Host release-tool/retention fixtures pass
on Linux with current Bash and genuine Bash 3.2.57. Actual offline common and
release-tool admission accepts the existing installations. No new dependencies,
tools or toolchains are fetched or installed. Snapshot integrity, declaration
checks, local links, ShellCheck and diff checks pass. Cargo files, pins and the
real index retain their incoming checksums. Evidence: `/tmp/host-shared039.B4nRrH/`.
No full local gate, sibling edit, commit, push or release ran. Native qualification
of this new snapshot remains separate from 0.12.9's previous snapshot results.

## Historical preparation for released 0.12.9 — macOS fixture portability and evidence capture

Released base remains 0.12.8 `c810c299acd0c41001a046c8e8ad0ee6daceefc0`.
Its [Apple Silicon job](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38068134110/job/114259932124)
passed setup, formatting hooks and release-adapter checks, then failed the Host
fixture-retention check. Linux and MSRV pass; Intel remains queued at inspection.
[Issue #57](https://github.com/dragginzgame/ic-host-tooling/issues/57) owns the
failure, local repair and remaining native acceptance.

The release-tool fixture now compares physical consumer paths and numeric line
counts. The neighboring Make fixture had the same padded-count defect and is
repaired too. Maintained coverage runs the real release-tool fixture through a
symlinked temporary root and padded counts, requiring both successful completion
and retained evidence on a deliberately wrong child status. Native CI selects
the shared collector's existing `portable-fixtures` directory as `TMPDIR`.
The compatible validation-only batch selects pending 0.12.9; no library contract
or package version changes.

Focused release-tool, retention and Make fixtures pass on Linux with current
Bash and genuine Bash 3.2.57. Old release and Make fixtures fail the new inputs;
repaired fixtures pass. The actual workflow snippet, with substituted failing
Make, preserves exit 37; the actual shared collector archives its failure marker.
ShellCheck, actionlint, snapshot integrity, local links and diff checks pass.
Evidence: `/tmp/host-0129-portability.gpGbOU/`. These results do not establish
native macOS qualification; no full CI/release gate was run locally.

The incoming Shared 0.3.8 adoption, `serde_json` 1.0.152 lockfile selection, real
index and all pins were preserved. No active Cargo/rustc process was present.
No sibling changes, downloads, commit, push, release or publication were performed.

## Historical maintenance review — filesystem and process boundaries

Local HEAD remains released 0.12.8
`c810c299acd0c41001a046c8e8ad0ee6daceefc0`. The maintainer reports the work
live, but the Shared 0.3.8 documentation adoption below is still uncommitted in
this checkout; its incoming bytes were verified unchanged before this handoff
update. Remote Shared main still matches the adopted
`67285b28a98b7c4211ad32de726709d4e87edea4`. No new Host PR or issue appeared;
the current native CI run remains queued and earlier superseded runs cancelled.

The bounded code-hygiene review covered regular/private/no-follow reads,
missing-path and executable resolution, lock waiting, durable staging identity
and publication errors, bounded process capture and owned-child cleanup/reaping.
Admission, mutation order, retained causes, cancellation and ownership remain
distinct at their existing owners. No new actionable defect or extraction was
established. The selected library checks pass on Linux with all features and
locked/offline dependencies: 73 filesystem tests and 80 process tests.
This does not establish native macOS qualification or a full delivery gate.

Evidence and incoming dirty work: `/tmp/host-fs-process-review.NCFn51/`.
Cargo selections, pins, the real index and the incoming Shared adoption are
preserved. Only this handoff changes in this pass; no new release candidate,
production change, sibling edit, install, full suite, commit, push or release
was justified or performed. The open GitHub issues retain their native acceptance
obligations; no unavailable/cancelled check was treated as success.

## Maintenance review — released 0.12.8, Shared 0.3.8

Released source: `c810c299acd0c41001a046c8e8ad0ee6daceefc0`; the maintainer
reports 0.12.8 live. The checkout was clean and no Cargo/rustc process was active.
Exact-source [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38068134110)
passes Linux and Rust 1.88 MSRV; both macOS jobs were queued at inspection.
The new hook gate is delivered and runs in native CI. Its remaining native
acceptance stays with [#55](https://github.com/dragginzgame/ic-host-tooling/issues/55);
queued or cancelled earlier checks do not provide that evidence.

Adopted committed Shared 0.3.8 `67285b28a98b7c4211ad32de726709d4e87edea4`,
verified against remote main, through the canonical exporter from an isolated
clean checkout. The upstream dirty changelog is excluded. The 92-file selection
is unchanged; selected changes are guidance for coordinated package archive
qualification and the fleet report's timezone prerequisite. Host's scripts,
package versions, lockfile and tool pins remain unchanged. No demonstrated
same-version packaging failure calls for adding a local archive qualifier here.
This governance-only adoption does not create another pending release entry.

A bounded review using the shared code-hygiene method covered artifact read,
write, copy, hashing, chunking, gzip and archive boundaries: byte allowances,
short/invalid I/O counts, completion, typed causes and publication ownership.
No new actionable defect was established. All 65 artifact library tests pass
with all features and locked/offline dependencies on Linux. This is a selected
package check, not a full gate or native macOS qualification.

Read-only consumer samples show Canic already using `gzip_matches`,
`chunk_digests` and `BoundedWriter`, and Testkit using `hash_gzip` for admitted
archive/executable identity. Their policy and publication remain consumer-owned;
no new extraction was justified by these samples. Sibling dirty work was left
untouched, with source identities and selected file hashes retained as evidence.
Snapshot integrity, dependency declarations, documentation links and diff checks
pass. Cargo, pins and the real index match incoming checksums.
Evidence: `/tmp/host-post0128.9X4GlI/`. No full suite, installation, publication,
sibling edit, commit, push, release or CI dispatch ran.

## Historical preparation for released 0.12.8 — native hook qualification

Released base: 0.12.7 (`7c31035ccf9922f914ba6c63096f2437a30e1e03`).
Its [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38065200009)
passes Linux and MSRV; both macOS jobs were queued at inspection. Shared remote
main remains the adopted 0.3.7 `34e5ad7aac3599306c9572bb547f2239d09df1a3`.
The checkout was clean, with no active Cargo/rustc processes or open Host PRs.

[Host #55](https://github.com/dragginzgame/ic-host-tooling/issues/55) identified
that native CI did not execute its focused hook acceptance. The new
`make formatting-hook-check` target runs canonical Shared hook regressions and
the shared adoption checker using Host's real formatting inputs. Native CI calls
it on every required host. Two canonical companions are added to the snapshot
through the clean exporter, bringing its selection to 92 files at the same
revision. Host owns only input selection; no hook or shared fixture is forked.
This compatible validation improvement selects pending 0.12.8. Cargo package
versions/lockfile, library code, tool pins and local hook activation remain intact.

The new target passes with `make -j4` on current Bash and genuine Bash 3.2.57 on
Linux, including the actual failed-observation cases and real Cargo sorting and
rustfmt. No builds or tool installations are involved. Native macOS qualification
still requires execution of this added target on delivered source; a green older
workflow cannot prove a check it did not run. The expanded failure-retention
fixture passes on both Bash profiles; existing Make routing, ShellCheck,
snapshot integrity, documentation links and diff checks pass. Incoming index,
Cargo files and tool pin checksums match after qualification.
Evidence and preserved inputs: `/tmp/host-0128-hook.2OOTqO/`.

The old 0.12.5 run used the previous concurrency identity and remained queued.
After verifying source ancestry and retained workflow checks, cancellation was
completed under the maintainer's earlier superseded-run authorization. The current
0.12.7 run was preserved. No full gate, sibling mutation, commit, push or release
ran. Earlier sections are historical preparation evidence.

## Historical preparation for released 0.12.7 — Shared and portable assertions

Released base: 0.12.6 (`5f356ef`). The canonical exporter refreshed the same
90-file selection from clean, isolated Shared 0.3.7
`34e5ad7aac3599306c9572bb547f2239d09df1a3`, verified against remote main.
Current baseline references identify that source. Shared fixes mandatory Bash
3.2 assertions in selected fixtures, the Cargo installer and evidence action.
Host 0.12.6 already applies the adopted newest-run CI policy; its matrix remains.

[Host #57](https://github.com/dragginzgame/ic-host-tooling/issues/57) tracks this
adoption and the matching consumer-owned repair. Host's Make, release,
publication and retention fixtures and native CI admission now explicitly reject
failed mandatory comparisons. Intentional predicates and completion guards keep
their contracts. The compatible tooling correction selects pending 0.12.7;
library source, Cargo versions/lockfile and tool pins remain unchanged.

Before repair, the real release-tool fixture accepted a substituted exit 36
where it required 37 on Bash 3.2, reported success and removed its evidence.
The maintained regression now requires failure and retained evidence. The old
native CI snippet also accepted wrong OS, architecture and macOS version
observations on Bash 3.2; the repaired snippet rejects each while accepting
valid inputs on both Bash profiles. These are controlled Linux substitutions,
not native macOS evidence.

All 13 affected fixture entrypoints pass on current Bash and genuine Bash 3.2.57
on Linux. Host/IC installer fixtures also exercise the evidence fixture and actual
composite collector with their disposable consumers. An extra standalone call
to that companion omitted its required arguments and returned usage; its proper
caller-driven cases passed on both profiles. ShellCheck, actionlint, snapshot
integrity, complete offline tool admission, dependency declarations, local links
and diff checks pass. Package and pin checksums match the incoming checkout.

Evidence: `/tmp/host-shared037.H69Dyc/`; original false-success fixture:
`/tmp/host-fixture-retention.CjPRUu/`. Delivery and native macOS qualification
remain separate under #57. No full gate, install, download, sibling edit,
commit, push or release ran. Earlier handoff sections below are historical
evidence; their queue and pending-release observations are not current status.

## Historical preparation for released 0.12.5 — paths and source observations

Released base: 0.12.4 (`5400f159474cebac1ec7ae7c8763abfd258bde03`).
[Host #54](https://github.com/dragginzgame/ic-host-tooling/issues/54) is implemented
locally: the shared pathname validator rejects NUL before parent creation,
normalization, staging or producer/admission callbacks. The same typed
`BeforePublication` / `InvalidInput` result is retained. This is a compatible
validation-order fix, so the pending candidate is 0.12.5; package versions,
lockfile and tool pins remain unchanged.

[Host #56](https://github.com/dragginzgame/ic-host-tooling/issues/56) is also
implemented locally. Host's release and publication adapters check observation
status before accepting source/version output. Failed Git, version-reader and
lockfile observations stop guarded effects; failed diff producers propagate
through the hash pipeline. Saved receipts and publication evidence are preserved.
This repairs existing guards without changing successful command contracts,
so it extends the same compatible 0.12.5 candidate.

Both command fixtures failed against the old guards and pass after repair on
Bash 5 and Bash 3.2 on Linux. They cover initial and final observations, failed
readers with plausible stdout, empty failed clean-status output, preparation,
receipt replacement and committed-release recovery. Cargo metadata is real and
locked/offline; setup, qualification, Git and upload effects are substituted.
ShellCheck, documentation links, snapshot integrity and diff checks pass.
Evidence and preserved incoming work: `/tmp/host-0125-observation-fix/`.

[Host #55](https://github.com/dragginzgame/ic-host-tooling/issues/55) is now
adopted locally from committed Shared 0.3.6
`0604bfd730ec7ec288cd2cfdad217a0d42bf256b`, verified against remote main.
The canonical exporter refreshed the same 90-file selection from an isolated
clean checkout. Six selected files changed: the hook, formatting adoption checker,
Cargo installer and its fixture, and their guidance. AGENTS and current local
guidance identify the new baseline; no vendor file was patched independently.
The adopted hook rejects failed index observations
([Shared #106](https://github.com/dragginzgame/shared-tooling/issues/106));
the checker retains evidence after premature completion
([Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103)).
The new optional lockfile-selected Cargo installer mode is qualified by its
substitute fixture ([Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96));
Host's release tool retains its explicit consumer pin.

Host's actual hook passes initial, post-snapshot and post-format failure injection,
normal refresh, mismatch and preservation cases on Bash 5 and Bash 3.2 on Linux.
The real Host formatting adoption check, Cargo installer and canonical upstream
retention fixtures pass on both Bash versions. The release-tool fixture and
actual complete offline tool admission also pass. The initial deliberately
unsorted manifest fixture attached a comment to a different dependency; corrected
fixture input preserves that attachment and passes. Production source was unaffected.
ShellCheck, snapshot integrity, dependency declarations, local links and diff
checks pass. Checksums preserve the incoming filesystem and adapter fixes,
package selections and all pins. Evidence and incoming work are retained at
`/tmp/host-shared036.6eY7la/`. Native macOS qualification and delivery remain
separate; no install, download, full gate, commit, push or release ran.

The new public-boundary regression failed before the repair because a missing
parent was created, then passed after it. It covers all six pathname APIs,
both streamed publication modes, and NUL in filenames and parent components.
All 73 filesystem library tests pass on Linux, including existing native-name,
directory-suffix, publication, cleanup and lock cases. Selected all-target,
all-feature Clippy with warnings denied, formatting, documentation links and
diff checks pass. Original reproduction: `/tmp/host-pathname-admission.NwZvzO/`;
before/after logs and preserved incoming handoff: `/tmp/host-0125-nul-path/`.

Released 0.12.4 [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38055667305)
passes Linux and MSRV; both macOS jobs remain queued. This does not qualify the
pending fix. The other open issues retain their native acceptance obligations;
no additional implementation failure was established. No full delivery gate,
native macOS execution, sibling mutation, dependency update, commit, push or
release ran. The incoming handoff edit is preserved below as historical evidence.

## Previous maintenance check — released 0.12.4

The following records the preceding inspection, before the pending repair above.

Released source: `5400f159474cebac1ec7ae7c8763abfd258bde03`; the maintainer reports
0.12.4 live. The checkout was clean at inspection. Shared remote main remains the
adopted 0.3.4 `169d77b8440568c5200eede971625126181f7bb2`.
Exact-source [Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38055667305)
is queued; delivery does not establish native qualification.

A bounded review of process capture, child ownership, executable resolution and
Git observation found no new actionable defect. The selected process library's
80 tests pass on Linux with the locked dependencies and Rust 1.99.0. This covers
the existing cancellation, no-deadline, bounded output, cleanup and retained
ownership regressions; it is not a full delivery gate or native macOS evidence.
Method: shared [code hygiene](../../audits/code-hygiene.md), current snapshot and
AGENTS overlay; product contracts remain in [extraction](../extraction.md).
Evidence and inspected source/consumer identities: `/tmp/host-0124-process-review/`.

Read-only consumer inspection confirms Testkit's bounded Cargo diagnostics and
complete probe captures are implemented; its remaining acceptance is owned by
[Testkit #49](https://github.com/dragginzgame/ic-testkit/issues/49).
Canic's working tree still has unbounded Git/Cargo provenance captures covered
by the existing proposal in [Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6096150261).
Both use Host 0.12 requirements; requirements alone do not prove qualification.
Dirty sibling work was preserved; no sibling edits or builds ran. No new Host
API or numbered release candidate is justified by this inspection. Only this
handoff changes; package versions, lockfile, pins and library source remain intact.

## Released 0.12.4 — Binaryen 133 and Shared Tooling 0.3.4

The following preserves the original pre-release evidence. The maintainer
subsequently released it as 0.12.4; current delivery and qualification are above.

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
