# Native file commands

`ic-host-file` is a binary in the existing `ic-host-fs` package. It translates
explicit native arguments into Artifacts/FS operations; it has no IC, process,
network, release or browser-contract dependency. The supported execution hosts
are Linux x86-64 and macOS 15 Intel/Apple Silicon. Non-Unix platforms refuse file
operations before effects; help/version remain available for identification.
Other Unix hosts are unqualified and retain the libraries’ platform refusals.

## Command contract

```text
ic-host-file --version
ic-host-file --help
ic-host-file hash-file PATH MAX_BYTES
ic-host-file hash-files0 MAX_BYTES MAX_LIST_BYTES MAX_FILES
ic-host-file create-private SOURCE DESTINATION MAX_BYTES
ic-host-file replace-private SOURCE DESTINATION MAX_BYTES
```

Limits are explicit unsigned integers fitting the host's `usize`; zero is valid.
Unknown operations, missing/extra arguments and invalid limits refuse. Paths are
OS-native arguments, including non-UTF-8 bytes where the filesystem accepts them.
The CLI does not expand globs, reinterpret paths as options or change directory.

`hash-file` delegates to `read::hash_file_no_follow`, returning exactly 64 lowercase
SHA-256 hexadecimal bytes followed by a newline. It hashes an admitted regular
file with constant working memory. Metadata and streamed bytes are independently
bounded. An empty file accepts `MAX_BYTES=0`.

`hash-files0` reads an ordered NUL-delimited path list on stdin, bounded by
`MAX_LIST_BYTES`. Empty input succeeds with empty output; a nonempty list must
end in NUL, contain no empty path and have at most `MAX_FILES` entries. Order,
duplicates and raw path bytes are preserved. Every file has its own `MAX_BYTES`
allowance. Output is `sha256<TAB>raw-path<NUL>` for each entry. List and file
admission finish before any batch output, so a later missing, oversized, linked
or nonregular file emits no successful prefix. Output storage is checked and
fallibly allocated; its size is list bytes plus 65 bytes per path. Read deadlines
and aggregate filesystem work remain caller-owned.

`create-private` reads an admitted source before any destination effects, then
uses `durable::write_with` with `CreateNew` and permissions `0600` subject to
umask. An existing destination is preserved. `replace-private` uses the same
source admission and permissions with `Replace`. Both delegate file sync,
publication, parent sync and failure/cleanup diagnostics to the canonical writer.
Success emits no stdout. Source bytes are buffered up to the explicit ceiling.
Missing parents use the writer's ordinary creation semantics; a private file
does not establish private parent directories.

Final-component source symlinks, directories and other nonregular sources are
refused. Ancestor aliases are followed: this CLI is not root confinement. Callers
own parent admission, namespace and concurrent-writer custody, path/permission
policy and the meaning of the selected digest.

Exit status is 0 for success and 1 for refusal, with diagnostics on stderr.
Diagnostics are human text, not a machine-readable recovery protocol. A failed
stdout write may leave a prefix: admit exit status before accepting output. A
nonzero write command can follow successful publication and failed completion;
reconcile destination/effect evidence before retrying. The CLI never retries,
reclaims retained staging, chooses a recovery action or runs an upload.

## Explicit preparation and checks

The containing version must first be published; source integration does not
make the new binary available from an older registry version. Consumers select
an exact published `ic-host-fs` version, native target, Rust toolchain and a fresh
installation prefix through their own qualified setup workflow. For example,
after explicitly selecting `HOST_FS_VERSION` and `HOST_FILE_ROOT`:

```sh
cargo install --locked --version "$HOST_FS_VERSION" --root "$HOST_FILE_ROOT" \
  ic-host-fs --bin ic-host-file
```

This is an explicit preparation command, not a check or permission to replace an
existing tool installation. Keep its installation/build logs and failed evidence.
Consumers admit the selected binary and expected provenance/digest before running
its offline `--version` check, which prints `ic-host-file PACKAGE_VERSION` plus a
newline. Checks and command invocation never install or update anything. Host
adds no product-tool pin or automatic entry to Shared's common tool bundle.

Auth's current batch policy can be selected explicitly with
`hash-files0 MAX_BYTES 4194304 4096`. Its browser contracts, Cargo lock admission,
receipt/intent schemas and release recovery remain Auth/Build-owned. Auth must
retain its current adapter until the published binary and actual file/release
interruption fixtures qualify together; do not add a second runtime fallback.

## Focused qualification

```sh
cargo test -p ic-host-fs --features test-support --test file_cli --locked --offline
cargo clippy -p ic-host-fs --all-targets --all-features --locked --offline -- -D warnings
```

The real-executable fixture covers byte limits, no-follow refusal, create-only
preservation, replacement permissions, raw path/batch bytes, all-or-nothing input
refusal and explicit command/version/limit admission. Generic writer crash and
sync regressions remain with FS. Product release interruption and installed-tool
admission remain consumer proofs. Native CI executes this fixture on all three
supported hosts; Linux results alone do not qualify macOS.
