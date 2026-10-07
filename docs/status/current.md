# Current handoff

## Latest qualification: 2026-10-07, after 0.4.5

Released 0.4.5 is `93a905b048bcaa2a0aed4214ac2f13f065dc2905`.
[Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37645681743)
passes Linux/MSRV. Both macOS architectures now compile the filesystem library,
pass its strict Clippy and run its tests; the u32/u16 defect is resolved. They
then fail the unconditional non-UTF-8 filename success assertion with EILSEQ,
after 57 other filesystem tests pass. Native logs are retained at
/tmp/ic-host-046-arm.log and /tmp/ic-host-046-intel.log.

The compatible **0.4.6** draft addresses
[Host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19) with a
fixture repair and API documentation clarification. Native rename/link operations
independently establish whether the same filesystem accepts the filename bytes.
Both replace and create-only Host calls must agree: accepted names retain exact
bytes/contents, while rejected names preserve native EILSEQ in BeforePublication
with successful cleanup and no output/staging left behind. No platform branch
skips the test or translates the name. Existing invalid-component, invalid-mode
and non-directory checks still assert rejection before producer invocation.

The issue's initial no-producer wording does not describe native final-name
rejection: the producer writes the valid staging entry before the filesystem
admits the destination at publication. The new test verifies that ordering and
cleanup rather than changing production policy. BeforePublication does not mean
the callback had no effects. This distinction is recorded on #19 and in Rustdoc.

Focused locked/offline Linux checks pass: 59 filesystem tests, strict filesystem
Clippy and Rust 1.88, following explicit offline cache preparation. Logs are in
/tmp/ic-host-046-*.log. Linux exercised the accepting-filesystem branch; the
rejecting branch still requires exact committed native macOS CI. Production
code, dependencies, package versions and Cargo.lock are unchanged. No function,
method or type was removed; the filename case moved into its own expanded test.

