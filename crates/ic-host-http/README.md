# IC Host HTTP

An optional native response-body reader for callers already using reqwest 0.13.
Add this package only where HTTP is needed; existing Host/Build/Testkit packages
have no dependency on it. TLS, compression, proxies, redirects, retries and
request/total timeouts are selected on the caller's client, not by this crate.

`read_body(response, ReadOptions { ... })` consumes an already-created response.
It never rejects an HTTP status or decodes JSON. Callers can inspect status and
headers before passing ownership. Success returns status and complete bytes;
failure returns status, bytes received so far, a typed cause and optional bounded
partial bytes. Debug output and retained bodies can contain sensitive data:
redaction and durable evidence remain caller-owned. Display omits body bytes.

Two consumer patterns preserve different contracts:

- Query: reject status before calling; select `reject_declared_oversize: true`
  and `retain_partial_on_error: false`, then decode JSON after success.
- Blob probes: select both `reject_declared_oversize: false` and
  `retain_partial_on_error: true`; retain status and bounded bytes on failure,
  and apply unsuccessful-status policy only after body reading.

The caller supplies `max_bytes` (including zero). An absent or smaller declared
length never bypasses streamed admission. Limits count decoded bytes yielded by
reqwest, including decompression when the caller enables it. They do not bound
upstream chunk buffers, decompressor memory or compressed wire bytes. On an
oversized chunk the received count includes the whole yielded chunk, but retained
bytes never exceed the ceiling. Counts saturate at u64::MAX. Allocation uses
Host Artifacts' fallible bounded reservation; allocator rounding is outside the
reader's control. No untrusted Content-Length is used to preallocate a buffer.

Native library and loopback checks:

```sh
cargo test -p ic-host-http --locked --offline
cargo clippy -p ic-host-http --all-targets --locked --offline -- -D warnings
```

The Wasm package exposes no HTTP adapter or dependencies. Native macOS
qualification and publication precede consumer adoption. Query and Blob's
existing implementations remain until their owners migrate and qualify them;
this package does not change either sibling repository.
