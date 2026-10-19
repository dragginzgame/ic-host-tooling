# Held filesystem capabilities

On Unix, `ic_host_fs::capability` owns the cap-std/cap-fs-ext implementation
adapted from Query's confined cache backend. The backend types stay private;
consumers use Host's directory and file-entry handles. This is filesystem
mechanism, not cache configuration or a lifecycle owner.

## Selection and policy

Enable `ic-host-fs`'s `capability` feature explicitly. It selects both private
cap-* dependencies on Unix; defaults and the independent `cli` feature do not.
Scoped `scratch::TemporaryFile` allocation instead requires `temporary-file`;
neither feature selects the other. See the [0.30 migration](changelog/0.30.md).

Open a root with explicit `DirectoryOptions`. `create_permissions: None` observes
absence; `Some(mode)` permits missing directory creation subject to umask.
`forbidden_permissions` is the caller's mask of disallowed low nine permission
bits. There is no implicit permission policy or repair of existing directories.
Existing ancestors above the selected root are traversed without following
links, but only newly created ancestors and the final root receive that mask.
Reject parent traversal before normalization; symlinked aliases such as macOS
`/tmp` must be resolved by the caller's trusted root selection when appropriate.

`resolve_parent` accepts a nonempty relative path of normal components and
retains its parent. It admits every traversed directory against the caller's
mask and creates nothing unless creation was selected. It does not open the
final leaf. `open_directory` selects one child with the same semantics.
Directory creation handles AlreadyExists by re-opening without following links,
admitting the actual resulting directory and synchronizing its retained parent.
Failures can leave visible created directories; reconciliation stays with callers.

## Descriptor operations

- Open regular files through the retained file entry. This shares Host's existing
  nonblocking no-follow descriptor admission, refusing special files rather than
  waiting for a FIFO writer. Missing leaves are distinct from IO/type failures.
  Check file permissions and content through the returned standard file handle.
  Use the existing bounded reader with the caller's byte budget; aggregate debit
  ordering, partial-read accounting and schema validation stay with the consumer.
- Create new read/write files exclusively with an explicit mode. Existing names
  retain their bytes and native AlreadyExists error. Creation alone performs no
  durability sync; callers choose the required file and parent barriers.
- Use `parent().open_syncable()` with `durable::write_at_validated_with` for
  publication. The existing writer retains create/replace, validation and typed
  publication outcomes. Do not re-open a diagnostic pathname or assume a cap-*
  descriptor is syncable: some use O_PATH. The syncable descriptor opens `.`
  relative to the same capability.
- `remove_file_and_sync` unlinks a leaf and synchronizes that held parent. Admit
  allowed leaf types/permissions first and retain namespace custody. It is not
  identity-conditional unlink: a raced-in symlink can itself be removed, without
  following its target. Missing leaves return false. Failure retains whether
  unlink completed and the original operation/cause; callers own recovery.

Renames do not retarget existing handles. A handle retains its admitted directory,
even if it is moved outside its original pathname tree; this is not a claim of
current pathname containment, immutable content or exclusion of concurrent writers.
Dropping a handle closes it without recursive deletion or cache cleanup.

## Discovery and consumer migration

`entries(maximum)` lazily yields native names and observed entry kinds without
following links. Every observed entry consumes the allowance. One lookahead
distinguishes an exact fit from overflow; overflow is reported once and ends the
iterator. Enumeration order is unspecified. Observations are not descriptor
admission: use a directory/file handle before subsequent IO.

Consumers retain recursion, depth and aggregate limits, candidate selection,
ordering, truncation presentation and public error projection. Query retains
cache paths, collection identities, filenames, required directory/file modes,
schemas and refresh leases. Its adapters should consume these handles and the
existing Host readers/writer, then move discovery onto the same handles.
Remove Query's direct cap-* edges only once all its callers migrate and its
cache/error/scan regressions pass. This producer change leaves Query unchanged;
[Host #80](https://github.com/dragginzgame/ic-host-tooling/issues/80) records the
extraction context. The [provenance manifest](../ci/extraction-sources.json)
identifies the generic source selection; Query's MIT notice is already retained.

The API is Unix-only. The non-Unix compile contract and graph exclusion remain
in place; Linux tests and cross-compilation do not qualify native macOS behavior.