Issue review confirms the 0.4.5 contribution adoption is committed and the
Darwin compiler defect is fixed. [#17](https://github.com/dragginzgame/ic-host-tooling/issues/17)
and [#18](https://github.com/dragginzgame/ic-host-tooling/issues/18) were closed
with exact release evidence; overall native acceptance remains tracked by #19.
Query's latest feedback reports prepared 0.48.0 typed descriptor adoption and
173 focused Linux tests, with its own capability/permission/schema rules kept
local. That is consumer-reported Linux evidence, not native acceptance or a
reason to remove Backup's separate crash-barrier obligations. Shared Tooling
remains at the already-adopted committed 0.1.20 revision. No sibling files,
Git refs, publication or CI runs were changed; no broad local gate ran.

## Previous qualification: 2026-10-07, after 0.4.4

Released 0.4.4 is `d7785db667db0c2fd5135f4aaa0bf02a847c3126`. Its failed
atomic push was subsequently recovered: remote main and the annotated v0.4.4
tag were verified at that commit, and the saved release plan is complete.
[Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37642928105)
passes Linux/MSRV but fails both macOS architectures compiling the durable writer.
`Mode::from_raw_mode` requires Darwin's u16 mode while the public options hold
u32. Both native logs are retained under /tmp/ic-host-045-macos-*.log. This
supersedes the earlier queued/running native status; the shared scratch-directory
fixture is no longer the failing step.

The compatible **0.4.5** draft repairs
[Host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18) with checked
Apple conversion after existing permission admission. Linux retains u32.
Public options, invalid-mode rejection and publication error state are unchanged;
no truncating cast or panic is introduced. The existing admission regression
now rejects special permission bits and values exceeding u16 at both entry
points, before parent/staging creation or producer dispatch.

Focused locked/offline Linux checks pass: 58 filesystem tests, strict filesystem
Clippy and Rust 1.88. Dependency cache preparation, sorting, formatting, snapshot,
pins and documentation checks pass. Logs remain under /tmp/ic-host-045-*.
Neither native macOS target is installed locally. The Apple branch still needs
execution/compilation in exact committed Intel and ARM macOS CI; the passing
Linux checks do not establish that qualification. Query and Blob adoption remain
coordinated on #18 until the repair is delivered and natively qualified.

[Host #17](https://github.com/dragginzgame/ic-host-tooling/issues/17) is prepared
through canonical adoption of committed Shared Tooling 0.1.20,
`3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`, whose
[upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37641211708)
passes. A clean private clone was exported, adding rules/contributions.md for
68 verified snapshot entries. Its path is recorded in /tmp/ic-host-045-shared-path.
The baseline, linked rules/guides and local AGENTS overlay now agree: scoped
commit/PR requests authorize their documented Git workflow; ordinary fixes remain
local, and merge/integration-branch/release effects retain separate authority.
Historical handoff descriptions of old prohibitions remain historical evidence.
No Git effects are implied by adoption itself. Policy links were checked locally;
runtime shared scripts, tool pins and CI workflows did not change in this refresh.

No function, method or type was removed. Package versions, dependencies and
Cargo.lock remain unchanged. No sibling edit, broad local gate, tool download,
commit, push or publication occurred. The fixes remain in the working tree;
#17 and #18 stay open through their delivery/qualification requirements.

## Previous qualification: 2026-10-07, after 0.4.3

The maintainer reports 0.4.3 live. The clean starting HEAD is
`644d49c096ae05c2e17e1b6aacf14770988c5cf6`, with finalized root notes and package
versions at 0.4.3. [Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37639415895)
has passed Linux/MSRV; native ARM macOS is running and Intel macOS is queued at
the latest inspection. That run does not qualify the new uncommitted work.
Shared Tooling's latest local committed revision remains the adopted 0.1.19
`a06e4719e3839b8eefcfb88ec8923aa88eb63ccc`; no new snapshot adoption is needed.

The next compatible batch targets **0.4.4**, adding gzip identity and exact
comparison helpers for [Host #12](https://github.com/dragginzgame/ic-host-tooling/issues/12).
`hash_gzip` identifies a strict single member; `hash_gzip_or_raw` additionally
selects raw input unless its first two bytes are gzip magic. Both return decoded
or raw `ArtifactIdentity` without a full decoded allocation, with independent
input/payload bounds. They accept an already-resident borrowed slice, not an
unbounded file or network stream. They do not validate Wasm. `gzip_matches`
compares exact bytes against a borrowed expected slice, retaining existing
decoded-limit versus malformed-gzip distinctions. All reuse the existing
decoder framing checks, bounded traversal, hashing and `MatchingWriter`.

Canic's inspected representation module is clean at
`4bee0f8f69c80825e41d6100bde7575c5002980d`. Its `qualify_representation` currently
allocates decoded Wasm only to compare it with the raw artifact. A concrete
`gzip_matches` replacement and preserved error projection are recorded on
[Canic #458](https://github.com/dragginzgame/canic/issues/458). No Canic files
were edited or consumer graph compiled. Toko's shell hashing still requires
consumer-owned CLI integration; adding a Rust helper does not delete it.

Focused offline Linux qualification passes: 61 artifact unit tests, seven
consumer-contract tests, 33 minimal-profile tests, strict artifact Clippy,
Rust 1.88 compilation and Rustdoc. Existing gzip/archive behavior passes after
sharing its framing owner. New cases cover exact input/payload limits, raw/gzip
identity equivalence, mismatch followed by malformed CRC, truncation, member
concatenation and trailing bytes. The initial Clippy const-function finding and
corrected run are retained under /tmp/ic-host-044-*.log. Cache preparation passed
offline against the unchanged selected lockfile.

The strict-gzip example now uses incremental hashing; a real run over minimal
Wasm compressed by the existing system gzip matches independent sha256sum
values for compressed and decoded bytes. Evidence is at the path recorded in
/tmp/ic-host-044-example-path. Snapshot, pins, links, sorting and formatting
checks pass. No dependency, feature, package version, lockfile or existing API
was removed or changed by this batch; gzip error diagnostics now describe both
raw and decoded payload failures. No function, method or type was removed.
Native macOS qualification remains outstanding, and no full local gate, commit,
push, publication, download or sibling edit occurred.

## Previous qualification: 2026-10-07, after 0.4.2

The maintainer reports 0.4.2 pushed; local HEAD is
`6501d0e9fa7ba0439ec7a4010ca7bf0205e1d712` with finalized root notes and package
versions at 0.4.2. [Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37624014360)
passes Linux/MSRV but both macOS jobs fail in the shared temporary-directory
fixture before Rust tests. Raw native logs are retained at
/tmp/ic-host-042-arm-ci.log and /tmp/ic-host-042-intel-ci.log. This supersedes
the previous queued state and does not establish registry or consumer adoption.

The next compatible batch targets **0.4.3**. Shared Tooling 0.1.19 is adopted
from exact local commit `a06e4719e3839b8eefcfb88ec8923aa88eb63ccc`, through a
clean private checkout and the canonical exporter. The 67-file snapshot adds
the Rust installer/fixture and the IC pin-reader dependency. The clean upstream
checkout and export evidence are under /tmp/ic-host-043-shared.oquvT1. Later
dirty commit-policy edits were excluded; the prohibition on agent commits
remains. No matching hosted upstream run was available at review. This local
adoption still requires final committed consumer/native qualification.

The snapshot repairs the macOS tool-command fixture's unnormalized temporary
root, guards optional Rust installation paths, preserves changelog history/EOF
and corrects LOC target-symlink handling. The consumer release adapter normalizes
heading whitespace consistently with the new finalizer. Optional Rust bundle
targets are exposed independently of aggregate setup/check; no real tools were
downloaded or installed. Existing pins remain and the shared matrix adds
cargo-sort-derives 0.13.0 and candid-extractor 0.1.6. Coordination remains on
[Host #6](https://github.com/dragginzgame/ic-host-tooling/issues/6).

`durable::write_typed_with` and Unix `durable::write_at_with` address
[Host #15](https://github.com/dragginzgame/ic-host-tooling/issues/15) and
[Host #16](https://github.com/dragginzgame/ic-host-tooling/issues/16). Both share
the existing publication engine and `NamedWriteError<E>`, with explicit
replace/create-only options and permissions. Descriptor publication borrows
standard `BorrowedFd`, validates one filename and stays anchored to the held
directory when its path moves. The hard-link create-only fallback now reports
staging cleanup failure as after-publication instead of silently succeeding.
Production dependencies and package versions are unchanged, and no API was
removed. Serde and serde_json test dependencies reuse their already-selected lockfile
versions. Cargo.lock changes only the filesystem package's dependency edges.

Focused Linux checks pass: 58 filesystem tests, 38 dependent process tests,
strict filesystem Clippy, Rust 1.88 compilation and Rustdoc. Tests retain real
serde/native errors, bounded writes, create-only conflicts, original output,
directory anchoring, invalid-name/descriptor refusal, final-symlink behavior and
foreign staging cleanup. Sync/unlink failures use injected faults; ordinary
filesystem operations are real. Initial lock-preparation and Clippy failures
are retained alongside corrected passes in /tmp/ic-host-043-*.log. No full local
CI/release gate ran. Native macOS Intel/ARM proof remains outstanding.

Focused shared checks also pass: trailing-slash and symlinked temporary paths, substituted
Rust/IC installation, release-runner and consumer adapter fixtures, historical
EOF preservation, LOC, snapshot integrity, declaration pins, links, sorting,
formatting and ShellCheck. LOC initially lacked cloc on PATH; the passing rerun
used existing sibling tools read-only after checking Host's exact pins. This
does not provision this checkout's tools. Installation and Git effects in
fixtures are substitutes.

Read-only consumer inspection identified Query at clean
`11aedb9d233dd2580a401fa4a9b35892a1a75f57`, Backup at
`7660b56c196d2af3d084524bc74bb9dd4982d1ad` with unrelated dirty work, and Canic at
`900ef517695a14423764910e22ae7c22af4a41a3` with its inspected JSON writer unchanged.
Query's cap-std directory implements standard `AsFd`, so it can retain root and
target admission while removing its allocator/publication pipeline and serde
error bridge. Its pre-serialization rule remains local if invalid input must not
create parents. [Query #19](https://github.com/dragginzgame/ic-query/issues/19)
owns adoption. Backup and Canic need a plan preserving their exact crash-barrier
and recovery proof before replacing their duplicate JSON publication engines;
see [Backup #24](https://github.com/dragginzgame/ic-backup/issues/24) and
[Canic #458](https://github.com/dragginzgame/canic/issues/458). No sibling files
were edited. All three Host issues remain open for delivery/native evidence;
no commit, tag, push or publication occurred.

## Previous qualification: 2026-10-07, after 0.4.1

The maintainer reports 0.4.1 live. Local HEAD and v0.4.1 identify
ce2dd57cedc5000b44bb6a9ff5194f7d65a42c38, with finalized root notes and package
versions at 0.4.1. [Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37614534197)
has passed Linux and MSRV; both native macOS jobs have now failed in the shared
tool-command fixture before reaching the Rust tests. The fixture compares an
unnormalized temporary directory with Make's independently resolved `CURDIR`.
[Shared #56](https://github.com/dragginzgame/shared-tooling/issues/56) records the
native failure and a Linux trailing-slash reproduction. Normalizing the fixture
root with `pwd -P` passes both trailing-slash and symlinked temporary-root cases
in a private copy at /tmp/ic-host-042-tool-command.k2qV6s. The canonical shared
source and this repository's snapshot remain unchanged pending a committed fix.
This supersedes the earlier queued status; native qualification and independent
registry verification are not complete.

The next compatible batch targets **0.4.2**, implementing the concrete installed
extractor case in [Host #11](https://github.com/dragginzgame/ic-host-tooling/issues/11).
`AdmittedTool::admit_version` takes `VersionSpec`, records a bounded observed
executable identity, checks the exact version and reuses the existing admission
and execution engine. Subsequent runs enforce that digest and permissions.
Existing `ToolSpec` admission retains its pre-execution trusted-digest check.
No functions, methods or types were removed; package versions, dependencies,
lockfile, tool pins and snapshot selection are unchanged.

The current IcyDB caller at clean 15e083e479129bb8a62aede0e768a2b5c40be23b
still hashes its installed extractor before constructing `ToolSpec`. The new API
removes that composition after publication/adoption; installation provenance,
executable selection, input/output budgets and recovery remain consumer-owned.
Observed bytes are never described as a published pin. Sibling files remain
read-only; adoption is coordinated in [IcyDB #307](https://github.com/dragginzgame/icydb/issues/307).

Focused offline Linux qualification passes: 38 process tests, 19 dependent tools
tests and example compilation, strict process Clippy, Rust 1.88 and Rustdoc.
Tests cover exact version admission, retained failed-version evidence, bounded
executable/output reads, deadline cleanup and rejection of byte/permission drift
before execution. Formatting, declaration pins, local links and snapshot checks
also pass. Evidence is under /tmp/ic-host-042-*.log, retaining the initial test
variant typo and Clippy correction separately from the passing results.

A private rehearsal at /tmp/ic-host-042-extractor.GFIdnx uses the existing native
candid-extractor 0.1.6 without downloading or installing it. Version admission
observes digest cbc4bc7ff04bec168a5c491990b6693ad6a30128196131ba6d2fdab982f62255
and 15529640 bytes, then the shared Candid adapter successfully extracts a minimal
WAT service at a spaced path while preserving its input. Its 64 MiB executable,
4 KiB input and 1 MiB output bounds are rehearsal choices, not consumer defaults.
This is not full IcyDB graph/adoption evidence. New API native macOS qualification
still requires matching committed CI. No full local gate or release ran.

The same 0.4.2 draft now adds two compatible artifact-owner improvements.
`chunk_digests` implements the ordered chunk/whole-input portion of
[Host #12](https://github.com/dragginzgame/ic-host-tooling/issues/12), reusing the
bounded reader with explicit nonzero chunk size, input-byte and digest-count
limits. Chunk boundaries ignore reader fragmentation; allocation is fallible,
no chunk-sized buffer is allocated, and failure returns no partial identities.
It hashes the exact supplied representation; gzip detection and decoded-module
hashing are not implemented by this function.

Read-only Canic inspection at ab7227f0acb216fdabdceb76fd134c7a369d2def identifies
`append_chunk_actions` and `stage_managed_release_set` as callers that separately
collect chunk hashes and the complete artifact hash. Their inspected source and
`cdk::utils::hash::wasm_hash` are clean at that revision. The hash helper is raw
SHA-256, without decompression or a domain prefix. Adoption still needs consumer
budget/error/schema choices and qualification, tracked on
[Canic #458](https://github.com/dragginzgame/canic/issues/458); no Canic code changed.

`From<CopyError> for io::Error` addresses the remaining I/O projection feedback in
[Host #14](https://github.com/dragginzgame/ic-host-tooling/issues/14). Native input
and output errors retain their kind, OS code and custom cause; non-I/O input
failures remain downcastable `CopyError` causes under `Other`. Direct `CopyError`
remains necessary when callers need input/output provenance. IcyDB's clean
`publish_artifact_copy` at bc8eef82a906384eb1d43dd314a1b23277b654d7 can replace
`map_err(io::Error::other)` after adoption; its files were not modified.

Focused Linux checks for these additions pass: 58 artifact unit tests and seven
consumer-contract tests with all features, 33 minimal-profile tests, strict
artifact/filesystem Clippy, artifact Rust 1.88 compilation and Rustdoc. The
focused durable-copy test confirms native I/O identity, typed byte-limit failure,
old-destination preservation and successful publication. Formatting, pins,
snapshot and documentation checks also pass. Evidence is in
/tmp/ic-host-042-more-*.log; initial test-only Clippy failures are retained beside
the corrected passes. No production APIs, functions or types were removed, no
dependencies changed, and native macOS qualification remains outstanding.

Shared Tooling review identifies committed 0.1.18,
a3430b34b32a60f3b245a2b4f7e2f5321556fe56, with
[successful upstream native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37604299590).
Its new Rust installer is affected by [Shared #54](https://github.com/dragginzgame/shared-tooling/issues/54);
the route-admission repair and runner disk checker are still dirty upstream work.
This batch retains the verified 0.1.17 snapshot rather than exposing that installer
or attributing unpublished fixes to 0.1.18. Adoption coordination stays on
[Host #6](https://github.com/dragginzgame/ic-host-tooling/issues/6).

## Previous batch: 0.4.1 preparation

The maintainer released 0.4.0 at 6b171744def811882ba6c71d50135efa898302a9
(v0.4.0), and reports it live. [CI for that exact source](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37602699181)
passes Linux, MSRV and both macOS 15 architectures. This supersedes the pending
native qualification in the historical preparation evidence below; it does not
prove adoption by any consumer.

The current compatible tooling and named-staging batch targets **0.4.1**.
Shared Tooling is adopted
at committed 0.1.17, 88f1d70cdf671aefb9507d7a81411ed5daa358b3, with 64 exact
snapshot entries. Export used a clean private checkout and excluded the sibling's
newer uncommitted changes. [Upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37601115116)
passes all its native jobs. The common Make include replaces six local setup/check
recipes, provides LOC reports, and selects pinned jq/yq/ripgrep/cloc consistently
for Make and CI. The only added tool pin is cloc 2.10; existing selections remain.

The release adapter accepts trailing heading whitespace while preserving note
placement and rejecting duplicate/misplaced entries. A concurrent maintainer edit
shortened the four internal dependency requirements to `0.4`; at the maintainer's
request this is preserved and the metadata projection now retains requirement
precision. Real offline cargo-set-version probes in private copies match exact
projected manifests and lockfiles for 0.4.1, 0.5.0 and 1.0.0. Actual package versions
remain 0.4.0 and Cargo.lock is unchanged.

Focused Linux release-adapter, shared release-runner, Make command delegation,
installer failure/preservation and LOC fixtures pass. Host workspace LOC reporting
also runs successfully. Formatting-hook selection/index preservation, formatting,
declaration pins, documentation links, ShellCheck and workflow lint pass.
Installer and Git release effects in fixtures are
substitutes. Real LOC execution used existing sibling tools read-only after
verification against the adopted pins; it does not establish installation in this
checkout. Logs and the frozen upstream source are retained under
/tmp/ic-host-shared-017.Cq5xAc. The initial adapter rejection of shortened
requirements and an incomplete private cargo-edit probe are retained alongside
their corrected passes. That tooling adoption changed no Rust production code;
the subsequent filesystem addition is qualified separately below. No full local
gate ran, and no native CI result yet covers these uncommitted adoption bytes.

The adoption and consumer-specific release checks remain coordinated on
[Host #6](https://github.com/dragginzgame/ic-host-tooling/issues/6). Sibling files
were not edited; no commit, tag, push, publication or tool download occurred.

The requested [Host #8](https://github.com/dragginzgame/ic-host-tooling/issues/8)
implementation adds `durable::write_named_with` and `NamedWriteError<E>`. Named
external producers share the existing durable publication engine; there is no
second allocator/rename pipeline. Callback errors keep their original type and
process evidence, cleanup failures remain separate, and final directory-sync
errors report that publication already happened. The held staging descriptor and
parent are checked before publication, and cleanup refuses foreign replacements.
Consumers must write/truncate the precreated inode, validate bounded content,
finish all writers and control the parent hierarchy. Namespace checks are not an
atomic comparison-and-rename primitive against concurrent hostile writers.

Focused offline Linux checks pass: 51 filesystem tests and 34 process tests,
including missing/invalid output, file/symlink/FIFO/directory substitution, extra
hard links, parent replacement, replacement after sync, cleanup rejection,
before/after-publication sync failures, bounded process overflow and timeout.
Strict filesystem/process Clippy, Rust 1.88 and Rustdoc pass. The initial Clippy
const-function suggestion and corrected result are retained under
/tmp/ic-host-issue8-*.log. No existing functions, methods or types were removed.

A private real Binaryen 132 rehearsal at /tmp/ic-host-issue8-binaryen.zzcRc4 uses
IcyDB's existing executable read-only after exact digest/version admission.
Producing a minimal Wasm module at spaced paths and rejecting invalid input both
pass, preserving the previous output, process evidence and ordinary staging
cleanup. The fixture records source hashes from IcyDB's dirty working input on
4d5b38a543bd4f7f349034b4797d78ddad6e76e3; it is not a full consumer adoption or
general Wasm-validation proof. Native macOS qualification of the new API and
published consumer adoption remain outstanding on Host #8 and
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307).

## Previous batch: 0.4.0 preparation

The maintainer released 0.3.3 at 3d18ca9a9ed0ac5935a16c5bac99694d8e9a7d0a.
The current uncommitted reader/error cleanup targets **0.4.0** because it removes
public APIs and changes error contracts. Manifests and Cargo.lock remain 0.3.3;
dependencies, tool pins and the adopted snapshot are unchanged. The
[migration notes](../changelog/0.4.md) bind the required consumer changes to
[Host #14](https://github.com/dragginzgame/ic-host-tooling/issues/14).

All reads now belong to `ic_host_fs::read`. Optional no-follow reads require a
byte budget and share the existing bounded descriptor/stream engine. Private
reads distinguish missing files from typed admission/read failures. The durable
module no longer duplicates readers or formats lock errors into strings.
Following paths, no-follow admission, opened descriptors, private permissions
and direct descriptor lock ownership retain their separate obligations.

Focused offline Linux checks passed: all 44 filesystem tests, 32 process tests,
19 tools tests and applicable examples; filesystem strict Clippy, Rustdoc and
Rust 1.88 compilation; formatting, links, pins, feature graphs and the exact
58-file snapshot check. Lock callback failure was exercised under actual
cross-process contention. Private-file metadata checks used real files, while
stream interruption/trailer failures used a controlled reader. The first
Clippy attempt failed on style lints; its log is preserved alongside the passing
attempt under /tmp/ic-host-040-*.log. Native macOS and full local CI/release gates
were not run for these uncommitted bytes.

Canic is the only inspected sibling source referencing the retired APIs. The
source inventory records 50 matching files, with hashes and clean source at
d9b10b07d610b513aa90d221040a8d27a7fc9ed7, in
/tmp/ic-host-040-consumer-source.json. An authorized isolated migration from that
revision is retained at /tmp/ic-host-040-canic-rehearsal/consumer.patch. Its full
Host/CLI dependency graph passes all-target/all-feature compilation and strict
Clippy on Linux; 138 selected network, release-set, diagnostic, Candid-cache,
generation and plan-content tests pass. One installed-extractor test is ignored
by its existing requirement; native extractor fixtures pass. This is not a full
consumer suite or native macOS qualification.

The private rehearsal selects candidate Host crates at fixture-only version
0.4.0 alongside published 0.3.3 transitive dependencies. Its proposed 16 MiB
document and 128 MiB artifact budgets require consumer review against retained
inputs; existing smaller format limits remain. The adoption patch excludes local
path overrides and Cargo.lock; registry lock selections must be generated after
publication. This work did not edit actual sibling checkouts or this repository's
package versions and lockfile. Canic's active Cargo.lock changed independently
during the rehearsal; the candidate still applies, but those newer dependency
selections are not qualified by the frozen rehearsal. No commit, tag, push,
upload or tool download occurred.

The migration exposed repetitive typed-error projection at existing consumer I/O
boundaries. ArtifactError and RegularFileLockError now convert into io::Error
without losing native error identity or typed admission causes. A focused
artifact regression, all 44 filesystem tests, strict artifact/filesystem Clippy,
Rust 1.88 checks and Rustdoc pass on Linux. The first Clippy attempt rejected a
single-variant wildcard; both failed and corrected logs remain under
/tmp/ic-host-040-io-*.log.

Publication now inspects all four prepared archives' clean source/path records
and fetches the exact commit into a fresh object store before upload. Expanded
offline failure fixtures and ShellCheck pass. A separate live rehearsal used
real Cargo archives and exact HTTPS retrieval for committed 0.3.3 source
3d18ca9a9ed0ac5935a16c5bac99694d8e9a7d0a; the final upload was replaced by an argv
recorder. Its evidence is retained under
/tmp/ic-host-040-publication-live/source/target/publish/publish.xPawGS. This proves
the guard mechanics, not remote availability or qualification of dirty 0.4.
The original provenance follow-up remains on
[Host #7](https://github.com/dragginzgame/ic-host-tooling/issues/7).

The existing CI matrix includes native macOS 15 Intel and ARM. Qualification of
these candidate bytes requires a maintainer commit and push/PR; agents must not
create commits. The successful CI for released 0.3.3 does not qualify dirty 0.4.
Consumer review remains on
[Canic #458](https://github.com/dragginzgame/canic/issues/458).

## Previous batch: 0.3.3 preparation

The maintainer released 0.3.2 at c7c0d85765054909c05d86f6d3fd2c9965510335.
The following batch was prepared for compatible 0.3.3; package manifests,
Cargo.lock, tool pins and the adopted snapshot stayed unchanged during preparation.
The history below records earlier batches and their evidence, not current versions.

[CI for the released source](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37589678525)
passed Linux and MSRV but failed on macOS 15 Intel and ARM at the same path test:
native canonicalization accepted `file/..` where Linux rejects it. The repaired
test compares native results, including after missing-prefix normalization,
without changing production resolution or weakening actual native errors.
Seven focused path tests pass on Linux. Native macOS requalification is pending.

The shared capture entry accepts the caller's existing `Command`, preserving
opaque platform setup and executable policy while using the admitted runner's
single bounded capture engine. `ToolError` now exposes borrowed evidence and
execution failure accessors. No production symbols were removed or compatibility
reexports added. Scope and consumer coordination remain on
[Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5) and
[Host #9](https://github.com/dragginzgame/ic-host-tooling/issues/9).

Focused offline Linux checks passed: 32 process tests, 19 dependent tools tests
and examples, process/filesystem strict Clippy and Rust 1.88 compilation, strict
process Rustdoc, formatting, documentation links, pins, feature graphs and exact
58-file snapshot verification. Logs remain under /tmp/ic-host-033-*.log, including
the original macOS CI failure and the initial test-style Clippy failure. No full
local CI/release gate or native macOS execution was performed.

All sibling repositories stayed read-only. A Canic candidate against inspected
5e0dd13f7b494e800201841223a6167b544a5892 is retained at
/tmp/ic-host-033-canic-capture/consumer.patch with source digests and logs. It
removes the local child owner/nonblocking reader loop, retains the product error
adapter and requires future published 0.3.3. The patch applies to that source;
two tests pass in a private reduced-error fixture, not Canic's full dependency
graph. Consumer adoption is coordinated on
[Canic #458](https://github.com/dragginzgame/canic/issues/458).
No commit, tag, push, publication, dependency upgrade or tool download occurred.

## Historical evidence

The maintainer authorized creating /home/adam/projects/ic-host-tooling with
four ic-host-* packages and copying the existing implementation as extraction input.
The committed library import is IC Host Tools 0.2.0 at
be7d73908225e73ab5babf8fbb9fe959f40d439b. Its dirty tooling/docs were excluded.
Canic's durable source is clean at the revision recorded in ci/extraction-sources.json.
The original IC Host Tools local checkout was subsequently retired as recorded
below. Other sibling checkouts and consumer dependencies remain unchanged.

Workspace setup separates artifact streams/inspection, filesystem operations,
admitted execution/Git observations and IC-specific adapters. Tests and examples
use their new owners. Bootstrap packages were non-publishable with inherited
0.2.0 metadata; the maintainer subsequently completed the breaking Git 0.3.0
release at 0456b40e116c7d070428b070d1c3befc36a9345d with tag v0.3.0. Package
publication policy now permits crates.io through the explicit commands below.
Registry publication has not been performed by the agent.

Shared Tooling remains separate and is adopted at exact reviewed revision
25e7ce83149e081e4dcc52c55c33724e44153f2a (0.1.14). The prior adoption was
46c02774a8335cb3949d6f04284c4f53375353c1. The original bootstrap baseline was
d957d1f8801885c5b69e4a9ef900155f5f2a8a9d; the earlier adopted revision was
21f3ec3dd97f2968c9f0b08924451bb2f71770d1. All automation uses its local snapshot.
The original bootstrap authorization excluded commits, GitHub repository
creation, pushes, publication, installation and consumer adoption. Native macOS
execution remains CI-owned.
Focused Linux setup checks passed for each of the four changed packages:
all-target/all-feature tests and examples, Clippy with warnings denied,
Rustdoc, and Rust 1.88.0 all-target/all-feature compilation. Artifact tests
also passed with default features disabled. All 38 imported registry lock
selections remain unchanged. Workflow syntax and release Make adapters passed;
the adapter check used a substitute runner with no release effects.

The local extraction setup is ready for maintainer review. Consumer adoption,
native macOS qualification and actual registry publication remain separate work.

The installed workspace's 52-file Shared Tooling snapshot, parsed dependency
and inheritance declarations, formatting, documentation links and selected
package tests passed during bootstrap. The local formatting hook is installed.
The original initial-commit blocker is superseded: the local checkout has the
maintainer-owned commit 67c10830922f9249bfa30ec4f37010bf746519ce and a tracked
Cargo.lock. The public dragginzgame/ic-host-tooling repository was created on
request, and origin uses https://github.com/dragginzgame/ic-host-tooling.git.

The consumer release adapter now validates locked/offline Cargo metadata without
requiring registry-publishable packages. Git version/tag releases remain separate
from registry publication. The extraction's 0.3.0 notes are finalized; the new
compatible publication commands select pending 0.3.1 without changing the current
0.3.0 manifests or lockfile. Release preflight
now prepares and checks only pinned host parsers; the IC executable bundle stays
explicit setup. Changelog selection and rewriting use the shared finalizer with
the saved previous version, retaining the local top-entry and placement guards.
Focused Linux checks cover non-publishable packages, failed metadata queries,
dirty-source refusal, complete-ledger conflicts, historical-note preservation,
saved receipts and exact committed payloads beneath different working metadata.
Cargo metadata admission is real and locked/offline; Git, setup, qualification
and version mutation in these regression fixtures are substitutes.

The refreshed 54-file snapshot includes the shared formatter prerequisite checker
and regression fixture. It was exported from a clean temporary checkout of the
committed revision, excluding the sibling's unrelated working edits. Formatter
refusal checks, actual formatting, hook integration and Make release delegation
passed on Linux, as did the shared host-tool and release-runner substitute
fixtures. The new minimal artifact target passed 18 tests with optional
features disabled and is included in configured native CI. This batch has no
native macOS execution or complete release-gate evidence. The agent did not
execute a release command or change package versions, dependencies or tool pins.
The hook check also passed dependency-order perturbation and working/index
preservation. An earlier malformed perturbation moved a comment's dependency
association and failed the expected-byte comparison; its fixture remains at
/tmp/formatting-adoption.hOzBNO. The corrected input changed only dependency
order and passed; the failed attempt is not relabeled as a product failure.

The explicit `make publish` target delegates the four-package upload to native
Cargo with a clean-source check, locked dependencies, all features and an explicit
crates.io destination. Its separate `make publish-check` admits working edits for
a no-upload dry run. Both retain source identity, metadata, exact arguments and
Cargo logs under target/publish; upload recovery remains caller-owned as described
in [the publication procedure](../publishing.md). No automatic retry, version bump,
Git effect or artifact cleanup occurs. Credentials remain with Cargo.

Focused Linux publication fixtures passed with real locked/offline metadata and
substituted Git/upload commands: dry-run flags, dirty-upload refusal, package
policy and version rejection, failure propagation, pre-dispatch intent, concurrency
refusal and artifact preservation. Release adapter and Make delegation checks also
passed with substituted effects, retaining non-publishable-package coverage.
Formatting, declaration pinning, shell lint and documentation links passed.
With separately authorized network access, Cargo
1.99.0 `make publish-check` packaged and compiled all four 0.3.0 crates with all
features; every upload was aborted by Cargo's dry-run flag. Evidence is retained
in target/publish/check.ZutGR9. Inspection then found missing license text; inherited
license-file metadata now includes the canonical root MIT notice in every crate.
The repeated dry run passed in target/publish/check.JrsZix, and all four packaged
LICENSE files matched the root bytes. Both attempts retain their exact archives.
That attempt emitted Cargo's redundant SPDX license/license-file warning. The
subsequent correction retains only inherited MIT SPDX metadata, with per-crate
LICENSE symlinks to the canonical root notice. Cargo packages those links as
regular files; all four archived notices matched the root bytes. The online dry
run passed without the license warning in target/publish/check.Y2p7MG, with its
exact archives retained. Cargo also reported all four 0.3.0 versions already
present in the crates.io index. The compatible correction remains in pending
0.3.1; no package version changed and no upload occurred during these checks.
This validates packaging on Linux, not credentials, registry ownership, actual
uploads, consumer adoption or native macOS.

The earlier offline dry run failed at registry access and remains recorded in
target/publish/check.DadcVI. An offline package-only attempt assembled all four
archives and verified artifacts, then failed verifying filesystem with Cargo's
internal "no hash listed" error for the temporary registry; its evidence remains
in target/publish/package-check.XyNI7p. The subsequent online pass does not
reclassify either failed attempt. No dependency selections or tool pins changed,
and this batch has no full CI or native macOS execution evidence.

The maintainer authorized deleting /home/adam/projects/ic-host-tools. No sibling
local dependency paths pointed at that checkout. Its complete Git metadata,
uncommitted tooling/docs, untracked files, ignored files and retained artifacts
were archived and compared against the original before removal. The verified
backup is /tmp/ic-host-tools-retirement.an3fDZ/repository.tar, with its SHA-256
beside it. This is temporary storage, not permanent archival. The GitHub source
repository and existing consumer dependency selections were unchanged.

The requested sibling extraction review covered the 13 sibling repositories at
discovery level and traced the eight consumers still selecting ic-host-tools 0.2.
Only this workspace was mutated; sibling working trees and retained artifacts were
read-only. The previous license-warning correction remains in the working tree.
Three compatible shared additions extend the pending 0.3.1 batch: MatchingWriter,
bounded zero-timestamp gzip encoding, and streamed durable replacement through
write_with. Byte-based writes converge through the same existing commit engine.
Exact committed source file digests and extraction selections are recorded in
ci/extraction-sources.json; selected files matched HEAD despite unrelated sibling
dirty work. The existing Canic copyright notice is preserved in the root license.

Focused Linux checks passed: all-feature artifact library tests (46), filesystem
library tests (25), minimal artifact tests (21), strict all-target/all-feature
Clippy, Rustdoc with warnings denied, and Rust 1.88 all-target/all-feature checks
for the two changed packages. Offline cache preparation, formatting, declaration
pinning and documentation links also passed. An initial Clippy attempt rejected
test-local items placed after statements; moving the fixture declarations before
statements resolved it. No dependencies, versions, tool pins, commits, releases,
publication, consumer builds or full CI were performed in this extraction batch.
Native macOS qualification remains outstanding.

The immutable [sibling review evidence](../reports/audits/2026/10/06/sibling-host-extraction/01/report.md)
records ownership traces, retained stronger contracts and qualification limits.
The authorized adoption feedback is on GitHub: existing
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6021451439)
was updated; new issues are
[IC Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13),
[IC Query #11](https://github.com/dragginzgame/ic-query/issues/11),
[IC Backup #11](https://github.com/dragginzgame/ic-backup/issues/11),
[IC Memory #16](https://github.com/dragginzgame/ic-memory/issues/16),
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307),
[IC Blob Storage #14](https://github.com/dragginzgame/ic-blob-storage/issues/14) and
[Toko Miner #30](https://github.com/dragginzgame/toko-miner/issues/30).
Shared API implementation is not consumer adoption: sibling copies still exist
until their owners complete those changes. New APIs remain uncommitted pending
0.3.1; issue bodies explicitly distinguish them from the released split 0.3.0 APIs.

The maintainer-requested second pass found three more consumer reuse sites using
existing 0.3.0 primitives: Canic network SHA-256 codecs, Query's confined bounded
stream collection and IcyDB's diagnostic file/stdin collection. Existing adoption
issues received source-backed comments with error/accounting acceptance checks;
no additional shared API was justified. The [second-pass evidence](../reports/audits/2026/10/06/sibling-host-extraction/02/report.md)
records current source identities separately from the first run. write_with docs
now distinguish returned errors and attempted staging cleanup from panic/interruption
retention. Focused filesystem Rustdoc, formatting and documentation checks passed;
this documentation-only correction adds no new runtime or native macOS evidence.

A follow-on maintainer-authorized extraction adds explicit-base missing-suffix
path resolution and descriptor lock waiting to the pending 0.3.1 batch. The
existing pathname progress helper shares the acquisition engine while preserving
its one-second reporting policy. The descriptor entry point lets Testkit retain
its opener, 25ms polling cap and phase heartbeat mapping. The source observation
and selected mechanics are recorded in ci/extraction-sources.json; cache domains,
pruning and retained-lock lifecycle remain in Testkit.

Consumer preparation exposed a maximum-filename regression in durable staging:
including a 255-byte destination name in the staging name failed with Linux
ENAMETOOLONG. The failed behavior log is
/tmp/ic-testkit-host-long-name-regression.log; its private fixture is retained.
An earlier attempt failed compiling the new collision fixture because its
existing hook's mode argument was missing; that attempt remains separately in
/tmp/ic-testkit-host-long-name-before.log. Short owned staging names now support
maximum-length destinations, exclude the output name even case-insensitively,
retry collisions within the existing bound and preserve unowned files.

Focused filesystem library tests (34), all-target strict Clippy, Rust 1.88
all-target checks and strict Rustdoc pass on Linux. The interruption test injects
Interrupted at the acquisition boundary; it does not claim native signal delivery
qualification. Real descriptor contention, callback failure, unowned collision
preservation, selected staging-namespace destinations and complete publication
are exercised. Native macOS qualification remains outstanding.

The prepared Testkit consumer patch is retained at
/tmp/ic-testkit-host-adoption/consumer-adoption.patch. A private consumer fixture
at /tmp/ic-testkit-host-extraction.b9rs0cq9 tested it with explicit local-source
patch selections and unchanged package versions; source hashes are recorded in
its reviewed-host-inputs.json. Focused digest/publication, missing-parent path,
lock heartbeat, all 28 transaction, linked/restricted Wasm publication, copied
stamp and partial-materialization tests pass using Testkit's normal target
folder. This is source qualification, not registry or live consumer adoption.
The actual consumer remains on published 0.3.0 until reviewed shared publication;
no version, commit, tag, push, upload or full gate was performed. Implementation
feedback is on [host-tooling #1](https://github.com/dragginzgame/ic-host-tooling/issues/1),
[host-tooling #2](https://github.com/dragginzgame/ic-host-tooling/issues/2) and
[Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13).

The requested [consumer crate-usage review](../reports/audits/2026/10/06/consumer-crate-usage/01/report.md)
records current working-tree imports separately from published dependency edges.
Its saved manifests, lock edges and source digests cover the eight direct consumers
and discovery across all 13 siblings. It identifies remaining Canic/Toko source
alignment failures and older published transitive owners without claiming new
consumer execution evidence. Siblings remained read-only; the new concrete Toko
and Backup findings were added to their existing adoption issues.

The requested Shared Tooling review refreshed the same 54-file adoption from
21f3ec3dd97f2968c9f0b08924451bb2f71770d1 to committed revision
46c02774a8335cb3949d6f04284c4f53375353c1 (0.1.11). The sibling's dirty pending
0.1.12 work was excluded by refreshing from a private checkout of that commit.
The new snapshot includes exact large-component changelog comparisons and the
paired maintenance guidance; no new shared file surfaces were added.

The release adapter now delegates manifest version observations to the existing
shared Cargo/TOML reader. Its consumer-owned AWK projection retains exact
version-only payload preparation, synchronized path requirements and owned
lockfile reads. Valid inline version comments survive preparation and recovery;
failed parser output, failed Cargo path observations, duplicate workspace keys
and unsynchronized path versions are rejected before setup or mutation. Version
queries require the pinned jq/yq set to be prepared first, as documented in README.

Focused offline Linux checks passed: release version observation, the expanded
adapter fixture, shared Cargo metadata fixtures, release-runner command-stub
fixtures, ShellCheck, Bash syntax, snapshot integrity, dependency declarations,
documentation links and formatting. Git effects, tool setup and version mutation
were substituted in release fixtures; no release operation was run. Cargo.lock,
tool pins and package versions are unchanged by this review. Native macOS
qualification and full release gates remain separate from this evidence; these
changes are uncommitted additions to the pending 0.3.1 batch.

The maintainer's commit attempt exposed an integration failure: the four crate
LICENSE symlinks conflict with the adopted formatting hook's regular-file-only
snapshot contract. They are replaced by regular byte-identical copies of the
canonical root notice; the shared hook remains unchanged. Both publication modes
now reject missing, symlinked or differing notices before Cargo dispatch. Focused
publication fixtures cover those refusals with substituted Git/upload effects.
Offline Cargo package listings include LICENSE for each crate and all four
working copies match the root. ShellCheck, Bash syntax, formatting, documentation
links and snapshot integrity passed. No new archive verification, native macOS
qualification, commit, release or publication occurred in this repair.
The repaired files were staged without disturbing the existing selection, and
the actual pre-commit hook passed against that selected index payload on Linux.

After the maintainer committed the batch as 5e73861, a filesystem test failed at
the immediate close/reacquisition assertion. The descriptor test lacked the
subprocess isolation already used by the older path-lock test: a parallel process
spawn can inherit a locked descriptor until exec, briefly retaining exclusion
after the originating descriptor closes. The descriptor test now runs alone in
an exact-test subprocess, preserving its contention, returned ownership and
immediate close-release assertions. Production acquisition behavior is unchanged.
This remains compatible pending 0.3.1 work; no tag or package version changed.

The reported failure was not reproduced in the isolated baseline run or 20
parallel baseline runs; the inherited-descriptor explanation is source-based,
not a captured failing interleaving. Baseline logs remain in
/tmp/ic-host-lock-before-*.log. After isolation, all four lock-wait module tests,
all 34 filesystem tests and 20 further library runs with 32 test threads passed;
the repetition logs remain in /tmp/ic-host-lock-after-*.log. Strict all-target
Clippy, Rust 1.88 all-target checks, formatting and documentation checks passed
offline on Linux. Full release gates and native macOS qualification were not
performed; this repair remains uncommitted for the maintainer.

The maintainer released 0.3.1 at 38a2a5127be064014e6d39d72d0300ffb2cf20be.
The current [host reuse audit](../reports/audits/2026/10/07/host-reuse/01/report.md)
repairs missing-path traversal/dangling-target resolution and adds an explicit
caller-owned missing-target depth allowance in pending 0.3.2; manifests remain
0.3.1. Five unused extracted fixture copies are deleted. All 39 filesystem tests,
strict Clippy, Rust 1.88 compilation and strict Rustdoc pass on Linux, as do the
selected process/tools tests after cleanup. Snapshot, pins and lock selections
are unchanged. Native macOS and full CI/release qualification remain separate.

The maintainer explicitly approved replacements in Canic and Toko Miner.
Canic now reuses published 0.3.1 gzip/hash/digest mechanics; 32 focused native
checks pass. Its source was separately committed by the maintainer during review,
not by this agent. Toko's direct bounded reader uses filesystem ownership and its
native User Hub tests compile. Offline lock sync removes only unreachable direct
0.3.1 tools/process selections. Query's prepared resolver deletion remains a
handoff patch until shared publication; five existing alias/export tests pass in
a private reduced-error fixture, not its full feature graph. Existing owning
GitHub issues record the applied fixes and remaining published graph convergence.
No release, publication, push or deployment was performed in this batch.

The maintainer authorized implementing Host issues
[#3](https://github.com/dragginzgame/ic-host-tooling/issues/3),
[#4](https://github.com/dragginzgame/ic-host-tooling/issues/4) and
[#6](https://github.com/dragginzgame/ic-host-tooling/issues/6). The response-only
tools profile excludes artifact/filesystem/process production dependencies when
default features are disabled; default Candid extraction retains its API and
required owners. The new bounded artifact HashingWriter hashes only successful
accepted writes and returns sink/prefix identity. Producer success, encoding,
flush/sync, descriptor admission and publication remain caller-owned. No JSON
production dependency or fifth crate was added.

The 58-file Shared Tooling snapshot was exported from a clean private checkout
of committed 25e7ce8, excluding the sibling's dirty pending 0.1.15 bytes. It adds
the workspace rule, explicit governance membership, delegated installer and Make
execution guard. Snapshot verification is independent of inspected helpers;
release pushes recheck their destination and dispatch through its captured URL.
The existing four crate locations conform, including inherited metadata and
dependencies. The committed rule permits application-owned apps/ trees without
requiring this repository to move packages. Upstream
[CI for that exact source](https://github.com/dragginzgame/shared-tooling/actions/runs/37586649650)
passed. Pending upstream LOC reporting and the IcyDB layout exception were
reviewed separately as dirty source and were not adopted or executed; cloc is
not installed locally.

Focused offline Linux checks passed: 52 artifact unit and seven consumer-contract
tests, 27 minimal artifact tests, 19 default tools tests and ten response-only
tests with applicable examples. An independent response-only consumer compiled
and ran with no extraction owners in its normal graph. Strict Clippy, Rustdoc and
Rust 1.88 checks passed for the changed packages, including the response-only
MSRV configuration. The dependency-graph guard, formatting, pins, local links,
exact snapshot verification, scoped consumer formatting-hook preservation,
snapshot corruption fixtures, host installer fixtures and release adapter/runner
substitutes passed. Release/installer effects were substituted; no real Git
mutation, upload or tool download occurred. Logs remain under
/tmp/ic-host-next-*.log. Initial compile/style/format failures remain separately
recorded rather than being relabeled as passing attempts.

Query's proposed response profile and private DigestingWriter replacement are
retained at /tmp/ic-query-host-next-candidate/consumer.patch with before/candidate
source digests. During review Query committed that inspected source as clean
0.47.7 at 73dd32bcbe7cdada61dec1cfcd966b252564e48a; all three before-file digests
still match and the patch applies there. It has not been applied or compiled in
Query's full graph. Query retains schema, canonical encoding, descriptor checks
and capability-confined publication. The consumer follow-up is
[Query #14](https://github.com/dragginzgame/ic-query/issues/14). Automatic review
rejected posting the full patch/source digests; the accepted issue contains only
already-public API proposals and Host issue links. The patch remains local.
Actual consumer archive/response tests and native macOS qualification must
follow reviewed publication. These compatible
additions extend pending 0.3.2; manifests and Cargo.lock remain 0.3.1, with tool
pins unchanged. Prior path fixes, audit evidence and fixture deletions are
preserved. Full CI/release gates and consumer native macOS were not run locally.
No commit, tag, push, publication or new sibling file edit was performed.
