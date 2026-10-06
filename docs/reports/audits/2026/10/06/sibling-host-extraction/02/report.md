# Second-pass shared reuse evidence

Review date: 2026-10-06. Trigger: maintainer asks whether further work belongs in
pending 0.3.1. Method: `audits/flow-convergence-and-duplication.md`, Shared Tooling
snapshot `21f3ec3dd97f2968c9f0b08924451bb2f71770d1`. Local overlay: AGENTS.md and
docs/extraction.md. Host workspace HEAD: `efd402e0063ccbbf8a143cc970be52ab41b1766d`,
with the existing pending extraction/license work and this documentation correction.

Verdict: **PASS WITH FINDINGS** within the selected stream/hash/publication traces.
Three additional consumer reuse sites need no new shared API. This is not an
exhaustive audit of sibling products or a release qualification. The first run's
test results remain historical evidence; this run does not relabel them.

## Source identity and trace

The selected sibling files matched their committed bytes. No sibling mutation or
consumer compilation was performed; unrelated dirty source was preserved.

| Consumer | HEAD | Selected file SHA-256 |
| --- | --- | --- |
| Canic | d815abfc661d791ecf72afc5b1e4b6a990f37a91 | network/mod.rs: b35b7e777dc9f44337bd95045f389368bb09852e2b0722b297d878462bb1ca52 |
| Query | 0905078d1ef410c63a89fb2924cd18a0653ffa4e | cache_file/confined/read.rs: f8c83f6110177d1f3f783f7b32625ae75dff2409c750c6f1f8782e20aed35c7d |
| IcyDB | 049e561a843a8d3f4526460fd15876a4a238df10 | diagnostic.rs: 9fccb32b5e741af6ea49c50bdc94f1821088ea14a8461a9975ec58672d4a0de5 |

Paths are under each consumer's crates/<owning-crate>/src. Query was clean at
inspection; Canic and IcyDB had existing dirty work outside these selected files.

| Entry to result | Canonical mechanics | Retained consumer contract |
| --- | --- | --- |
| Canic network fingerprint hash/parse/format -> network identity and serialization | Existing artifact::Sha256Digest compute/FromStr/Display | Domain framing, lowercase admission, network errors; uppercase-admitting policy gate remains distinct |
| Query confined open -> metadata admission -> read_bounded_stream -> aggregate charge | Existing artifact::read_reader | Capability root, metadata-first rejection, actual overflow count, host conversions, aggregate budget and typed projection |
| IcyDB file/stdin -> read_error_json -> object/schema validation | Existing artifact::read_reader | 64 KiB policy, file admission, public Error schema, recursion bound and diagnostics |

These LOW duplicate-mechanic findings are tracked by source-backed comments on
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6021687368),
[Query #11](https://github.com/dragginzgame/ic-query/issues/11#issuecomment-6021688665)
and [IcyDB #307](https://github.com/dragginzgame/icydb/issues/307#issuecomment-6021689971).
Comments specify consumer acceptance checks and allocation/error projection.
Existing 0.3.0 APIs suffice; no fourth shared primitive is justified by these traces.
This does not claim consumer adoption or production-symbol removal.

## Documentation correction and retained separation

The new write_with producer error documentation overstated staging cleanup.
The implementation attempts unlink after a returned error; unlink can fail, and
callback panic/process interruption can retain staging. API and extraction docs
now state those limits, preserving the existing publication engine and semantics.
The extraction ownership summary now includes gzip encoding as well as decoding.
No functions, methods or types were removed, and no dependencies or versions changed.

Backup's private parents/files and crash barriers, Query's capability confinement,
Testkit's framed tree keys and process groups, Canic's inherited ICP lock descriptors,
public response error detail and installation policy remain deliberate separation.
Changing exhaustive public response errors would need compatibility review and may
require a pre-1.0 minor release; it is not silently folded into this patch batch.

## Verification

No Cargo/rustc process was active before documentation mutation or Rustdoc.
Focused Linux Rustdoc passed for ic-host-fs, all features, locked/offline, no-deps,
with RUSTDOCFLAGS=-D warnings. Formatting, documentation links and diff whitespace
checks passed. GitHub comment creation returned the linked receipts.
No new runtime behavior was introduced, so earlier focused runtime evidence was
not rerun. No sibling builds, full CI, native macOS execution, release, package
version change, commit, tag, push or publication occurred. Native macOS 15 Intel
and Apple Silicon qualification of the pending additions remains outstanding.
