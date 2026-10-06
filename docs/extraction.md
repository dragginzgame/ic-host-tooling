# Crate ownership and extraction contract

The initial library source is committed IC Host Tools 0.2.0 at
be7d73908225e73ab5babf8fbb9fe959f40d439b. Pending sibling tooling edits
were excluded. Canic's clean durable module and its tests are bound to their
observed committed bytes in [source provenance](../ci/extraction-sources.json).

Dependencies point down from ic-host-tools to ic-host-process and ic-host-fs,
from ic-host-process to ic-host-fs, and from each to ic-host-artifacts.
No generic crate depends on ic-host-tools, Canic, IcyDB or an IC runtime.
Test/example artifact features do not become production filesystem dependencies.

- ic-host-artifacts owns stream identities, bounded read/write/copy mechanics,
  optional gzip decoding, verified tar member bytes and optional Wasm structural facts.
- ic-host-fs owns pathname/descriptor reads and the durable module's replace,
  create-new, private-file, parent synchronization and descriptor-lock operations.
- ic-host-process owns executable resolution/admission, explicit execution context,
  output limits, deadlines, direct-child cleanup evidence and separate Git queries.
- ic-host-tools owns Candid text/extractor interpretation and ICP response formats.

Source tests move with their owner; public artifact projection tests belong to
ic-host-artifacts. Runnable examples retain explicit dependencies on each owner.
No generic compatibility reexports remain in ic-host-tools.

The first import preserves implemented guarantees. It does not establish root
confinement for following pathname APIs, immutable source bytes, process sandboxing,
descendant cleanup, hardware-independent crash durability or safe paid-effect retries.
Query's capability confinement and refresh leases remain consumer-owned.
Backup's tree checksums, multi-file publication and journals remain consumer-owned.
Further adoption must retain each caller's independently established guarantees.

The maintainer authorized retiring the original local checkout after verification
of a full backup. Existing consumers retain their prior dependency selections;
adoption of these split crates remains separate. The manifests permit crates.io
publication through the [explicit publication procedure](publishing.md).
The breaking removal of generic APIs from ic-host-tools requires a coordinated
pre-1.0 minor release. This contract assigns ownership; GitHub issues own follow-up tracking.
