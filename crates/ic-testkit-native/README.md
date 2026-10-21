# IC Testkit Native

Std-only native allocation measurement by default. This package has no PocketIC,
Host or canister dependencies and exposes no APIs on wasm32.

Install `MeasuredAllocator::new(System)` as the test binary's global allocator,
or wrap another allocator. `measure(iterations, operation)` returns allocation
and reallocation counts, net bytes, peak positive net bytes and elapsed time for
this thread only. Keep workload construction and output formatting outside the
measurement. Other threads are deliberately excluded; this is not a process RSS
or IC instruction measurement.

Allocations freed during the scope may predate it: net bytes are signed and the
peak is relative to the scope's starting point. Nested measurement refuses before
running its operation. Unwinding disables measurement; the original panic remains.

Enable `stable-memory` for `memory::MeteredMemory` and `MemoryIoCounts`, using
Platform 0.6.2's upstream `Memory` and `VectorMemory` identities. Clones share
backing bytes and counters; `reset()` clears observations without clearing bytes.
Read/write attempts and completed growth calls retain the backing's behavior,
including specialized unsafe reads and refused growth. Check `overflowed` and
refuse exact reports when set; counters saturate rather than wrap. These are
native IO observations, not IC instructions or cycle costs. Workloads, corruption
injection and production memory governance remain caller-owned.
