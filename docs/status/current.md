# Current handoff

## Latest work: 2026-10-09, #42 checkout-local admission repaired

The authorized [#42 repair](https://github.com/dragginzgame/ic-host-tooling/issues/42)
now binds `SHARED_TOOLING_ROOT` in Host's Makefile before loading shared includes.
Host owns its checkout-local routing, so this completes the local fix without
patching the 88-file Shared 0.2.7 snapshot or waiting for the generic upstream
correction. The earlier upstream-only readiness blocker below is superseded;
[Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30) retains the
generic include-root and argument-bearing Make concerns.

The new maintained Host adoption check failed against the previous Makefile,
then passed with the fix. It covers external environment/command-line roots,
recursive parallel Make, exact formatter/release routing and twenty unsafe-mode
refusals before effects. It runs through `tooling-command-check`. Both GNU Make
4.3/Bash 5 and GNU Make 3.81/Bash 3.2 pass on Linux; the older Make required
separate `override` and `export` declarations. Actual formatting-hook adoption
with real prepared formatters and inherited external roots, plus release routing
with substitute effects, pass under both tool pairs. Snapshot, ShellCheck,
documentation and whitespace checks pass. Evidence, including failed drafts,
is retained in `/tmp/ic-host-095-root-fix/`.

The compatible draft remains 0.9.5. No Rust function, method or type was removed
in this follow-up. Manifests, lockfile, pins, snapshot and real Git index/config
are unchanged. No sibling edit, full gate, native macOS execution, download,
real hook activation, release, commit or push ran. Delivery and native host
qualification remain outstanding.

## Latest review: 2026-10-09, pending 0.9.5 admission-root blocker

Shared Tooling local and remote main still select 0.2.7
`47d6ae6488b8007323fa7c2e22a6efa11d77ae63`; the pending 88-file snapshot
verifies unchanged. A new focused reproduction using Host's actual Makefile
shows that environment or command-line `SHARED_TOOLING_ROOT` selects an external
admission helper during parsing, even for `make help`. An owned sentinel ran
before Make refused; the control succeeded without invoking it.

Evidence is retained in `/tmp/ic-host-095-root-review/probe-2SpEOu` and reported
on [Host #42](https://github.com/dragginzgame/ic-host-tooling/issues/42#issuecomment-6084106030)
and [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30#issuecomment-6084105635).
Keep the pending adoption, but hold release readiness until a qualified,
committed upstream correction binds admission to its reviewed companion.
Earlier passing fixture-isolation checks do not prove this parse-time boundary.
No vendored or Rust source changed in this review; native macOS qualification
remains separate.

## Latest work: 2026-10-09, pending 0.9.5 Make admission and fixture isolation

Completed the authorized [#42 repair](https://github.com/dragginzgame/ic-host-tooling/issues/42)
on released 0.9.4. Canonically adopted committed Shared Tooling 0.2.7
`47d6ae6488b8007323fa7c2e22a6efa11d77ae63`, adding `make/execution.mk` for an
88-file snapshot. Unsafe Make modes now fail at the shared entrypoint boundary;
release-command fixtures pin their tooling root to the disposable checkout.
Formatting-hook qualification also preserves newline-ending roots and Git
object paths. Both changelog views select compatible **0.9.5**.

Applied the prepared Host Makefile integration: isolated release checks export
the new include and its behavioral probe; removed the redundant local
`format-execution-check` target in favor of the shared parse-time guard.
No Rust function, method or type was removed. Direct-only delivery policy,
manifests, lockfile, pins, real Git index/configuration and sibling files remain
unchanged. No commit, push, release, download, activation or full gate ran.

The five upstream executable files and all relevant Host integration inputs
match the earlier rehearsal byte-for-byte, binding its thirty unsafe-mode
cases, external-root isolation, Bash 3.2, newline-root and parallel-Make evidence
below to the adopted behavior. Fresh checks of the actual Host checkout pass:
release routing with substitute effects, real formatting-hook adoption,
88-file snapshot verification, selected ShellCheck, documentation links and
whitespace. Evidence is retained in `/tmp/ic-host-095-adoption/` and the original
rehearsal directory. Native macOS acceptance remains separate; dirty source has
no remote CI result. The preceding upstream-commit blocker is resolved.

## Latest preparation: 2026-10-09, #42 awaiting committed Shared fixes

The authorized [#42 repair](https://github.com/dragginzgame/ic-host-tooling/issues/42)
is prepared and rehearsed in `/tmp/ic-host-095-preparation/host`, based on
released Host 0.9.4. Shared remote main still selects 0.2.6; its pending 0.2.7
repairs are uncommitted. The actual Host snapshot and Makefile remain unchanged.
The temporary checkout overlays five upstream candidate files with exact hashes
in `upstream-inputs.json`; it is not a canonical snapshot or delivered source.

The prepared `host-make.patch` adds the execution include and behavioral probe
to isolated release-check inputs, and removes Host's redundant
`format-execution-check` Make target in favor of the shared parse-time guard.
It preserves direct-only release policy. Thirty direct/inherited unsafe Make
mode cases refuse before effects; direct and parent-Make external-root sentinel
checks remain isolated. Release routing passes under Bash 5/3.2, real Host
formatting-hook adoption passes, and a newline-ending Host checkout passes that
adoption helper under genuine Bash 3.2. Normal parallel formatting checks pass.
All evidence remains under `/tmp/ic-host-095-preparation/`; initial command-CWD
failure evidence is retained separately from passing retries.

Finish by reviewing the committed upstream revision, canonically adding
`make/execution.mk` to the snapshot, applying the prepared local change and
maintaining compatible 0.9.5 notes. Recheck candidate bytes and actual snapshot
integrity before reusing rehearsal evidence. No source fix is applied here yet;
no new release draft, Rust symbol removal, full gate, build, real release, hook
activation, commit, push or sibling edit occurred. Native macOS is unqualified.

## Latest review: 2026-10-09, released 0.9.4 filesystem/process boundaries

Released 0.9.4 is `4e3daebd5df07c6449279535668436024a45c02b`.
[Delivery evidence](https://github.com/dragginzgame/ic-host-tooling/issues/41#issuecomment-6083419271)
records passing Linux/MSRV CI and queued native macOS jobs. Shared main remains
the adopted 0.2.6 revision; its #90 qualification-helper finding remains open.

The bounded code-hygiene review used `audits/code-hygiene.md` at that Shared
revision and the local extraction/host contracts. Traced missing-path/symlink
normalization, required/optional/private descriptor admission, executable
selection, process pipe exchange and lock waiting. Their different missing-file,
permission, confinement, deadline and cleanup contracts justify the retained
boundaries. No additional reproducible defect or safe simplification was found;
no new release draft is justified by this pass.

All seven path and fifteen read-module tests pass on Linux with locked/offline
dependencies. Logs remain in `/tmp/ic-host-095-path-tests.log` and
`/tmp/ic-host-095-read-tests.log`. Process paths received source review only;
this is not native macOS or a full-gate result. No production source, dependency,
package version or sibling changed, and no function, method or type was removed.
Only this handoff and #41 delivery evidence were updated; no commit or release ran.

## Latest work: 2026-10-09, pending 0.9.4 shared Make owners

On released 0.9.3 `545e7236b91d84e190c80931b784f72cc4fafb11`, canonically
adopted Shared Tooling 0.2.6 `ce13a5314916891fd239d9b199b4a91b04775054` from
a clean private checkout. The 87-file selection adds `make/release.mk` and
`make/rust-format.mk`; their command definitions replace equivalent Host recipes.
Host retains direct-only release admission and its formatting execution guard
as a local prerequisite. Isolated release checks now copy both includes.
The compatible pending version is **0.9.4**;
[#41](https://github.com/dragginzgame/ic-host-tooling/issues/41) owns delivery
and native qualification.

Focused Linux checks pass on Bash 5 and genuine Bash 3.2: actual Host
increment/resume routing with a substitute runner, actual formatting-hook
adoption with prepared Rust formatters, and upstream formatting command fixtures.
A separate substitute check proves default help, default release arguments,
direct-only refusal, and guard-before-format ordering/refusal in serial and
parallel Make. Snapshot integrity, selected documentation links and whitespace
checks pass. Evidence remains in `/tmp/ic-host-094/`.

No Rust functions, methods or types were removed; the existing release and
format target names remain available through the shared includes. Manifests,
lockfile, pins, real Git index/configuration and sibling files are unchanged.
No compilation, full gate, download, real release, publication, hook activation,
commit or push ran. Native macOS qualification remains separate. Shared #90's
newline-root formatting-check helper fix is absent from 0.2.6 and remains upstream;
the production hook fix adopted in 0.9.3 is unchanged.

## Latest work: 2026-10-09, pending 0.9.3 hook-path fix

Adopted committed Shared Tooling 0.2.5
`04e07b4bf54e7aeb03eb7804a845cee27b7305df` through its canonical exporter from
a clean private checkout, retaining all 85 selected files. Hook installation
preserves literal configured paths and refuses conflicting newline-ending
selections; installation and execution preserve newline-ending checkout paths.
[#40](https://github.com/dragginzgame/ic-host-tooling/issues/40) owns delivery
and qualification. The compatible pending version is **0.9.3**.

The canonical hook fixtures pass under Bash 5 and genuine Bash 3.2 on Linux,
including selected-file formatting, rejected conflicting selections and failed
Git observations. Host's actual formatting-adoption check, snapshot integrity,
selected ShellCheck, documentation links and whitespace checks pass. Evidence
is retained in `/tmp/ic-host-093-adoption/`. Native macOS qualification remains
separate; no full gate, compilation, download, hook activation or release ran.

The additional bounded flow review traced artifact read/hash/copy/chunk entrypoints,
bounded/hash/matching writers, gzip framing and tar member admission against the
unchanged Shared flow-convergence method and `docs/extraction.md`. Traversal and
gzip framing already converge. Separate archive digest/framing/member checks
protect independent admission boundaries; retain the prior measured copy
specialization. No further production change or public API is justified.
No function, method or type was removed. Package versions, lockfile, pins, Git
index/configuration and sibling files are preserved; changes remain uncommitted.

## Latest follow-up: 2026-10-09, Backup duplicate CI fix

The authorized local [Backup #33 fix](https://github.com/dragginzgame/ic-backup/issues/33#issuecomment-6081929163)
restricts automatic pushes to main while retaining PR checks and existing manual
exact-ref dispatch. All job bodies and native gates remain unchanged. Actionlint,
six event/ref cases, workflow-body equality and selected documentation checks
pass; pending Backup notes select 0.11.3. Incoming dirty lock bytes are preserved.
Testkit, Query and Auth already contain their corresponding committed workflow
fixes; this batch did not edit them. No Rust symbols were removed.

Host 0.9.2's existing CI still has passing Linux/MSRV jobs and both macOS jobs
queued at recheck. #37/#38 therefore retain native acceptance; no source failure
was observed. No full gate, dispatch, commit, push or release ran in this batch.

## Latest follow-up: 2026-10-09, released 0.9.2 and Canic qualification

Host 0.9.2 is released at `c5decaefd17809829bfa969966729d672f609c49`.
[#39](https://github.com/dragginzgame/ic-host-tooling/issues/39) is closed after
consecutive main pushes retained separate CI runs. Native acceptance remains
under #37/#38; released Host Linux/MSRV passed, while macOS remained queued.
The [queue diagnosis](https://github.com/dragginzgame/ic-host-tooling/issues/38#issuecomment-6080329920)
found five running and 118 queued macOS jobs across 55 accessible repositories
at its recorded observation time. Duplicate-gate owners retain their own issues,
including [Backup #33](https://github.com/dragginzgame/ic-backup/issues/33).

With explicit maintainer authorization, the prepared Canic checkpoint-reader
replacement and scoped 0.110.55 notes were applied in Canic. Its
[qualification](https://github.com/dragginzgame/canic/issues/458#issuecomment-6081757333)
now passes all eight local-fleet tests, including the real owned persistent
PocketIC session, plus strict selected library/test Clippy on Linux. The initial
offline cache failure remains retained; the later check used the unchanged
incoming lock recorded in `target/review-validation/checkpoint-reader-092/`
in Canic. No dependency changes or online fetch were performed by this work.
Source remains uncommitted; no function, method or type was removed. Native
macOS and broader adoption qualification remain separate.

Shared Tooling 0.2.4 `ffbf665b8481c36b2d9f4d988abec557c3485fa6` was inspected:
all required companions of Host's selected files are already present. No new
Host defect or release draft is justified by this check; its adopted snapshot
remains 0.2.2. No full gate, commit, push or release ran.

## Latest work: 2026-10-09, pending 0.9.2 tooling and CI fixes

Adopted Shared Tooling 0.2.2 `ee48bb37c98c771e77b92fd891f0757d8c1c8b99`,
verified against upstream main at inspection. The canonical exporter ran from a
clean private checkout, preserving the 85-file selection and sibling dirty work.
IC setup/check now consumes an unterminated final pin row; CI tool publication
uses exact-path rename so a late directory cannot redirect installation and a
late symlink's target remains untouched. No tools were downloaded or installed.

[Host #39](https://github.com/dragginzgame/ic-host-tooling/issues/39#issuecomment-6079897793)
now has a local fix: each pushed SHA receives its own CI concurrency group,
while superseded PR revisions remain cancellable. Triggers, native matrix and
gates are unchanged. Actionlint and event/group review pass; actual scheduling
needs observation after authorized pushes. Released 0.9.1 CI has passing Linux
native/MSRV jobs and queued macOS jobs at inspection. Older cancelled runs are
not repaired by this change; #37/#38 acceptance remains separate.

IC installer fixtures against adopted bytes and CI installer fixtures against
the identical committed shared helper pass under Bash 5 and existing Bash 3.2
on Linux. Snapshot verification, ShellCheck, selected documentation links and
whitespace checks pass. Downloads, executable identities and Darwin selections
are substituted in these fixtures; native macOS and real-installation evidence
are outstanding. Logs and reviewed source remain in `/tmp/ic-host-shared-022/`.

The bounded follow-up uses the unchanged Shared flow-convergence method at this
revision, with `docs/extraction.md` and AGENTS.md as local ownership constraints.
Ordinary/optional/private descriptor reads retain separate admission contracts;
streaming copy/chunk hashes already use one reader traversal. No further removal
is justified, and the prior measured copy specialization remains intact.
The compatible pending version is **0.9.2**. No function, method or type was
removed; Rust source, package versions, pins, dependencies and siblings are
unchanged. No compilation, full gate, commit, push or release ran in this batch.

## Latest review: 2026-10-09, released 0.9.1 consumer redundancy audit

Host production source is clean at `4a016053525fa710bc13f3aedbe85a471b78f6ed` (0.9.1), reported
pushed by the maintainer. Its [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37919257776)
was queued at inspection. The source delta contains the two reviewed cleanups
below; their prior focused Linux checks do not establish fresh native acceptance.

The bounded source/lock review traced host-facing artifact, read, publication,
lock and process paths in Canic, Testkit, Query, Backup, IcyDB, Toko Miner,
Auth, Blob Storage, Memory and Metrics. Canic's dirty root lock retains Host
artifacts/fs 0.8.10 through Backup 0.10.1 alongside direct Host 0.9.1; its
concurrent Query update to 0.52.0 removed the other observed old-generation path.
IcyDB retains all four Host 0.8.10 packages through Testkit 0.25.5
alongside direct 0.9.0. Qualified adoption of their updated dependency owners
removes those second generations; no Host compatibility API is needed.
Blob's concurrent work moved to Testkit 0.26.0 and one Host 0.9.1 generation
during inspection. These are observed local selections, not release/adoption proof.

[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6079604483)
carries the checkpoint response collector patch using existing
`artifact::read_reader`, preserving the 64 KiB bound and Capacity/native-I/O
projections. It passes read-only patch application checking. The selected Host
artifacts library builds locked/offline; three Linux comparison checks pass for
byte results/consumption across limits and short reads, original I/O causes and
malformed input preservation. The harness uses a substitute error enum: it is
not Canic compilation, lifecycle qualification or native macOS evidence.

[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307#issuecomment-6079604794)
narrows the earlier execution recommendation: `Command::output` already delegates
collection to the standard library. Retain it absent a demonstrated need for
output bounds, cancellation or Host cleanup guarantees; an adapter alone adds
complexity. The dependency convergence remains useful. Existing direct-child
communication permits no elapsed deadline if adoption becomes warranted;
consumer output limits, status projection and update-effect recovery stay local.

Retain Query confinement/alias checks, Backup private parents and crash barriers,
Blob partial-run evidence, Testkit domain-framed source hashes and consumer JSON
schemas. Ordinary raw SHA use is already owned by its dependency and is not a
reason for another Host abstraction. No new public API or release draft is
justified by this pass. No production code, sibling files, dependencies or symbols
were changed. Only the selected Host artifact build and standalone comparison
harness ran; no consumer compilation, managed operation, download or release ran.
Only this handoff and existing issues were updated. Patch and comparison evidence
remain under `/tmp/ic-host-091-consumer-audit/checkpoint/`. Native macOS
qualification and owning consumer acceptance remain separate from Host Linux
and standalone projection checks.

## Latest work: 2026-10-09, pending 0.9.1 capture-state cleanup

The second bounded cleanup removes separate stdout/stderr EOF flags from the
process exchange loop. `read_chunk` closes an exhausted pipe; absence now owns
the completion state. Remaining streams, child status, cancellation, output
bounds and observer/cleanup behavior retain their existing paths. No additional
function, method or type is removed. The compatible pending version stays 0.9.1;
the publication cleanup below is preserved.

All 55 focused `ic-host-process` tool-module tests and strict package/all-target
Clippy pass on Linux, including closed-stream deadlines, full-duplex capture,
overflow, observers and retained-child cleanup. Formatting and selected
documentation links pass. Native macOS qualification remains pending; no full
gate, version change, sibling edit, commit or release ran.

The read-only follow-up inspected gzip/archive framing, Wasm inspection,
executable resolution/admission, Git observations, path/private reads and child
ownership. Their remaining checks retain distinct contracts; no further removal
was justified. Released 0.9.0's [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37917141870)
is queued at inspection and does not qualify these dirty edits. Existing Shared
adoption issues [#37](https://github.com/dragginzgame/ic-host-tooling/issues/37)
and [#38](https://github.com/dragginzgame/ic-host-tooling/issues/38) remain open;
0.8.10's Linux/MSRV jobs passed but its macOS jobs were cancelled. The 0.9.0
delivery is now recorded on #38; no duplicate tracker was created.

## Latest work: 2026-10-09, pending 0.9.1 publication cleanup

Following released `715854b` (0.9.0), a bounded cleanup removes the private
`durable::FileCommitMode` and
`durable::supported::commit_with_producer_and_hook` from
`crates/ic-host-fs/src/durable/mod.rs`. Convenience entrypoints now pass their
existing publication/permission selections as `WriteOptions`; named producers
call `commit_path_with_options` directly. Public contracts, unsupported-host
errors, failure phases and the publication engine are unchanged. This is a
compatible internal cleanup, so the pending notes select **0.9.1**.

All 45 focused durable-module tests and strict package/all-target Clippy pass on
Linux. Native macOS qualification remains for CI; no full gate or release ran.
No sibling files, package versions, dependencies or lock selections changed.

The same review rejected composing `copy_reader` through `HashingWriter`:
optimized Linux probes were similar for full chunks but took about 40% longer
for seven-byte sink writes because hashing ran for every short write. The
artifact source was restored exactly. Probe binaries, source, measurements and
the rejected diff remain in `/tmp/ic-host-copy-cleanup/`; this synthetic result
is not a filesystem benchmark or macOS evidence. No other removal was justified
in the inspected read, process-exchange and response-decoding boundaries.

## Latest work: 2026-10-09, pending 0.9.0 Shared 0.2.0 adoption

The requested refresh adopts committed Shared Tooling 0.2.0
`8140e3dd1b44409d682c721889ab702f438c6a17` through its canonical exporter from
a clean private checkout, preserving the 85-file roster. Host's three PocketIC
pin rows are removed; remaining pins are unchanged. No maintained Host library,
CI or release path uses PocketIC, and no Testkit dependency is added. The retired
upstream checkers were never selected here. No function, method or type is removed.

Pending notes select **0.9.0**, since the five-tool bundle changes the advertised
setup/check contract. Rust APIs and package versions remain unchanged. Existing
six-tool bundles require explicit setup before offline verification; previous
bundles/receipts are preserved. No actual tool installation was performed.

Focused Linux checks pass: IC installer and evidence fixtures under Bash 5 and
existing Bash 3.2, Make command wiring, pin-matrix/declaration admission, snapshot
digests, ShellCheck and documentation links. Downloads/executable identities and
native-host selections in the fixtures are substituted; they establish refusal,
activation and retention behavior, not real-installation/macOS qualification.
Logs and source diff remain in `/tmp/ic-host-shared-020/`.
[#38](https://github.com/dragginzgame/ic-host-tooling/issues/38) owns delivery and
native acceptance. Package versions, Cargo.lock, Rust tool pins and index are
unchanged. No full gate, compilation, downloads, sibling edits, commit or release ran.

## Latest work: 2026-10-09, pending 0.8.10 Shared pinning admission

[#37](https://github.com/dragginzgame/ic-host-tooling/issues/37) is implemented
locally through the canonical snapshot exporter from a clean private checkout of
Shared 0.1.38 `926a20606591214ab29faa236b0b584e4857439e`. The 85-file roster is
unchanged. The refresh rejects multi-document pinning exception catalogs and
preserves checkout-local executable lookup during isolated formatting. Existing
installer additions and policy changes were reviewed together; no installer ran.
The compatible pending release is 0.8.10, with no Rust API or symbol removal.

Focused checks pass: snapshot digests, actual Host dependency declarations,
canonical dependency/hook fixtures, simulated Rust-tool installation fixtures,
Host formatting-hook adoption, ShellCheck and selected documentation links.
Dependency and installer fixtures also pass with existing Bash 3.2 on Linux.
The initial hook-adoption fixture moved a comment while deliberately unsorting
dependencies; its comparison failure is retained, and a dependency-only
perturbation passes. Logs and the exact source diff remain in `/tmp/ic-host-0810/`.
No real installation/download or Rust compilation was needed.

[Shared's exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37912382208)
passes Linux regression and lint/security; both macOS jobs remain queued.
Native macOS qualification of this Host candidate is also pending. Versions,
Cargo.lock, pins and index are unchanged. No full gate, sibling edit, commit,
push or release ran. Canic #501 still owns the direct Testkit URL-contract
convergence after Toko's successful downstream adapter proof; Toko #42 retains
its separate release-cache repair. Neither sibling was modified here.

## Latest work: 2026-10-09, authorized Testkit streaming check adoption

The maintainer explicitly approved editing Testkit for the proposed bundle-check
optimization, overriding the sibling-read-only default for this scoped change.
It is now implemented locally for compatible pending Testkit 0.25.6:
`check_bundle` uses existing Host `hash_gzip`, with one authenticated archive-read
helper shared with installation. No Host API or symbol was removed. The existing
dirty Testkit lock selects Host 0.8.9 and was preserved byte-for-byte.

Six focused provisioning tests, strict CLI Clippy, binary build, formatting and
selected documentation links pass on Linux. The freshly built CLI also verifies
the retained official 16.1.0 bundle with PATH tools disabled. Evidence and native
macOS/delivery limitations are recorded in Testkit's `docs/hosts.md` and
[issue #38](https://github.com/dragginzgame/ic-testkit/issues/38#issuecomment-6078004202).
No downloads, full gate, commit or release ran. Canic's active work and the wider
Shared PocketIC retirement remain untouched pending their qualification.

## Latest review: 2026-10-09, released 0.8.9 consumer audit

Host 0.8.9 is committed at `0464db5146be910a0f078447831fa2807072c75a`,
reported delivered by the maintainer. Its [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37905479815)
is queued at inspection; #36 retains native qualification. This supersedes the
local-only delivery statement below without relabelling prior test evidence.

Read-only source review traced artifact checks, publication and process adapters
in Canic, Testkit, Query, Backup, Memory, Toko Miner and IcyDB. Root locks select
Host 0.8.8 in Canic/Testkit/Query/Toko and 0.8.9 in Backup/Memory/IcyDB. Dirty
consumer work is preserved; these selections do not prove released adoption.
Canic's latest #458 evidence records local shared hashing/foreground adoption;
Testkit's released 0.25.5 includes the shared lock opener and provisioning CLI.
Its native CI also remains queued; Cargo-output convergence remains deferred.

A concrete check-only allocation reduction is recorded on
[Testkit #38](https://github.com/dragginzgame/ic-testkit/issues/38#issuecomment-6077681677):
use existing Host `hash_gzip` in `check_bundle` instead of decoding up to 512 MiB
solely to hash and discard it. Authenticate the compressed archive first through
one helper shared with installation; retain all limits and admission contracts.
The reviewed provisioning file matches Testkit `311c39b9a7f9cc04fec050797c3324842e338328`.
This is source evidence, not an implemented or measured consumer optimization.

The larger simplification remains Testkit's PocketIC ownership handoff, tracked
by Testkit #38 and [Shared #76](https://github.com/dragginzgame/shared-tooling/issues/76).
Existing consumer error projection, domain-framed hashes and confinement remain
distinct contracts. No new Host API, hard cut or release draft is justified.
Shared Tooling is now committed at `be550af` with further dirty installer work;
no snapshot adoption was performed. No source edits, compilation, downloads,
sibling mutations or release effects ran in this audit. Only this handoff and
the existing consumer issue were updated; documentation links pass.

## Latest work: 2026-10-09, pending 0.8.9 release cache preparation

The compatible **0.8.9** batch fixes
[#36](https://github.com/dragginzgame/ic-host-tooling/issues/36) locally. Release
preflight prepares the selected dependency graph with `cargo fetch --locked`
after admission; standalone verification retains `--offline`. Explicit Cargo
offline settings remain authoritative. Fetch failures preserve the adapter's
status and stop before validation or version preparation. Saved recovery intent
and source selection still belong to the unchanged shared runner.

Real Cargo probes with separate empty caches confirm offline no-dependency
metadata admission succeeds and offline fetch fails on a missing locked input.
Focused Linux adapter fixtures pass for successful preparation, explicit offline
refusal, network failure, source mismatch and unchanged source metadata. Fetch,
setup and Git effects are substituted in those fixtures; no online fetch was
performed. The shared release-runner command-stub suite, Bash syntax, ShellCheck,
snapshot verification and selected documentation-link checks pass. Evidence is
retained in `/tmp/ic-host-089/`. Native macOS qualification of this candidate
remains pending; released 0.8.8 CI is not candidate evidence.

The adopted flow-convergence and module-surface audit methods were applied to
Host and the seven consumers identified below. Plain IO and typed publication
callers, direct-child and group ownership, and consumer-specific aggregate hashes
retain distinct live contracts. No further removable production owner or public
API was confirmed; no hard cut or symbol removal is justified by this pass.
Existing consumer adoption remains with Canic #458 and Testkit #35/#36.

Shared Tooling's committed `3d33cd2` does not require a selected-helper refresh;
its pending cache-preparation guidance remains dirty and was not adopted.
Versions, lockfile, pins and snapshot are unchanged. No Rust compilation, full
local gate, sibling edit, download, commit, push, release or publication ran.

## Latest review: 2026-10-09, released 0.8.8 consumer convergence

Host 0.8.8 is released at `ccfd7724dd31c14cfbb8ae434f683babfeabf906`.
[Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37901415314)
passes Linux x86-64, macOS 15 Intel/Apple Silicon and Rust 1.88. Host issues
[#33](https://github.com/dragginzgame/ic-host-tooling/issues/33),
[#34](https://github.com/dragginzgame/ic-host-tooling/issues/34) and
[#35](https://github.com/dragginzgame/ic-host-tooling/issues/35) are closed with
that acceptance evidence. This supersedes their pending delivery statements
below; it does not alter the failed/cancelled 0.8.7 run's historical evidence.

A bounded source review using the adopted code-hygiene method at Shared Tooling
`ddd3e1c` traced process exchange/cleanup, no-follow reads, typed publication,
executable resolution and bounded response/Candid handling. No new actionable
Host defect or reusable API gap was confirmed. This is source review using the
released CI evidence, not a fresh execution or whole-system correctness claim.

The inspected root locks in Canic, Testkit, Query, Backup, Memory, Toko Miner and
IcyDB each select only Host 0.8.8 for their included Host crates. Several consumers
have active dirty work: lock inspection does not establish released adoption.
Canic's dirty work on `c4c046f94` now uses the shared no-follow hasher and direct-child
communication; [Canic #458](https://github.com/dragginzgame/canic/issues/458) owns
its remaining qualification and delivery. Testkit's cache-lock opener now matches
committed 0.25.4 `085756dd0e0e2304de7e4a0b6b887201918646b6`; its
[#35](https://github.com/dragginzgame/ic-testkit/issues/35) retains consumer
acceptance. The Cargo reader refactor remains explicitly deferred in
[#36](https://github.com/dragginzgame/ic-testkit/issues/36).

No new release draft or production edit is justified by this pass. Shared Tooling
has committed `3d33cd2`; its PocketIC checker fix affects an unselected helper,
and its further installer changes remain dirty. No snapshot refresh, dependency
change, compilation, sibling edit, commit, push or release ran during this review.

## Latest work: 2026-10-09, pending 0.8.8 foreground ownership and CI fixture repair

Host 0.8.7 is delivered at `8edce53c43bb872cd4aaa15659e37f0236adc203`, reported
live by the maintainer. Its [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37899468089)
passes MSRV and macOS 15 ARM, but Linux failed in the executable-admission fixture
and Intel macOS remains queued. Issues #32/#33 therefore remain open through
qualification; delivered source is distinct from a passing native gate.

The compatible **0.8.8** batch implements
[#34](https://github.com/dragginzgame/ic-host-tooling/issues/34):
`OwnedChild::spawn_direct` is promoted from crate-private to public, retaining its
existing implementation and synchronous KillAndWait policy. It preserves command
IO/context and inherited or explicitly selected process groups, and owns only the
direct child. Communication uses the same engine; cancellation, overflow, unwind,
ordinary waiting and Drop cannot acquire group ownership. Descendants, terminal
control, signal handling and descendant-held pipes remain caller-owned.
No function, method or type is removed, renamed or duplicated.

Canic's latest [adoption evidence](https://github.com/dragginzgame/canic/issues/458#issuecomment-6076429624)
shows its JSON publication convergence implemented in dirty work, while foreground
replica adoption still depends on preserving terminal-group membership. The
inspected `icp/run.rs` matches Canic base `c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`.
The new public constructor supplies that mechanism; actual interactive Ctrl-C
qualification and consumer deletion remain in Canic #458. Testkit's reader-thread
adoption remains in #36. No sibling source was changed or compiled.

The Linux release failure is tracked and repaired locally in
[#35](https://github.com/dragginzgame/ic-host-tooling/issues/35). The real staged
executable test now runs alone in an exact-test subprocess, retaining execution
before replacement and final installed execution. Parallel fork-time inheritance
of the writable staging descriptor explains the observed ETXTBSY by source
inspection; no offending descriptor interleaving was captured. The pre-fix exact
test passed locally, while the failed CI log records 66 passes and one failure.
Production admission/publication code is unchanged; no retry or sleep was added.

Locked/offline Linux validation passes: all 76 process library tests, all 67
filesystem library tests plus five further runs with 32 threads, strict selected
package all-target Clippy, Rust 1.88 all-target compilation and process Rustdoc
with warnings denied. Direct-child fixtures prove inherited/configured group
identity and another live group member surviving success/cancellation/overflow/
observer unwind/Drop. This is process-group evidence, not interactive terminal
qualification. Formatting and documentation links pass. Evidence and the failed
release log remain in `/tmp/ic-host-088/`. Native macOS qualification of this
candidate remains pending; the failed released run is not relabelled as passing.

Shared Tooling remains at committed `ddd3e1c` with additional dirty work excluded.
Versions, lockfile, tool pins and snapshot are unchanged. No full local gate,
downloads, sibling edits, commit, push, release, publication or CI rerun occurred.

## Latest work: 2026-10-09, pending 0.8.7 no-follow streaming hashes

Host 0.8.6 is delivered at `9f3d9a83def91030056c78e44c9efaa489be7d12`.
Its [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37896909262)
passes Linux x86-64, macOS 15 Intel, macOS 15 Apple Silicon and Rust 1.88 MSRV.
This completes Host [#31](https://github.com/dragginzgame/ic-host-tooling/issues/31);
Testkit and Canic adoption remain separate consumer obligations.

The compatible **0.8.7** draft implements
[#33](https://github.com/dragginzgame/ic-host-tooling/issues/33) locally.
`read::hash_file_no_follow` composes the existing Unix no-follow/nonblocking opener,
descriptor metadata admission and constant-memory artifact hasher. It returns the
existing identity/error types and independently bounds metadata and stream bytes.
Following hashes keep their existing behavior; trusted ancestors, writer custody,
exact expected-length checks and domain errors remain with consumers. No function,
method or type is removed by the hashing change. Versions, dependencies and pins
are unchanged; the shared snapshot selection change is described below.

The three Canic hashing callers still match committed base
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`, despite unrelated dirty work.
Their shared opening/hash mechanics can converge after release, but build-reuse
prechecks and local-fleet early-size versus streamed-overflow classifications
must retain their current domain contracts. Consumer adoption/deletion is tracked
in [Canic #458](https://github.com/dragginzgame/canic/issues/458).

Focused locked/offline Linux validation passes: 15 read-module tests, strict
filesystem all-target Clippy, Rust 1.88 all-target compilation, warning-denied
Rustdoc, selected formatting and documentation links. Coverage includes real
regular files, empty/exact/overflow limits, final and ancestor symlinks, original
missing-file errno, special-file refusal and a FIFO without a writer. A controlled
growth fixture changes the actual file between descriptor metadata admission and
the shared hasher, without racing or adding a production hook. Evidence remains
under `/tmp/ic-host-087/`. This candidate still requires native macOS qualification.
No full gate, sibling edits, downloads, commit, push, release or publication ran.

Host [#32](https://github.com/dragginzgame/ic-host-tooling/issues/32) is now
implemented locally in the same compatible 0.8.7 batch. The canonical Shared
Tooling repository confirms committed 0.1.33
`ddd3e1c01ba8aab13a56277e05679e43a8a9d88a`; a clean private checkout supplied the
canonical exporter. The 85-file snapshot omits `scripts/dev/cloc-tooling.pl` and
updates the common Make diagnostic/fixtures for that optional report. Host help
and README direct fleet reporting to Shared Tooling; local LOC, setup, offline
checks and their integration coverage remain. No consumer workflow or schedule
calls the removed reporter. Shared Tooling retains its implementation and tests.

The deleted reporter's private functions are `usage`, `capture`, `read_file`,
`write_file`, `safe_path`, `is_linked`, `in_scope` and `load_snapshot`. Their only
local owner was the removed file; use Shared Tooling's central report instead.
No vendored helper was patched. Newly adopted PocketIC handoff guidance does not
change this consumer's existing pin matrix or install a replacement.

Focused Linux checks pass: 85-file snapshot integrity, common Make command
fixtures with substitute installers/reports, actual offline local `make cloc`
and `make host-tools-check`, the expected omitted-fleet diagnostic, selected
ShellCheck/Bash syntax and documentation links. Tool pins, Cargo manifest and
lockfile are byte-identical to the pre-refresh inputs. Evidence is retained in
`/tmp/ic-host-087-shared/`; the prior Rust checks remain applicable because this
addition changed no Rust sources. Shared Tooling's new exact-source CI was queued
at initial review; native qualification of this Host candidate remains separate.
No sibling edit, download/install, full gate, commit, push or release occurred.

## Latest work: 2026-10-09, pending 0.8.6 communication without a deadline

Host 0.8.5 is released at `1cad3253096b6eb67be5187209e7fb606593c501`,
reported delivered by the maintainer. The compatible **0.8.6** draft implements
[#31](https://github.com/dragginzgame/ic-host-tooling/issues/31) locally.
`CommunicationLimits` offers an explicit optional deadline for the two existing
owned-child communication functions. They also accept existing `OutputLimits`
arguments, preserving their finite behavior. Capture and executable admission
retain that mandatory finite contract. Both choices use the existing I/O engine;
no function, method or type is removed, aliased or deprecated.

No-deadline communication retains byte bounds, cancellation, fair I/O and cleanup,
including observer unwinding. It may wait indefinitely for exit or pipe EOF;
consumers retain successful-exit disposition, cancellation and independent
cleanup timing. Testkit build adoption/deletion remains in
[#36](https://github.com/dragginzgame/ic-testkit/issues/36), and Canic foreground
replica adoption remains in
[#458](https://github.com/dragginzgame/canic/issues/458). Siblings remain read-only.

Focused locked/offline Linux checks pass: 55 process-tool tests, strict selected
package all-target Clippy, Rust 1.88 all-target compilation and warning-denied
Rustdoc. Fixtures cover both deadline choices for full-duplex IO, live binary
observation, bounded overflow, silent cancellation, descendant-pipe cleanup and
callback unwinding. A real-child fixture supplies three hours of elapsed time
to the engine; this is injected-clock evidence, not a multi-hour build run.
Invalid finite deadlines preserve owner/pipes, including through the new type.
Initial Clippy findings and final passing results remain in `/tmp/ic-host-086/`.
Native macOS Intel/Apple Silicon qualification is pending for this candidate.
Manifests, lockfile, pins and snapshot remain unchanged. No full gate, download,
commit, push, release, publication or sibling source edit occurred.

## Latest work: 2026-10-09, pending 0.8.5 installer reuse

Host 0.8.4 is delivered at `97187b2a46d6f8a6964224a36a133d858ef0d223`.
Its [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37818647474)
passes Linux x86-64, macOS 15 Intel, macOS 15 Apple Silicon and Rust 1.88 MSRV.
This supersedes the pending delivery/native statements in the historical entries
below. Completed Host issues
[#27](https://github.com/dragginzgame/ic-host-tooling/issues/27),
[#28](https://github.com/dragginzgame/ic-host-tooling/issues/28),
[#29](https://github.com/dragginzgame/ic-host-tooling/issues/29) and
[#30](https://github.com/dragginzgame/ic-host-tooling/issues/30) are closed with
that acceptance evidence. Testkit's separate adoption remains consumer-owned.

Query's committed `edd68020d9ef8e76a241e89daf77e51155a92f57` also passes
[native CI](https://github.com/dragginzgame/ic-query/actions/runs/37823182436)
on all three hosts, including checks, MSRV and canister jobs. The committed
complete gate runs the actual harness through its Host-backed process helper.
This completes [#5](https://github.com/dragginzgame/ic-host-tooling/issues/5).
Evidence is job/step results plus committed gate wiring; attempted per-job log
downloads returned empty files, so no new individual fixture counts are claimed.

The compatible **0.8.5** draft adopts committed Shared Tooling
`9af82393c620e486578febed74a648523725c234`, verified in its canonical GitHub
repository. A clean private checkout supplied the canonical exporter; the
existing 86-file selection is unchanged. The IC installer compares validated,
sorted pin records to reuse bundles after comment/order changes without
rewriting receipts. Version/digest/host changes and malformed retained or caller
pins still reject reuse before execution
([shared #79](https://github.com/dragginzgame/shared-tooling/issues/79)).
The local IC pin file remains byte-for-byte unchanged, including PocketIC 16.0.0.
Package versions, lockfile and all Rust production code are unchanged. No
function, method or type is removed. Uncommitted Shared Tooling work is excluded.

Focused Linux checks pass: synthetic IC installer/retention/activation fixtures,
86-file snapshot integrity, unchanged pin bytes, selected ShellCheck/Bash syntax,
dependency declarations and documentation links. Synthetic host identities and
download payloads do not qualify native macOS; this candidate needs its own CI.
Evidence is retained under `/tmp/ic-host-085-audit/`. The source review of process
communication, bounded artifact writers/archive admission and filesystem lock/
read/publication boundaries found no additional confirmed defect in this pass.
No Rust compilation, full local gate, real tool installation/download, sibling
edit, commit, push or release occurred for this batch.

## Latest work: 2026-10-08, pending 0.8.4 lock opening and shared tooling

Host 0.8.3 is released at `67d031222073f23ad437b45053156e229f86a016`,
reported live by the maintainer. This compatible **0.8.4** batch implements
[#29](https://github.com/dragginzgame/ic-host-tooling/issues/29) and
[#30](https://github.com/dragginzgame/ic-host-tooling/issues/30) locally.

Existing lock entries now skip durable publication staging. Missing entries
still create parents/file once and converge on the same no-follow/nonblocking
open and descriptor checks. Existing bytes, lock acquisition policy and typed
errors are retained. A concurrent removal may return NotFound without retry.
An actual Linux syscall probe of three existing-file opens changed from three
temporary files/fsyncs/no-replace attempts/unlinks to zero, and read/write opening
under a mode-0555 parent now succeeds. This measures effects, not elapsed speed.
The controlled prior failure is retained in `/tmp/ic-host-audit-083/`; current
probe, source and checks are under `/tmp/ic-host-084/`.

Shared Tooling was exported canonically from a clean private checkout of
`4e274a2219c0b0cc3af68ec65658b373253518fb`. The final 86-file selection includes
the new release-source checker and maintenance catalog/companions. No scheduler
is activated. Host's adapter delegates source inventory to the shared checker,
retaining its allowed paths for each phase. Consumer release-runner tests now
simulate Git effects; upstream retains real tracking qualification separately.
The exact previous `ci/ic-tools.tsv` bytes are preserved as a documented local
selection outside the snapshot, avoiding an incidental PocketIC pin upgrade.
Upstream dirty changes, new Cargo installer qualification and optional npm checks
are not adopted as active Host tasks. Package versions and lock selections stay
at the released state. No function, method or type was removed; the private
adapter inventory loop was replaced inside its existing `admit_files` function.

Focused locked/offline Linux validation passes: 45 durable tests (including
concurrent creation, read-only parents and device/FIFO/symlink refusal), strict
filesystem Clippy, Rust 1.88 all-target compilation and warning-denied Rustdoc.
Host adapter/publication fixtures, shared simulation-runner and source-checker
fixtures, both installer suites, selected ShellCheck, declaration pins and
snapshot integrity pass. Installer payloads/downloads and release effects are
substituted; source-checker fixtures use a disposable Git index without commits.
Initial checker invocations with an unsupported `--root` flag are retained
separately from the successful `--consumer` runs. Documentation links, formatting
and diff checks pass. Native macOS Intel/ARM qualification remains pending for
this candidate. No full gate, real tool download/install, commit, push, release,
publication, schedule activation or sibling edit occurred.

## Previous work: 2026-10-08, pending 0.8.3 consumer composition APIs

Host 0.8.2 is released at `92bd2fecc71124b562e227a32a67644e1e5e34b7`,
reported live by the maintainer. The tree was clean before this batch.
The compatible **0.8.3** draft adds live output observation
([#28](https://github.com/dragginzgame/ic-host-tooling/issues/28)) and unlocked
regular lock-file admission
([#27](https://github.com/dragginzgame/ic-host-tooling/issues/27)). Versions,
dependency selections and the Shared Tooling snapshot remain unchanged.

`communicate_child_with_observer` shares the existing bounded I/O engine and
reports only retained bytes, including overflow prefixes. Callback unwinding
closes pipes and attempts the selected cleanup before resuming the panic, even
when a caller catches it while retaining the borrowed child. Consumers retain
event schemas, heartbeat scheduling and output/time limits. The private
`durable::open_regular_lock_file` is renamed/promoted to
`open_regular_lock_file_with_parents`; acquisition moves to its existing callers.
There is one opener, with unchanged admission and path-lock behavior, and no
compatibility alias. No other function, method or type was removed.

Focused locked/offline Linux validation passes: 54 process-tool tests and 43
durable-filesystem tests, strict selected-package all-target Clippy, Rust 1.88
all-target compilation and warning-denied Rustdoc. Coverage includes live binary
output, fair simultaneous streams, bounded observed prefixes, silent cancellation,
descendant-held pipes, callback unwinding with a retained owner, shared locking,
explicit unlock with a live clone and special-file refusal. The initial Clippy
line-count/assertion failures and corrected results remain in `/tmp/ic-host-083/`.
Native macOS Intel/Apple Silicon qualification remains pending for this candidate.

The maintainer's subsequent filesystem run failed
`lock_errors_preserve_admission_and_original_io_causes`. The added `/dev/null`
case depended on permission to create sibling staging under `/dev` before the
non-regular-file check. An initial exact rerun failed; a diagnostic rerun passed,
so the prior device-fixture result was environment-dependent. A replacement Unix
socket fixture was also rejected by the sandbox and was removed. The final test
uses only its existing owned directory/symlink/FIFO fixtures, with path/error
diagnostics. Production behavior is unchanged and no symbol is removed. All 64
filesystem library tests and strict filesystem Clippy pass locked/offline on
Linux. Attempts and final results are retained in `/tmp/ic-host-083-admission/`.
This is still pending 0.8.3; native macOS qualification remains separate.

Testkit adoption/deletion remains separately tracked in
[#35](https://github.com/dragginzgame/ic-testkit/issues/35) and
[#36](https://github.com/dragginzgame/ic-testkit/issues/36). The Canic CLI to Toko
parser-deletion route remains consumer-owned; Host's Wasm composition API is
already sufficient. Sibling source was not modified. No full gate, dependency
download, commit, push, release or publication was performed.

## Previous work: 2026-10-08, released 0.8.1 and pending 0.8.2 macOS fixture repair

Host **0.8.1** is released at `973f00a029dd873c242a4d06e9f8d2d5172ff0df`;
the local tag and GitHub main agree, and the maintainer reports it live. The
working tree was clean before this continuation. This supersedes the pending
0.8.1 statements below. The initially absent
[exact-source 0.8.1 CI run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37799978288)
now passes Linux and MSRV; macOS ARM is running and Intel remains queued.
Registry publication is reported by Query's
[consumer check](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6063007687),
not independently rechecked here.

Both macOS 15 architectures in the older
[0.8.0 CI run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37792592340)
failed the same process fixture: `/bin/true` is absent, so spawn returned NotFound
before testing cleanup. Each recorded 67 process tests passed and one failed;
Linux and MSRV succeeded. The offending fixture was unchanged in 0.8.1.

The compatible **0.8.2** local correction uses `/bin/sh -c 'exit 0'`, matching
existing portable fixtures. It retains the real child, deliberate external reap,
original cancellation/output and separate TERM/KILL/reap error assertions under
[#5](https://github.com/dragginzgame/ic-host-tooling/issues/5). Four focused process
boundary tests, strict process Clippy, the exact regression on Rust 1.88,
formatting, changed-document links and diff hygiene pass locked/offline on Linux.
Failed native logs remain in `/tmp/ic-host-080-failed-ci.log`; new evidence is in
`/tmp/ic-host-082/`. Native Intel/Apple Silicon qualification remains pending.
No production code, public API, dependency selection or package version changed;
no function, method or type was removed.

Query now reports its actual process replacement implemented locally for pending
0.50.1, updated to published Host 0.8.1, with 37 real Linux harness fixtures. This is
[consumer-reported evidence](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6063097729),
not a new consumer build or native qualification here. Shared Tooling still has
committed HEAD db039347; its installer link-byte fix for
[#75](https://github.com/dragginzgame/shared-tooling/issues/75) remains dirty and
was not adopted. The issue completion review closed
[#23](https://github.com/dragginzgame/ic-host-tooling/issues/23),
[#25](https://github.com/dragginzgame/ic-host-tooling/issues/25) and
[#26](https://github.com/dragginzgame/ic-host-tooling/issues/26) with delivered
evidence. Both older macOS jobs passed the unchanged path fixtures before their
unrelated process failure; the example/buffer sources match the qualified hashes.
[#24](https://github.com/dragginzgame/ic-host-tooling/issues/24) retains its explicit
native lock-qualification requirement; #5 retains fixture delivery and consumer
acceptance. Canic #481/#482 receive the delivered recipe/nonblocking-lock handoff.
No new Host API gap was established. Siblings remain read-only. No full
gate, download, commit, push, release or CI rerun/dispatch occurred.

## Previous work: 2026-10-08, pending 0.8.1 local Rust improvements

The compatible **0.8.1** batch now also implements
[#24](https://github.com/dragginzgame/ic-host-tooling/issues/24),
[#25](https://github.com/dragginzgame/ic-host-tooling/issues/25) and
[#26](https://github.com/dragginzgame/ic-host-tooling/issues/26) locally on released
0.8.0 `fc74f679c7503ef9dd2db8fbd893c5c5907c72c4`. The previously dirty installer
snapshot and documentation changes below are preserved. Package versions, locked
dependency selections and shared snapshot selection are unchanged by this batch.

`try_lock_regular_file_with_parents` delegates to the existing opener and attempts
one nonblocking exclusive lock, retaining native WouldBlock on contention.
Artifact reads/chunk vectors and captured process output use bounded geometric
capacity requests; Candid normalization measures and allocates once. The compiled
`inspect_install_limits` example inspects once, explicitly selects the reference
limits, keeps raw budgets separate, and computes its outcome before text/JSON
rendering. It runs with the pure `ic-limits` feature profile. No function, method
or type was removed, and no public compatibility contract was broken.

Focused locked/offline Linux checks pass: 42 durable tests, 46 artifact-module
tests, 50 process-tool tests, 9 Candid tests, 3 actual example tests and 2 IC limit
tests. Cases cover independent-process contention before owner release,
close-on-exec, preserved bytes and special-file rejection; fragmented capacity
bounds; 50,000 definitions plus an import, imported globals, exact/+1 body limits,
malformed/truncated input, raw admission and text/JSON outcome parity. Selected
strict Clippy, Rust 1.88 library/example compilation, new-API Rustdoc, feature
graph isolation, manifest ordering and documentation links pass. Initial lint
failures and their corrected results are retained, not relabeled as passes.

The same scratch counting-allocator probe changed allocation/reallocation calls
from 65,536 to 17 for a 64 KiB one-byte-fragment read, 10,000 to 1 for Candid lines,
and 4,096 to 13 for one-byte chunk digests. This measures requests, not elapsed
runtime or copied bytes. No unsafe allocator is added to repository code.
Evidence is in `/tmp/ic-host-081-local/`; the baseline probe remains under
`/tmp/ic-host-local-review/`. Native macOS Intel/Apple Silicon qualification and
consumer adoption remain pending. Siblings remain read-only; no full gate,
download, commit, push, release or publication ran.

## Previous work: 2026-10-08, released 0.8.0 and pending 0.8.1 installer fixes

Host 0.8.0 is released at `fc74f679c7503ef9dd2db8fbd893c5c5907c72c4`;
the maintainer reports it live. Its
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37792592340)
passes Linux and MSRV; both macOS 15 jobs remain queued. This supersedes the
pending/unpublished-0.8 statements below without claiming native acceptance.
The older 0.7.1/0.7.2 runs were cancelled with native jobs incomplete; only
0.7.0 has previously recorded complete native evidence in this handoff.

The compatible local **0.8.1** batch canonically adopts committed Shared Tooling
`db039347d2372b877c1c46dcdd2b5c3aa9412009` from a clean private checkout.
The 72-file selection is unchanged; concurrent dirty upstream 0.1.28 work is
excluded. Relative host/IC/Rust installer operands now resolve independently of
CDPATH, and fixture companion declarations reject incomplete exports. The shared
selector documentation describes the existing owner. No Rust API/runtime,
manifest/lock, package-version, tool-pin or CI-upload-wiring change is made.

The released host installer reproduced a two-line consumer path and exit 1 from
its parent using `CDPATH="$PWD"` and `--consumer ic-host-tooling --check`. The
same command passes after adoption against the existing pinned tools. Focused
host/IC/Rust installer fixtures pass with substituted payloads/Cargo. Three
disposable incomplete exports reject the actual missing companion edges before
creating any selected files or manifest. Snapshot integrity, selected ShellCheck
and Bash syntax, dependency declarations, documentation links and diff hygiene
pass on Linux. Logs and the failed control remain in `/tmp/ic-host-081/`.
Candidate native macOS qualification remains separate. Adoption evidence is on
[shared #73](https://github.com/dragginzgame/shared-tooling/issues/73#issuecomment-6062166982).

Read-only consumer snapshots record Canic's direct Host 0.8 plus transitive 0.7.2
through published Backup 0.8.0, Query 0.50.0 and Testkit 0.24.0. Query and Testkit
are concurrently updating their dirty manifests/locks to Host 0.8; that is not
published graph convergence. Canic's observed error fixture still needs
`term_error: None`, reported on
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6062205655).
Testkit's public Host reexports require its next minor 0.25.0 if it retains the
0.8 selection; both observed dirty notes still said 0.24.1. The compatibility
finding is on [Testkit #32](https://github.com/dragginzgame/ic-testkit/issues/32#issuecomment-6062205183).

[#5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6062206243)
now records the delivered Host cleanup API. Query still has no process dependency
or actual runner replacement; its artifact/fs update does not establish that
integration. Source/manifest/lock snapshots under `/tmp/ic-host-081/consumers/`
are point-in-time inspection, not consumer compilation or native qualification.
No additional Host abstraction was justified by this review. No function,
method or type was removed. Siblings remain read-only; no full local gate, tool
download, commit/push, release or publication ran.

## Previous work: 2026-10-08, released 0.7.2 and pending 0.8.0 bounded cleanup (superseded)

Host 0.7.2 is released at `8236b506307d33f7c34f56d6e0ba9db4300792d7`;
the maintainer reports it live. Its
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37789259299)
passes Linux and Rust 1.88; both macOS 15 jobs were queued at inspection. This
supersedes the pending-0.7.2 statements below. Registry availability was not
independently checked, and this release does not qualify the new candidate.

The local **0.8.0** batch addresses the concrete Query timing requirement under
[#5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6061981009).
`OwnedChild::spawn_with_cleanup` accepts `CleanupPolicy::TermThenKill` with
caller-owned grace and reap durations. TERM preserves the leader through grace;
KILL still reaches the group after an early leader exit. Nonblocking reap
observations use a single remembered allowance across explicit termination and
Drop. Expiry retains ownership for caller recovery; dropping can leave an
unreaped child until parent exit. No background reaper or hidden blocking wait
is introduced. The existing owner and communication engine remain authoritative.

`CleanupError` and `ExecutionError` add a separate `term_error` field. Update
public struct literals/exhaustive destructuring, using None for existing
immediate-KILL fixtures; this requires the pre-1.0 minor release. Existing
spawn/capture defaults, explicit natural waits and successful handoff retain
their contracts. No function, method or type was removed. Root/detail notes,
README, extraction contract and the explicit documentation check cover 0.8.0.
Manifests, lock selections, tool pins and the Shared Tooling snapshot remain
unchanged at the released selections; no package version mutation ran.

Focused locked/offline Linux validation passes: 68 process tests, one API example
compile, strict all-target Clippy, Rust 1.88 all-target check, warning-denied
Rustdoc and all-feature/all-target tools-consumer compilation. Cases exercise
TERM-exited leaders with TERM-ignoring descendants, sole zombies, held pipes,
blocked stdin, cancellation/timeouts, unwinding and handoff. Reap-expiry coverage
substitutes already-dispatched escalation around a real running child; it does
not induce an unkillable native task. External reaping supplies real ownership
errors, not native permission-denial evidence. The first Clippy attempt's empty
assertion lint and corrected final results remain in `/tmp/ic-host-080-cleanup/`.
Documentation links and diff hygiene pass. Candidate macOS qualification is pending.

Query is now released 0.50.0 at `bd583e2b6baa7570be363b2948345c57582c3f31`,
still using its Python process owner and published artifact/fs dependencies.
Its current cleanup implementation/fixtures were read, not replaced or executed.
Actual Query helper, receipt and signal integration, deletion and native acceptance
remain under #5; the old private bridge is not an apply-ready patch for this source.
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6061981427)
records its one observed ExecutionError test-literal update on eventual 0.8
adoption, separately from its concurrently changing dependency graph.
Siblings remain read-only. No full local gate, tool download, new unsafe code,
commit/push, release or publication occurred.

## Previous work: 2026-10-08, released 0.7.1 and pending 0.7.2 path fixes (superseded)

Host 0.7.1 is released at `410fee7c309e781edf6a361f0e480d71b7c11e5a`.
The maintainer reports it live. Its
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37776708008)
passes Linux and MSRV; both native macOS jobs were queued at inspection. This
supersedes the pending-0.7.1 statements below, without claiming registry checks
or native qualification of this new working tree.

The compatible **0.7.2** batch fixes inherited-CDPATH and trailing-newline
checkout handling at Host-owned adapter/publication/feature entrypoints
([#23](https://github.com/dragginzgame/ic-host-tooling/issues/23)). The old
read-only adapter version command fails with exit 127 under CDPATH. The old
publication adapter also fails the new newline-checkout fixture; both controls
and final passing results remain in `/tmp/ic-host-072-paths/`. The first expanded
publication fixture exposed its own line-oriented evidence-path parsing; it
now inspects the actual attempt directories. No production function was removed.

The canonical exporter adopted committed Shared Tooling 0.1.27
`b866d41041a1986eeec95bde9af4c6ba0853d2e3` from a clean private checkout.
The 72-file selection includes four required new installer-evidence fixture
inputs. Their missing upstream companion declarations are reported in
[shared #73](https://github.com/dragginzgame/shared-tooling/issues/73).
The composite action is a fixture dependency, not newly wired Host CI uploads.
AGENTS.md and snapshot digests agree. Notes cover the path repair, installer
evidence qualification and dotted snapshot-name LOC correction.

Focused Linux checks pass: release adapter, publication, feature graphs, shared
entrypoint paths, release runner (including disposable real-Git tracking cases),
both installer suites with substituted payloads/downloads, tooling LOC, snapshot
integrity and dependency declarations. Selected Bash syntax and ShellCheck pass;
the runner's unselected PR companion requires SC1091 exclusion. Earlier lint
attempts without that selection context are retained. Native macOS qualification
of this candidate remains pending; upstream exact-source CI was also queued.

New [Query feedback on #5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6060063432)
explicitly requires TERM grace and bounded reaping. Immediate KILL and synchronous
reaping cannot replace its current runner. This is the next separate process
owner change; the tooling batch makes no runtime/API change or adoption claim.
Manifests, lock selections and package versions remain 0.7.1. No sibling edits,
full local gate, real tool download, commit/push, release or publication occurred.

## Previous work: 2026-10-08, released 0.7.0 and pending 0.7.1 tooling refresh (superseded)

Host 0.7.0 is pushed at `491fc0e231b9650526f5f57b9ab7b1f62f02218c`.
[Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37773664766)
passes Linux x86-64, Rust 1.88 and native macOS 15 Intel/Apple Silicon. This
supersedes the candidate-native statements below. Registry publication was not
independently checked in this continuation; actual consumer adoption remains
separate under [#5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6059575927).

The compatible **0.7.1** local batch adopts committed Shared Tooling 0.1.26
`75a8a60f49cec11d3f6aecab5c977029c42cc549` through its canonical exporter
from a clean private checkout. The existing 68-file selection is unchanged;
upstream dirty edits are excluded. Matching local tracking refs now refresh
after confirmed release/resume without overwriting concurrent/symbolic refs.
Snapshot updates can advance a verified unchanged uncommitted export, check
explicit additions/companions and refuse selected-path/index races. Tooling LOC
includes bin/ and unborn repositories. AGENTS.md and exact snapshot digests agree.

The new real-Git fixture against Host's old runner reproduces successful delivery
followed by `ahead 1`; all 19 tracking cases pass with the adopted owner. Focused
Host adapter/receipt and Make wiring fixtures, snapshot distribution, tooling LOC,
selected ShellCheck, Perl syntax and snapshot verification pass. An initial LOC
run lacked cloc in PATH; the existing Host tool path resolves that without a
download. Source/controls/failures/final logs remain in
`/tmp/ic-host-audit-20261008-03/`. No real release effects were exercised.

The [new frozen convergence audit](../reports/audits/2026/10/08/host-reuse/03/report.md)
records current sibling identities, dirty adoption and fresh Toko Wasm failures.
Canic's working publication/inspection code and Toko Miner's encoder now use
shared owners. The remaining parser deletion route is Canic #481 → Toko #1791;
both owners received fresh evidence. Query still needs its actual process/receipt
integration decision; its TERM grace/bounded reap differ from Host's owner.
[Testkit #32](https://github.com/dragginzgame/ic-testkit/issues/32) records the new
public Host 0.7 dependency boundary and necessary next minor release, separately
from its previously completed Host 0.5/0.6 adoption. No further Host abstraction
was justified. Consumers are concurrently editing; inventory is point-in-time
source evidence, not a delivered or universally qualified dependency graph.

Both changelog views describe pending 0.7.1. The 0.7 detailed headings now carry
only versions, leaving draft/date state with the finalized root ledger rather
than another label the release runner does not finalize. Existing release prose
is preserved. Rust source, manifests, lock selections and package versions are
unchanged; no function, method or type was removed.

Exact upstream 0.1.26 CI passes Linux, Apple Silicon and lint/security; Intel
remained running at observation. This local Host snapshot adoption still needs
its own matching native CI, distinct from released 0.7.0. Siblings remain
read-only. No broad local gate, tool download, commit/push, release or publication
occurred. Owning GitHub issues remain the sole follow-up tracker.

## Previous work: 2026-10-08, released 0.6.0 and pending 0.7.0 communication (superseded)

Host 0.6.0 is released at `6f066e727c977e0b7ec8d3d77821df8508b95c64`;
manifests and lock reflect that release. The maintainer reports it live, and
[independent consumer feedback](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6059032413)
confirms non-yanked registry packages and the published Toko encoder rehearsal.
[Exact-release CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37769906817)
passes Linux x86-64, Rust 1.88 and native macOS 15 Intel/Apple Silicon.
The 0.6 detail heading now agrees with the finalized root ledger. Historical
candidate statements below are superseded, not current publication status.

The local pending **0.7.0** batch continues
[#5](https://github.com/dragginzgame/ic-host-tooling/issues/5) through the existing
capture engine. `communicate_child` accepts an existing `OwnedChild`, borrowed
input, available output pipes, budgets and a prompt cancellation predicate.
Inherited/file/null output remains caller-configured. `SuccessfulExit::Cleanup`
cleans on leader exit; `Retain` reserves successful ownership through IO so the
consumer can admit output, recheck cancellation/deadlines, then explicitly wait,
terminate or hand off. Reaped/transferred owners are rejected. No IO worker
threads, new dependency, unsafe boundary or second production loop is added.

The public enum additions `ExecutionFailure::Cancelled` and
`ExecutionOperation::{StdinPipe, WriteInput}`, and removal of obsolete
`ExecutionOperation::{StdoutPipe, StderrPipe}`, require a pre-1.0 minor release.
Update exhaustive matches and remove references to the two deleted variants.
No function, method or type was removed. Existing capture/admission entrypoints
retain their IO and cleanup selections. Both changelog views select pending
0.7.0; package versions and dependency selections remain unchanged at 0.6.0.

Focused locked/offline Linux checks pass: 61 process library tests, strict
all-target Clippy, Rust 1.88 all-target compilation, warning-denied Rustdoc and
all-feature/all-target tools-consumer compilation. Coverage includes full-duplex
pipe pressure, EOF and early stdin closure, inherited IO, cancellation with
blocked input and at final success, retained leader cleanup on timeout/overflow,
post-IO rejection/handoff and reuse refusal. Existing direct/group capture and
cleanup-error fixtures remain passing. Logs are `/tmp/ic-host-communicate-*.log`;
initial lint failures and corrected final3 results are preserved.

`/tmp/ic-host-communicate-query/` contains a standalone adaptation of the previous
withdrawn bridge, sources/lock/digests, source delta and logs. Nine selected
existing process fixtures pass, including full-duplex IO, inherited output,
escaped pipes and successful background heartbeat. Three additional probes
prove cancellation, held-pipe timeout and invalid UTF-8 retain their original
failure and stop the owned background group before handoff. The consumer
projection retains Query's own output-limit wording; its initial mismatch and
correction are preserved. Receipt publication is substituted by disposable JSON
in this process-only rehearsal. Its cached external lock selections are retained;
only temporary path-package identities were refreshed to this 0.6.0 checkout.

The rehearsal removes `Capture`, `Capture::start`, `Capture::finish`, `drain`
and the old bridge's IO-worker/supervision loop, replacing them with shared
communication and caller-owned output/error admission. These are deletions in
the private withdrawn candidate, not in a sibling checkout. Current Query is
released 0.49.1 with concurrent dirty source, including receipt changes; copies
of the reviewed source are retained with the rehearsal. Its actual integration
must preserve current receipts, signal interpretation and lifecycle policy.
The existing Python TERM grace and bounded reap are not supplied by Host's
synchronous KILL/reap owner. This is not an apply-ready patch against current
Query or qualification of its full graph, real receipt entrypoint or live network.

Candidate native macOS Intel/ARM qualification remains pending; 0.6 CI does not
qualify these new bytes. Keep #5 open through actual consumer adoption and native
acceptance. Siblings remain untouched. No full local gate, tool download,
commit/push, release, publication or package-version change ran.

## Previous work: 2026-10-08, pending 0.6.0 owned-group capture (superseded)

The only open Host issue, [#5](https://github.com/dragginzgame/ic-host-tooling/issues/5),
now has a concrete Toko Miner bounded-wrapper consumer. The local **0.6.0** batch
adds `tool::capture_group_command` by selecting owned-group spawn in the existing
capture loop. It cleans remaining group members on leader exit or failure,
retaining the original failure, output, status and separate cleanup errors.
Existing direct capture and admitted-tool execution retain their semantics.
The public `ExecutionError::group_error` field requires a pre-1.0 minor release:
update struct literals/exhaustive destructuring, using None in direct-child
fixtures. Manifests, lock and dependency selections remain at released 0.5.2;
no release command or version mutation ran. Both changelog views describe 0.6.0.

Focused locked/offline Linux checks pass: 53 process library tests, strict
all-target Clippy, Rust 1.88 all-target check, warning-denied Rustdoc and
all-feature/all-target tools-consumer compilation. New coverage exercises group
cleanup after success, failure, deadline and both output limits; fair streams,
no-spawn validation and unchanged direct-child behavior remain covered. A
substituted IO fault at finalization cleans a real group; a deliberate external
reap retains original timeout plus group/reap errors. No native pipe-read fault
is claimed. Logs are `/tmp/ic-host-group-*.log`, including the initial lint
failures and successful corrections. No function, method or type was removed.

The private `/tmp/ic-host-group-rehearsal/` runs the actual unchanged Toko Node
wrapper with a synthetic Node fixture: direct timeout permits a late descendant
write; group timeout and overflow do not. The actual encoder module, switched
only to the new capture call, passes all four original tests including its real
30-second deadline. Exact sources/lock/hashes, logs and an applicable source
patch remain there. This is not a live encoder workload or full consumer graph
qualification. Toko retains its environment, admission, budgets and Debug error
projection; its staging supervisor is unchanged. Source/acceptance are delivered
on [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6058697111)
and Toko Miner #33. Sibling working trees remain untouched.

Candidate native macOS Intel/ARM qualification remains pending; released 0.5.2
CI below does not qualify new bytes. Keep #5 open through delivery and actual
consumer acceptance, with Query piped input/inherited output/cancellation still
separate. Signalling is not a descendant-exit barrier or process-tree confinement.
No new unsafe boundary, dependency, background handoff, retry or paid-effect
recovery is introduced. Prior dirty handoff work was preserved. No downloads,
broad local gate, Git commit/push, release or publication occurred.

## Previous continuation: 2026-10-08, released 0.5.2 and custody convergence

Remote main matches released Host 0.5.2 at
`c7014995bf0890c1df9cd9b9a6ec14ea70f98c6f`.
[Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37762087718)
passes Linux x86-64, Rust 1.88 and native macOS 15 Intel/Apple Silicon, including
poll_exit/handoff. This supersedes candidate-native statements below; actual
Query bridge adoption remains separate. Registry publication was not independently
checked in this continuation.

Fresh sibling inspection found an existing owner for the next descriptor slice:
Backup 0.7.0 `f836d5e58170c6b16dba2552c10391666f86ce7e` provides owned
command inheritance, single dispatch, retained exact custody and quiescence.
Its [native release CI](https://github.com/dragginzgame/ic-backup/actions/runs/37753291440)
passes Linux/macOS Intel/ARM. Canic's current Observatory already delegates to
Host capture_command. The remaining local raw-descriptor custody belongs to the
consumer adoption assessment, with paid intent/recovery retained in Canic.

A selected offline Linux probe at `/tmp/ic-host-custody-probe/` proves Host's
existing capture preserves caller-configured command-fds 0.3.3 inheritance:
parent CLOEXEC remains set, the leader is reaped, and after parent descriptor
copies are dropped a contender stays excluded until the descendant's own stop.
Exact source/lock digests, result and initial import-path compilation failure
are retained. No parent-death or paid-effect recovery qualification is claimed.
Command owns its configured clone until dropped; Backup's consuming spawn
already prevents that clone outliving spawn. Its returned standard Child and
Host's spawning capture entrypoint are not automatically an integrated pipeline.
Do not bypass single-dispatch admission or invent bounded paid-command policy.

[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6057856389)
and [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6057856823)
record the concrete owner/deletion route and composition limit. No new Host
wrapper, dependency, unsafe boundary or runtime change is justified by this
mechanism alone. Query piped IO/cancellation and complete consumer acceptance
remain open under #5. Only this handoff changed locally. No function, method
or type was removed; no sibling mutation, download, broad local gate, commit,
push, release or publication ran. No new release draft was created for this
source review and documentation-only continuation.

## Previous work: 2026-10-08, pending 0.5.2 successful background handoff (superseded)

The compatible **0.5.2** batch adds `OwnedChild::poll_exit` and `handoff`
for [#5](https://github.com/dragginzgame/ic-host-tooling/issues/5), based on
released Host `81f9809861159def2fd0987fcb7961cda4afd969`. Polling reserves
an exited leader through `WNOWAIT` without signalling/reaping; explicit handoff
only accepts a still-owned successful exit. Existing wait/termination/Drop
cleanup remains the default. Refused handoff retains ownership, unless lost to
an external reaper; completed handoff never exposes reusable group authority.
The consumer admits IO, checks cancellation and owns subsequent readiness,
stop and recovery. No new dependency, stored format, unsafe block or package
version change is introduced. No function, method or type was removed.

Focused locked/offline Linux evidence: all 48 process library tests pass,
including 10 child cases; strict all-target Clippy, Rust 1.88 all-target check
and warning-denied Rustdoc pass. Child coverage includes leader reservation,
live background IO after handoff/Drop, refused running/failed/signalled exits,
original status preservation, repeated calls, cancellation/unwinding after an
observed successful exit, and external-reaping ownership loss. Logs remain at
`/tmp/ic-host-handoff-{child,process,clippy-final,msrv,doc}.log`.

A private standalone rehearsal under `/tmp/ic-host-handoff-query/` adapts the
withdrawn Query candidate preserved at `/tmp/ic-query-host5.yUGdBe/`. Only
background-start requests opt in. The helper observes while draining IO, cleans
failed exits through ordinary wait, admits decoded output/cancellation/deadline,
and finally calls handoff. The original successful-start heartbeat fixture
fails in the control using ordinary wait, then passes with the new disposition.
Nine selected existing consumer process fixtures pass; two additional scoped
checks prove invalid UTF-8 and a held-pipe deadline clean the background group
before transfer while retaining the original error. Receipt publication is
substituted by disposable JSON output in this process-only rehearsal; no real
network or full Query integration is qualified. Candidate/control sources,
exact digests, logs and deltas against the withdrawn bridge are retained there.
Those deltas are evidence, not an apply-ready patch against current Query.
No Python tooling was added to Host or to the real sibling checkout.

Native candidate macOS 15 Intel/ARM qualification remains pending: released
0.5.1 CI is evidence for its own bytes, not these APIs. Full Query adoption must
rebase its bridge and qualify its real receipt/network entrypoint. Canic inherited
operation-lock custody and bounded execution convergence remain separate #5
requirements. The real Query checkout and all other siblings remain read-only.
Existing audit/handoff edits were preserved. No broad local gate, download,
commit, push, release or publication ran; both changelog views carry 0.5.2.

## Previous recheck: 2026-10-08, Testkit 0.22 and Canic 0.110.53 pushed

Testkit 0.22.0 is on remote main at
`2951fd19e58799580e60ec0f6f5864d296271a62`.
[Its CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37752946475)
passes portable-host, checks and PocketIC concurrency on Linux/macOS Intel/ARM;
the MSRV job is skipped. The maintainer reports publication live; an independent
registry API read returned HTTP 403. Blob Storage, IcyDB, Timers and Toko Miner
now have dirty manifest/lock selections of Testkit 0.22.0 and only Host 0.5.1.
This supersedes their older dependency observations below, not their pending
consumer delivery/qualification obligations.

Canic's remote main is released 0.110.53 at
`4c51a87c6a32397196bb3f65d064641194df10a5`. Its pushed graph still selects
Host 0.4.6/Testkit 0.21; active dirty manifest/lock edits select Host 0.5.1,
Testkit 0.22.0 and Backup 0.7.0. The remaining old Host artifacts/fs reverse
edge in that working lock comes from Query 0.48.1. Installer/Wasm owner
replacement is still pending, separately from dependency selection.
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6056656562)
and [Testkit #25](https://github.com/dragginzgame/ic-testkit/issues/25#issuecomment-6056656248)
carry the superseding evidence.

Both Canic native macOS jobs in
[release CI](https://github.com/dragginzgame/canic/actions/runs/37752054026)
fail with `test-pocketic-workers.sh: line 92: rg: command not found`.
The workflow invokes worker fixtures before installing pinned host tools.
[Canic #465](https://github.com/dragginzgame/canic/issues/465#issuecomment-6056655921)
owns the smallest correction: install/export the existing canonical tools before
those fixtures, then qualify both native jobs. This is not evidence of a Host
child failure. Other Canic release jobs were still running at observation.
Sibling checkouts remain untouched; only this handoff changed locally during
recheck. No builds, dependency resolution, runtime changes or deletions occurred.

## Previous qualification: 2026-10-08, released 0.5.1 and renewed sibling audit

Host 0.5.1 is committed at `81f9809861159def2fd0987fcb7961cda4afd969`.
[Exact-release CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37750135927)
passes Linux x86-64, Rust 1.88 and native macOS 15 Intel/Apple Silicon.
This supersedes the pending delivery/native statements below;
[#22](https://github.com/dragginzgame/ic-host-tooling/issues/22) is closed.
The maintainer reports publication live; this audit independently checked Git/CI,
not registry publication.

The [new frozen convergence audit](../reports/audits/2026/10/08/host-reuse/02/report.md)
records exact sibling working-source and dependency evidence. The main ready
simplification remains coordinated Testkit 0.22/Canic adoption, then removal of
old Host 0.4 edges and Toko's duplicate Wasm parser. Canic's prepared patch still
passes applicability; pending sibling changes are not delivered adoption.
Query's attempted process bridge was withdrawn after its successful-background
handoff fixture failed. [#5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6055899595)
owns that newly demonstrated contract, preserving existing Testkit cleanup.

Canonical 0.1.23 release-runner adoption findings were delivered to existing
Canic #453/IcyDB #299 and new Query #20/Blob Storage #28; Testkit #25 carries the
current downstream dependency evidence. Toko Miner's snapshot moved to 0.1.23
during inspection. Shared Tooling's latest committed revision remains the one
already adopted here. No new Host runtime defect or further generic byte/file
abstraction was established. The retained Toko contract failures remain bound
to the earlier fixtures and unchanged source, not a new execution.

Only this handoff and new audit evidence changed locally. Siblings stayed
read-only; no runtime code, dependency, function, method or type changed or was
removed. No compilation, broad local gate, release, publication, commit or push
ran. GitHub issues remain the sole follow-up tracker.

## Previous qualification: 2026-10-08, pending 0.5.1 release-runner repair (superseded)

The compatible **0.5.1** batch adopts committed Shared Tooling 0.1.23
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0` for
[#22](https://github.com/dragginzgame/ic-host-tooling/issues/22), the consumer
adoption of [upstream #58](https://github.com/dragginzgame/shared-tooling/issues/58).
The canonical refresh exported the existing 68-file set from a clean private
clone, preserving the prior dirty handoff and audit evidence. Only the shared
direct runner, its owning regression fixture and release guidance changed within
that set; local AGENTS.md and the exact-digest snapshot identify the new revision.

Direct push now rechecks committed payload/index/worktree and the exact annotated
tag after the final consumer check. Completed resume verifies local/remote tag
identity and observed branch ancestry before returning success. Conflicts and
unavailable observations stop without repeating release effects; known descendant
tips remain valid. Direct delivery, adapter receipts and stored plan format remain
unchanged. The optional PR helper is not added or enabled by this refresh.

The new owning fixture reproduces the old runner's final-hook tag-retargeting
failure before adoption: it accepted the mutation, pushed and marked completion.
That command-substitute failure is retained in
`/tmp/ic-host-051.cdcrRj/before.log` and `/tmp/release-runner-test.Fm5yQL`.
After adoption, focused Linux direct-runner, Host adapter/receipt and Make wiring
fixtures pass. The direct-runner fixture also passes under genuine Bash 3.2.57
on Linux using an already-installed binary, with child shell selection included.
That shell-portability check is not native macOS execution. Snapshot integrity,
documentation links and whitespace checks pass.
Selected ShellCheck passes with its optional source lookup bound to the same
reviewed upstream clone; initial missing-unselected-helper diagnostics are retained.
No real release effect is exercised by these fixtures. Logs and exact source clone
remain under `/tmp/ic-host-051.cdcrRj/`.

Upstream [exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37746567888)
passes Linux, macOS 15 Intel/ARM and lint/security. This is upstream native
qualification; the uncommitted Host adoption still needs its own matching native
CI before delivery qualification is complete. Released Host 0.5.0 CI remains
evidence for its own bytes only. #22 stays open for delivery/qualification;
#5 retains the separate consumer capture/cancellation and inherited-lock work.

Both changelog views carry the pending 0.5.1 correction. Crate APIs, Cargo
manifests, dependency selections and package versions are unchanged. No function,
method or type was removed. No Rust compilation, broad local gate, tool download,
sibling mutation, commit, push, release or publication occurred. The preceding
audit files remain present and unchanged.

## Previous qualification: 2026-10-08, released 0.5.0 and consumer audit

Remote main and annotated v0.5.0 resolve to
`db637fac8b7a9ef62301e1d9009ffeb5ffcd0be7`, matching this checkout.
[Exact release CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37744999108)
passes Linux x86-64, Rust 1.88, macOS 15 Intel and macOS 15 Apple Silicon.
This supersedes the pending delivery and native qualification statements below.
The separate draft PR #21 remains open; its status does not describe release main.
The maintainer reports the release live; this audit verified Git/CI identity,
not registry publication independently.

The [sibling redundancy audit](../reports/audits/2026/10/08/host-reuse/01/report.md)
finds that the next useful simplification is consumer adoption of delivered
owners. The existing Canic and Testkit patches still pass applicability checks
against their current selected source. Their earlier isolated Linux rehearsals
remain historical evidence; no new consumer build or native integration is
claimed. Toko's two shell callers still use a hand-written Wasm parser. Three
local fixtures demonstrate import-count drift, malformed-input acceptance and
a JSON/enforcement inconsistency; a locked/offline Host 0.5 source probe returns
the expected structural facts and rejects the malformed fixture.

Evidence and concrete deletion routes are delivered to
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6055356118),
[Canic #481](https://github.com/dragginzgame/canic/issues/481#issuecomment-6055364940),
[Testkit #25](https://github.com/dragginzgame/ic-testkit/issues/25#issuecomment-6055355777)
and [Toko #1791](https://github.com/dragginzgame/toko/issues/1791#issuecomment-6055355350).
Host #10, #14 and #20 are closed with release evidence, including concurrent
consumer verification. #5 retains its distinct capture/cancellation and inherited
lock obligations. GitHub remains the sole follow-up tracker.

Shared Tooling's committed source is still the adopted 0.1.22 revision
`2687f26317952c43c685f7f799ed09288dc10a67`; pending 0.1.23 edits were not adopted.
No new Host production abstraction is justified by this audit. Consumer domain
hash framing, measured scratch-buffer choices, confinement, private crash
barriers, artifact sets and paid-operation recovery retain their existing owners.
Only this handoff and audit evidence changed locally. No function, method, type,
dependency or package version was removed or changed. Sibling working trees
remain untouched; no release, publication, commit, push or broad local gate ran.

## Previous qualification: 2026-10-07, pending 0.5.0 (superseded)

Released base remains 0.4.6 at
`0fb05f9e18f032425188d68e1d69317a0f0127d5`. The current local batch carries
forward the child work below and implements the remaining artifact/API and
single-file executable publication requirements. The pending release is now
**0.5.0**, superseding the earlier 0.4.7 draft: numeric gzip compression arguments,
new public Wasm fact fields, stricter import/global framing and additional typed
inspection failures require consumer changes. Both changelog views are aligned;
manifest/package versions remain 0.4.6. No stored format changed.

`durable::write_validated_with` closes the staged writer before caller admission,
retaining verified read-only inode custody in the existing publication engine.
It preserves original producer/admission failures, separate cleanup errors and
before/after-publication filesystem evidence. Linux tests execute a staged
`/bin/sh` before replacing the old destination and cover rejection, foreign inode
replacement, create-only races and sync/publication failure ordering. Writable
clones, child completion, content/namespace custody, executable policy and
multi-file bundles remain consumer obligations.

Wasm inspection now reports imported functions, defined/imported globals and
exact code-body bytes. Source review corrected the premise in
[Host #10](https://github.com/dragginzgame/ic-host-tooling/issues/10): the replica
checks bytes after the encoded function-count prefix, whereas existing
`code_section_bytes` includes it. Both facts are retained with distinct meanings;
a padded-LEB fixture verifies the actual prefix width is excluded. The opt-in
`ic-limits` tools adapter compares three selected resources using caller limits
or reference revision `9499f64bda8bcf087188dd8a1bb594115640ba9e`. It does not
establish complete installability or deployed-network policy. With defaults
disabled it introduces no filesystem/process dependencies.

After explicit locked offline cache preparation, focused Linux checks pass:
63 filesystem tests, 65 all-feature artifact tests, 12 response/limits-profile
tests, strict selected Clippy, Rust 1.88 all-target checks, warning-denied Rustdoc
and actual feature-graph exclusions. The child qualification below remains
applicable to unchanged child source. Logs, including corrected initial lint
failures, are preserved under `/tmp/ic-host-next-*`, `/tmp/ic-host-050-*` and
`/tmp/ic-host-047-*`. No full local CI/release gate ran. Native macOS compilation
and execution of the candidate, including the approved private FFI call and
closed-writer executable fixture, remain unqualified on both architectures.
Released 0.4.6 CI success is not candidate evidence.

GitHub delivery now succeeds after earlier server failures. Source/evidence and
remaining delivery requirements are recorded on
[#5](https://github.com/dragginzgame/ic-host-tooling/issues/5),
[#10](https://github.com/dragginzgame/ic-host-tooling/issues/10),
[#14](https://github.com/dragginzgame/ic-host-tooling/issues/14) and
[#20](https://github.com/dragginzgame/ic-host-tooling/issues/20).
[#13](https://github.com/dragginzgame/ic-host-tooling/issues/13) is closed as not
planned: `candid_parser::utils` already owns service compatibility/reporting,
and Canic's cited domain-specific method equality is not the same operation.
No redundant compatibility wrapper or dependency was added.

The prepared child replacement and exact private rehearsal evidence are delivered
in [Testkit #25](https://github.com/dragginzgame/ic-testkit/issues/25).
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6043858342)
records the closed-writer contract, numeric gzip adoption and corrected Wasm
comparison, bound to reviewed dirty module digests. Sibling checkouts remain
untouched. Testkit publicly reexports Host crates, so selecting the new Host minor
also requires its consumer minor. Canic must preserve typed publication/cleanup
outcomes and retain its independently used bundle staging helper. Query's Python
capture/cancellation integration and Canic's inherited operation-lock descriptors
are still unimplemented requirements under #5; owned pipes alone do not satisfy
those contracts. No descendant-exit or paid-effect recovery guarantee was added.

The only removed Host function is private `tool::process::reap`, replaced by
`child::OwnedChild` cleanup/reaping. No type or public function was removed;
`encode_gzip` changed its argument contract. No dependency version or registry
selection changed, and no download, sibling mutation, commit, push, release or
publication was performed.

The subsequent adoption rehearsal refreshes all 68 selected Shared Tooling files
from committed 0.1.22 `2687f26317952c43c685f7f799ed09288dc10a67`, using a clean
isolated copy; the sibling's uncommitted 0.1.23 edits are excluded. AGENTS.md and
the snapshot identify the new revision. Its upstream native Linux/macOS jobs
[pass at that exact revision](https://github.com/dragginzgame/shared-tooling/actions/runs/37659875012).
Focused local snapshot, release-adapter, direct recovery and command-wiring
fixtures pass. `RELEASE_DELIVERY=direct` remains the consumer's qualified policy;
other selections are refused before command execution. PR release delivery,
including its pending older-gh compatibility fix, has not been adopted. Ordinary
contribution PRs remain available through the existing contribution rules.

Canic rehearsal is bound to clean committed source
`95038e1a381ce88eb19415f5bb6d8439457b75f1` in an isolated copy. All four Host
packages use explicit local source patches; the selected registry versions are
not upgraded or presented as published 0.5. Nine installer, 24 artifact and two
staged executable-admission tests pass, with strict canic-host all-target Clippy.
Original admission errno, independent foreign-stage cleanup refusal and typed
after-publication projection are covered. Existing bundle fixtures still pass.
No real tool installation or network download ran. The complete proposed patch,
scope, source and consumer minor-boundary requirements are delivered on
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6044765060).
The real Canic checkout is untouched.

The proposed Canic patch removes private `artifact_io::wasm::validate_defined_functions`
and `validate_wasm_code_section_size`, replacing them with one projection over
the shared report. Its three former boundary tests are consolidated into the
new reference/import boundary case; final-artifact exact/+1 coverage remains.
`publish_executable` is refactored, not removed. Bundle helpers remain required.
No additional Host function or type was removed by the snapshot/rehearsal work.
Logs, failed lint attempts, exact source digests and patches are retained under
`/tmp/ic-host-050-adoption.6V2DSt/`.

This machine is Linux x86-64. Native candidate macOS qualification requires the
existing PR CI matrix; its latest completed Host run still covers released
0.4.6. The complete candidate patch and draft PR description are prepared in
the same evidence directory. A scoped commit/PR request is required by the
contribution rules before publishing the candidate branch. No candidate native
macOS run, commit, branch push or PR has occurred yet.

## Previous qualification: 2026-10-07, initial 0.4.7 draft (superseded)

The maintainer reports 0.4.6 live. The compatible **0.4.7** candidate implements
the explicit child/group cleanup portion of
[#5](https://github.com/dragginzgame/ic-host-tooling/issues/5), based on
`0fb05f9e18f032425188d68e1d69317a0f0127d5`. `child::OwnedChild` preserves the
caller's command IO/context while creating a new group. Polling, natural waiting,
termination and Drop share leader reservation, signalling and reaping mechanics.
The existing capture engine now uses that owner internally with its existing
direct-child contract. Its private `tool::process::reap` function was removed;
`OwnedChild::terminate` and its private reap method replace it. No public function,
method or type was removed. No application lifecycle or recovery policy moved.

The maintainer explicitly approved the small macOS FFI boundary after reviewing
`/tmp/ic-host-047-macos-boundary.rs`. One private fixed-buffer membership query
handles the sole-zombie group EPERM case; unsafe code is denied elsewhere.
The already-selected libc 0.2.190 is now a macOS-only direct dependency and
rustix enables its process feature. Cargo.lock adds only the libc dependency
edge; package versions and registry version selections are unchanged. No tool
or dependency download was needed. Native macOS compilation/execution of these
candidate bytes remains outstanding, independently of released 0.4.6 success.

After explicit offline cache preparation, Linux checks pass: 44 process tests,
strict process Clippy, Rust 1.88 checks and Rustdoc. Native child tests exercise
caller IO/context, new group selection, normal exit with a remaining descendant,
explicit termination, Drop/unwind, repeated status inspection and lost reaping
ownership. Interrupted-call retry is injected at its helper boundary, not claimed
as native signal-delivery evidence. Initial lint findings and corrected logs are
retained under `/tmp/ic-host-047-*`. Sorting, formatting, dependency declarations,
snapshot integrity and documentation checks pass. No full local gate ran.

The proposed Testkit patch `/tmp/ic-host-047-testkit.patch` applies to the unchanged
startup module on `2db7b4f6b616b484408695656e26207628d74c5f`; its source digest
is recorded in `ci/extraction-sources.json`. A private source rehearsal at
`/tmp/ic-host-047-testkit.0Jzbxb` replaces the guard/poll/group-kill/reap helpers,
removing 134 net lines. Explicit local-source Cargo patches pass 16 startup tests,
7 server-runner integration tests and strict Testkit library Clippy on Linux.
One real-server startup test and two runner cases (real server and its worker)
remain ignored. No real PocketIC server ran. This is candidate source evidence,
not registry adoption or native consumer qualification. Testkit's real checkout
and unrelated dirty work remain untouched. No commit, push or release occurred.

GitHub reads succeeded, but attempts to create the consumer handoff issue and
comment on existing #5 failed with server errors. Reconciliation found no created
consumer issue; the prepared payload and patch remain in `/tmp/ic-host-047-*`
as delivery evidence, not a replacement issue tracker. #5 remains the owning
follow-up for qualification and its separate Query/Canic execution requirements.

## Previous qualification: 2026-10-07, after 0.4.6

Released 0.4.6 is `0fb05f9e18f032425188d68e1d69317a0f0127d5`.
[Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37648086908)
passes MSRV, Linux x86-64, macOS 15 Intel and macOS 15 Apple Silicon. This
supersedes the outstanding native qualification below: the Darwin mode repair
and native non-UTF-8 publication fixture now pass on both supported architectures.
The maintainer reports the release pushed; CI evidence alone does not establish
registry publication or consumer adoption.

The issue review recorded this release evidence on the completed Host extraction
and snapshot-adoption issues before closing them. Consumer integration remains
with its existing owning-repository issues, including Query's descriptor writer
and Backup's crash-barrier obligations. Source review of the remaining proposals
found no additional confirmed patch defect: process-group cleanup needs a bounded
ownership design, Candid compatibility must reuse its existing upstream engine,
and changes to the public compression type or Wasm fact fields require a minor
release. GitHub remains the follow-up tracker; this handoff is qualification
evidence rather than a second issue queue.

Shared Tooling remains at committed 0.1.20
`3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`, already adopted. This continuation
only updates the handoff and removes the README's stale release-number copy.
No production code, function, method, type, dependency or package version changed.
Documentation links and whitespace checks pass. No local compilation, full gate,
sibling mutation, commit, push or publication was performed.

## Previous qualification: 2026-10-07, after 0.4.5

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
