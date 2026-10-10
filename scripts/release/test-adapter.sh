#!/usr/bin/env bash
set -euo pipefail

# Exercise the consumer adapter with real locked/offline Cargo metadata. Git,
# setup, fetching, qualification and version mutation are substitutes: no
# commits, tags or release pushes. CI does not need cargo-edit for this fixture.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-host-release-adapter.XXXXXX")"
completed=false
finish() {
    local status=$?
    # Bash 3.2 may enter EXIT with status zero after a nounset error.
    [[ "$completed" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else echo "Release adapter fixtures retained: $fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
ADAPTER_REAL_CARGO="$(command -v cargo)"
export ADAPTER_REAL_CARGO
ADAPTER_REAL_BASH="$(command -v bash)"
ADAPTER_REAL_AWK="$(command -v awk)"
export ADAPTER_REAL_BASH ADAPTER_REAL_AWK
export ADAPTER_METADATA_REWRITE="$root/scripts/release/metadata-version.awk"
export ADAPTER_EVENTS="$fixture/events"
export RELEASE_SOURCE=0000000000000000000000000000000000000000
export RELEASE_DATE=2026-10-06 RELEASE_KIND=minor
adapter_root="$fixture/adapter"
for input in scripts/release/adapter.sh scripts/release/metadata-version.awk \
    scripts/ci/read-cargo-workspace-version.sh scripts/ci/check-release-source.sh \
    scripts/ci/next-release-version.sh scripts/ci/finalize-release-changelog.awk; do
    mkdir -p "$adapter_root/${input%/*}"
    cp "$root/$input" "$adapter_root/$input"
done
adapter="$adapter_root/scripts/release/adapter.sh"
# The real selector/installer boundary is covered by test-tools.sh. Here the
# selected release executable uses the existing version-mutation substitute.
cat > "$adapter_root/scripts/release/tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == run ]] || exit 1
shift
exec cargo set-version "$@"
STUB
RELEASE_PREVIOUS="$(cd "$root" && bash scripts/release/adapter.sh version)"
export RELEASE_PREVIOUS
RELEASE_VERSION="$(bash "$root/scripts/ci/next-release-version.sh" "$RELEASE_PREVIOUS" "$RELEASE_KIND")"
export RELEASE_VERSION
mkdir "$fixture/bin"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/README.md" "$root/LICENSE" "$fixture/"
# A valid inline comment must survive observation, preparation and recovery.
awk '/^version = / { $0 = $0 " # retained workspace version comment" } { print }' \
    "$fixture/Cargo.toml" > "$fixture/Cargo.toml.commented"
mv "$fixture/Cargo.toml.commented" "$fixture/Cargo.toml"
for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
    mkdir -p "$fixture/crates/$package/src"
    # Git release qualification remains independent of registry eligibility.
    sed 's/^publish = .*/publish = false/' "$root/crates/$package/Cargo.toml" > "$fixture/crates/$package/Cargo.toml"
    : > "$fixture/crates/$package/src/lib.rs"
done
# Observe the real workspace through relative and absolute adapter entrypoints
# in a physical newline-bearing tooling root, with a competing CDPATH lookup.
tooling="$fixture/tooling"$'\n'
mkdir -p "$tooling/scripts/release" "$tooling/scripts/ci" "$fixture/decoy/scripts/release"
cp "$adapter" "$tooling/scripts/release/"
cp "$root/scripts/ci/read-cargo-workspace-version.sh" "$tooling/scripts/ci/"
(
    cd "$fixture"
    export CDPATH="$fixture/decoy:$fixture"
    [[ "$(bash "$tooling/scripts/release/adapter.sh" version)" == "$RELEASE_PREVIOUS" ]] || exit 1
    [[ "$(bash "${tooling#"$fixture/"}/scripts/release/adapter.sh" version)" == "$RELEASE_PREVIOUS" ]] || exit 1
)
# Heading presentation must not detach notes from the selected release.
printf '# Changelog\n\n## [%s] \t \n\n- Fixture notes.\n' "$RELEASE_VERSION" > "$fixture/CHANGELOG.md"
printf '\n## [%s]\n\n- Undated imported history.\n\n## [0.0.1] - 2026-10-01\n\n- Dated history.\n' "$RELEASE_PREVIOUS" >> "$fixture/CHANGELOG.md"
mkdir "$fixture/source" "$fixture/committed"
cp "$fixture/Cargo.toml" "$fixture/Cargo.lock" "$fixture/CHANGELOG.md" "$fixture/source/"
export ADAPTER_SOURCE="$fixture/source" ADAPTER_COMMITTED="$fixture/committed"
export RELEASE_COMMIT=1111111111111111111111111111111111111111
cat > "$fixture/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    'rev-parse HEAD') echo "${ADAPTER_HEAD:-$RELEASE_SOURCE}" ;;
    'rev-parse --git-path release-state') echo release-state ;;
    "rev-parse $RELEASE_COMMIT^{commit}") echo "$RELEASE_COMMIT" ;;
    'diff --binary HEAD') ;;
    'hash-object --stdin') cat > /dev/null; echo "$RELEASE_SOURCE" ;;
    show\ *)
        case "$2" in
            "$RELEASE_SOURCE:"*) cat "$ADAPTER_SOURCE/${2#*:}" ;;
            "$RELEASE_COMMIT:"*) cat "$ADAPTER_COMMITTED/${2#*:}" ;;
            *) exit 2 ;;
        esac ;;
    'rev-parse --show-prefix') ;;
    'status --porcelain=v1 -z --untracked-files=all')
        if [[ -n "${ADAPTER_DIRTY_PATH:-}" ]]; then printf ' M %s\0' "$ADAPTER_DIRTY_PATH"; fi
        if [[ -n "${ADAPTER_STAGED_PATH:-}" ]]; then printf 'M  %s\0' "$ADAPTER_STAGED_PATH"; fi
        if [[ -n "${ADAPTER_UNTRACKED_PATH:-}" ]]; then printf '?? %s\0' "$ADAPTER_UNTRACKED_PATH"; fi ;;
    *) echo "Unexpected Git operation: $*" >&2; exit 2 ;;
esac
if [[ "$*" == "${ADAPTER_GIT_QUERY:-}" ]]; then
    count=0
    if [[ -f query-count ]]; then read -r count < query-count; fi
    count=$((count + 1))
    printf '%s\n' "$count" > query-count
    if [[ "$count" == "${ADAPTER_GIT_FAILURE_AT:-1}" ]]; then exit 23; fi
fi
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >> "$ADAPTER_EVENTS"
case "$*" in
    'locate-project --workspace --message-format plain --manifest-path Cargo.toml')
        "$ADAPTER_REAL_CARGO" "$@"
        exit "${ADAPTER_LOCATE_RESULT:-0}" ;;
    'metadata --no-deps --format-version 1 --locked --offline')
        "$ADAPTER_REAL_CARGO" "$@"
        exit "${ADAPTER_METADATA_RESULT:-0}" ;;
    'set-version --help') ;;
    'fetch --locked'|'fetch --locked --offline')
        if [[ -n "${ADAPTER_CACHE:-}" && ! -e "$ADAPTER_CACHE" ]]; then
            if [[ "$*" == *--offline || "${CARGO_NET_OFFLINE:-}" == true ]]; then
                echo 'fixture: locked input unavailable offline' >&2; exit 101
            fi
            if [[ -n "${ADAPTER_FETCH_RESULT:-}" ]]; then
                echo 'fixture: registry unavailable' >&2; exit "$ADAPTER_FETCH_RESULT"
            fi
            : > "$ADAPTER_CACHE"
        fi ;;
    "set-version --workspace --offline $RELEASE_VERSION")
        for mode in manifest lock; do
            if [[ "$mode" == manifest ]]; then file=Cargo.toml; else file=Cargo.lock; fi
            awk -v mode="$mode" -v replacement="$RELEASE_VERSION" \
                -f "$ADAPTER_METADATA_REWRITE" "$file" > "$file.candidate"
            mv "$file.candidate" "$file"
        done ;;
    *) echo "Unexpected Cargo operation: $*" >&2; exit 2 ;;
esac
STUB
cat > "$fixture/bin/make" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    '--no-print-directory install-tools'|'--no-print-directory tools-check'|'--no-print-directory dependency-pins-check'|'--no-print-directory ci'|--no-print-directory\ msrv\ PACKAGE=ic-host-*)
        printf 'make %s\n' "$*" >> "$ADAPTER_EVENTS" ;;
    *) echo "Unexpected Make operation: $*" >&2; exit 2 ;;
esac
STUB
chmod +x "$fixture/bin/git" "$fixture/bin/cargo" "$fixture/bin/make"
# Delegate normally, then fail selected readers after their valid stdout.
printf '#!%s\n' "$ADAPTER_REAL_BASH" > "$fixture/bin/bash"
cat >> "$fixture/bin/bash" <<'STUB'
set -euo pipefail
if [[ "${1##*/}" == "${ADAPTER_FAILED_READER:-}" ]]; then
    "$ADAPTER_REAL_BASH" "$@"
    exit 23
fi
exec "$ADAPTER_REAL_BASH" "$@"
STUB
cat > "$fixture/bin/awk" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
"$ADAPTER_REAL_AWK" "$@"
if [[ "${ADAPTER_FAILED_READER:-}" == lock && "$*" == '-v mode=lock -v read_version=1 '* ]]; then exit 23; fi
STUB
chmod +x "$fixture/bin/bash" "$fixture/bin/awk"
export PATH="$fixture/bin:$PATH"
cd "$fixture"

# A failed shared reader must not leak plausible version stdout into admission.
mkdir failed-parser-bin
cat > failed-parser-bin/jq <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$RELEASE_PREVIOUS"
exit 17
STUB
chmod +x failed-parser-bin/jq
: > "$ADAPTER_EVENTS"
if PATH="$fixture/failed-parser-bin:$PATH" bash "$adapter" version \
    > failed-reader-version.log 2>&1; then
    echo 'Version observation accepted failed parser output' >&2; exit 1
fi
[[ ! -s failed-reader-version.log ]] || exit 1
if PATH="$fixture/failed-parser-bin:$PATH" bash "$adapter" preflight \
    > rejected-parser.log 2>&1; then
    echo 'Preflight accepted failed parser output' >&2; exit 1
fi
printf '%s\n' 'cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml' \
    'cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml' > expected
cmp expected "$ADAPTER_EVENTS"

# Cargo validity is checked before TOML projection, including failed queries
# that print plausible paths and duplicate workspace version keys.
for scenario in failed-locate duplicate-version unsynchronized-path; do
    : > "$ADAPTER_EVENTS"
    cp source/Cargo.toml Cargo.toml
    case "$scenario" in
        failed-locate) export ADAPTER_LOCATE_RESULT=17 ;;
        duplicate-version)
            awk '/^version = / { print } { print }' source/Cargo.toml > Cargo.toml ;;
        unsynchronized-path)
            awk '$1 == "ic-host-fs" { sub(/version = "[^"]+"/, "version = \"99.0.0\"") } { print }' \
                source/Cargo.toml > Cargo.toml ;;
    esac
    cp Cargo.toml rejected-input
    if bash "$adapter" preflight > "rejected-$scenario.log" 2>&1; then
        echo "Preflight accepted $scenario manifest" >&2; exit 1
    fi
    unset ADAPTER_LOCATE_RESULT
    printf '%s\n' 'cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml' > expected
    cmp expected "$ADAPTER_EVENTS"
    cmp rejected-input Cargo.toml
    cmp source/Cargo.lock Cargo.lock
    [[ ! -e release-state ]] || exit 1
done
cp source/Cargo.toml Cargo.toml

# Plausible stdout never overrides a failed Git observation. Admission failures
# must stop before setup, qualification or version mutation in every entrypoint.
for operation in preflight verify prepare; do
    : > "$ADAPTER_EVENTS"
    rm -f query-count
    status=0
    ADAPTER_GIT_QUERY='rev-parse HEAD' bash "$adapter" "$operation" \
        > "failed-head-$operation.log" 2>&1 || status=$?
    [[ "$status" == 23 && ! -s "$ADAPTER_EVENTS" && ! -e release-state ]] || exit 1
    cmp source/Cargo.toml Cargo.toml
    cmp source/Cargo.lock Cargo.lock
    cmp source/CHANGELOG.md CHANGELOG.md
done

# Release selection, manifest and lockfile readers have the same status boundary.
for reader in next-release-version.sh read-cargo-workspace-version.sh lock; do
    : > "$ADAPTER_EVENTS"
    status=0
    ADAPTER_FAILED_READER="$reader" bash "$adapter" preflight \
        > "failed-reader-$reader.log" 2>&1 || status=$?
    [[ "$status" == 23 && ! -e release-state ]] || exit 1
    if grep -E '^make |^cargo (fetch|set-version)' "$ADAPTER_EVENTS"; then exit 1; fi
    cmp source/Cargo.toml Cargo.toml
    cmp source/Cargo.lock Cargo.lock
    cmp source/CHANGELOG.md CHANGELOG.md
done

# The copied member manifests retain publish=false. Preflight must reach setup
# and dependency admission without modifying their metadata or creating intent.
: > "$ADAPTER_EVENTS"
bash "$adapter" preflight > accepted.log 2>&1
cat > expected <<'EVENTS'
cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml
cargo metadata --no-deps --format-version 1 --locked --offline
make --no-print-directory install-tools
make --no-print-directory tools-check
make --no-print-directory dependency-pins-check
cargo set-version --help
cargo fetch --locked
EVENTS
cmp expected "$ADAPTER_EVENTS"
cp expected accepted-events
cmp Cargo.toml source/Cargo.toml
cmp Cargo.lock "$root/Cargo.lock"
[[ ! -e release-state ]] || exit 1

# Missing selected inputs are prepared only by preflight. All fetch effects here
# are substituted; real metadata remains locked/offline even with this env unset.
for scenario in online offline network verify source; do
    : > "$ADAPTER_EVENTS"
    cache="$fixture/cache-$scenario"
    status=0
    (
        unset CARGO_NET_OFFLINE
        export ADAPTER_CACHE="$cache"
        operation=preflight
        case "$scenario" in
            offline) export CARGO_NET_OFFLINE=true ;;
            network) export ADAPTER_FETCH_RESULT=73 ;;
            verify) operation=verify ;;
            source) export ADAPTER_HEAD=2222222222222222222222222222222222222222 ;;
        esac
        bash "$adapter" "$operation"
    ) > "cache-$scenario.log" 2>&1 || status=$?
    case "$scenario" in
        online) [[ "$status" == 0 && -f "$cache" ]] || exit 1; cmp accepted-events "$ADAPTER_EVENTS" ;;
        offline|verify) [[ "$status" == 101 && ! -e "$cache" ]] || exit 1 ;;
        network) [[ "$status" == 73 && ! -e "$cache" ]] || exit 1 ;;
        source) [[ "$status" != 0 && ! -e "$cache" && ! -s "$ADAPTER_EVENTS" ]] || exit 1 ;;
    esac
    if [[ "$scenario" == verify ]]; then
        grep -Fx 'cargo fetch --locked --offline' "$ADAPTER_EVENTS" > /dev/null
        if grep -F 'install-tools' "$ADAPTER_EVENTS"; then
            echo 'Standalone verification attempted tool setup' >&2; exit 1
        fi
    fi
    if grep -E '^make .* (ci|msrv)|^cargo set-version --workspace' "$ADAPTER_EVENTS"; then
        echo 'Cache preparation dispatched validation or version mutation' >&2; exit 1
    fi
    [[ ! -e release-state ]] || exit 1
    cmp Cargo.lock source/Cargo.lock
    cmp Cargo.toml source/Cargo.toml
    cmp CHANGELOG.md source/CHANGELOG.md
done

# Complete and major.minor requirements both admit the synchronized workspace.
# Preserve each form's precision through the actual adapter transaction below.
for requirement in "$RELEASE_PREVIOUS" "${RELEASE_PREVIOUS%.*}"; do
    awk -v requirement="$requirement" '
        /^ic-host-/ { sub(/version = "[^"]+"/, "version = \"" requirement "\"") }
        { print }
    ' source/Cargo.toml > Cargo.toml
    : > "$ADAPTER_EVENTS"
    bash "$adapter" preflight > "accepted-$requirement.log" 2>&1
    cmp accepted-events "$ADAPTER_EVENTS"
done
cp source/Cargo.toml Cargo.toml

# A failed Cargo query may print valid metadata: its failure must still stop
# preflight before any setup or fetch is attempted.
: > "$ADAPTER_EVENTS"
if ADAPTER_METADATA_RESULT=17 bash "$adapter" preflight > rejected-metadata.log 2>&1; then
    echo 'Preflight accepted failed metadata validation' >&2; exit 1
fi
printf '%s\n' 'cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml' \
    'cargo metadata --no-deps --format-version 1 --locked --offline' > expected
cmp expected "$ADAPTER_EVENTS"

# Keep the independent dirty-source admission boundary ahead of setup.
: > "$ADAPTER_EVENTS"
if ADAPTER_DIRTY_PATH=crates/ic-host-tools/src/lib.rs ADAPTER_STAGED_PATH=Cargo.lock \
    ADAPTER_UNTRACKED_PATH='unexpected source.rs' \
    bash "$adapter" preflight > rejected-source.log 2>&1; then
    echo 'Preflight accepted uncommitted source' >&2; exit 1
fi
grep -F 'unstaged: crates/ic-host-tools/src/lib.rs' rejected-source.log > /dev/null
grep -F 'staged: Cargo.lock' rejected-source.log > /dev/null
grep -F 'untracked: unexpected\ source.rs' rejected-source.log > /dev/null
printf '%s\n' 'cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml' > expected
cmp expected "$ADAPTER_EVENTS"

# Complete-ledger selection rejects duplicates and competing future candidates.
# Each failure must stop before setup and leave the changelog untouched.
printf '%s\n' 'cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml' \
    'cargo metadata --no-deps --format-version 1 --locked --offline' > expected
for scenario in unnumbered misplaced-unnumbered duplicate duplicate-whitespace competing misplaced misplaced-whitespace finalized finalized-whitespace finalized-other-date; do
    case "$scenario" in
        unnumbered)
            printf '# Changelog\n\n## [Draft]\n\n- Candidate without identity.\n' > CHANGELOG.md ;;
        misplaced-unnumbered)
            cp source/CHANGELOG.md CHANGELOG.md
            printf '\n## [Draft]\n\n- Candidate without identity.\n' >> CHANGELOG.md ;;
        duplicate)
            cp source/CHANGELOG.md CHANGELOG.md
            printf '\n## [%s]\n\n- Duplicate.\n' "$RELEASE_VERSION" >> CHANGELOG.md ;;
        duplicate-whitespace)
            cp source/CHANGELOG.md CHANGELOG.md
            printf '\n## [%s] \t \n\n- Duplicate.\n' "$RELEASE_VERSION" >> CHANGELOG.md ;;
        competing)
            cp source/CHANGELOG.md CHANGELOG.md
            future="$(bash "$root/scripts/ci/next-release-version.sh" "$RELEASE_VERSION" minor)"
            printf '\n## [%s]\n\n- Competing draft.\n' "$future" >> CHANGELOG.md ;;
        misplaced)
            printf '# Changelog\n\n## [%s]\n\n- History.\n\n## [%s]\n\n- Misplaced draft.\n' "$RELEASE_PREVIOUS" "$RELEASE_VERSION" > CHANGELOG.md ;;
        misplaced-whitespace)
            printf '# Changelog\n\n## [%s]\n\n- History.\n\n## [%s] \t \n\n- Misplaced draft.\n' "$RELEASE_PREVIOUS" "$RELEASE_VERSION" > CHANGELOG.md ;;
        finalized)
            printf '# Changelog\n\n## [%s] - %s\n\n- Finalized.\n' "$RELEASE_VERSION" "$RELEASE_DATE" > CHANGELOG.md ;;
        finalized-whitespace)
            printf '# Changelog\n\n##\t[%s]\t -  %s \t\n\n- Finalized.\n' "$RELEASE_VERSION" "$RELEASE_DATE" > CHANGELOG.md ;;
        finalized-other-date)
            printf '# Changelog\n\n##\t[%s]\t-\t2001-01-01\n\n- Finalized.\n' "$RELEASE_VERSION" > CHANGELOG.md ;;
    esac
    cp CHANGELOG.md before
    : > "$ADAPTER_EVENTS"
    if bash "$adapter" preflight > "rejected-$scenario.log" 2>&1; then
        echo "Preflight accepted $scenario changelog" >&2; exit 1
    fi
    cmp expected "$ADAPTER_EVENTS"
    cmp before CHANGELOG.md
done
cp source/CHANGELOG.md CHANGELOG.md

# Initial and post-validation diff observations must preserve producer status,
# including a failed diff whose downstream hash command still prints a match.
for query in 'rev-parse HEAD' 'diff --binary HEAD' 'hash-object --stdin'; do
    for occurrence in 1 2; do
        : > "$ADAPTER_EVENTS"
        rm -f query-count
        status=0
        ADAPTER_GIT_QUERY="$query" ADAPTER_GIT_FAILURE_AT="$occurrence" \
            bash "$adapter" verify > "failed-verify-$occurrence.log" 2>&1 || status=$?
        [[ "$status" == 23 && ! -e "release-state/$RELEASE_VERSION.validation" &&
            ! -e "release-state/$RELEASE_VERSION.validation.notes" ]] || exit 1
        if [[ "$occurrence" == 1 ]]; then
            if grep -E '^make .* (ci|msrv)' "$ADAPTER_EVENTS"; then exit 1; fi
        else
            grep -Fx 'make --no-print-directory msrv PACKAGE=ic-host-tools' "$ADAPTER_EVENTS" > /dev/null
            [[ -f "release-state/$RELEASE_VERSION.validation.log" ]] || exit 1
        fi
        cmp source/Cargo.toml Cargo.toml
        cmp source/Cargo.lock Cargo.lock
        cmp source/CHANGELOG.md CHANGELOG.md
    done
done

# Verification qualifies the source without installing tools. The gate and
# MSRV checks are substitutes; saved notes and receipt handling are real.
: > "$ADAPTER_EVENTS"
bash "$adapter" verify > verified.log 2>&1
cat > expected <<'EVENTS'
cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml
cargo metadata --no-deps --format-version 1 --locked --offline
make --no-print-directory tools-check
make --no-print-directory dependency-pins-check
cargo set-version --help
cargo fetch --locked --offline
make --no-print-directory ci
make --no-print-directory msrv PACKAGE=ic-host-artifacts
make --no-print-directory msrv PACKAGE=ic-host-fs
make --no-print-directory msrv PACKAGE=ic-host-process
make --no-print-directory msrv PACKAGE=ic-host-tools
EVENTS
cmp expected "$ADAPTER_EVENTS"
receipt="release-state/$RELEASE_VERSION.validation"
cmp source/CHANGELOG.md "$receipt.notes"

# Failed preparation and requalification preserve an existing receipt verbatim.
cp "$receipt" saved-receipt
for operation in prepare verify; do
    rm -f query-count
    status=0
    occurrence=1
    if [[ "$operation" == verify ]]; then occurrence=2; fi
    ADAPTER_GIT_QUERY='rev-parse HEAD' ADAPTER_GIT_FAILURE_AT="$occurrence" \
        bash "$adapter" "$operation" \
        > "failed-receipted-$operation.log" 2>&1 || status=$?
    [[ "$status" == 23 ]] || exit 1
    cmp saved-receipt "$receipt"
    cmp source/CHANGELOG.md "$receipt.notes"
done

# The prepared fixture must preserve both undated and dated historical notes
# through saved receipts; real locked/offline Cargo admits its updated graph.
printf '# Changelog\n\n## [%s] - %s\n\n- Fixture notes.\n\n## [%s]\n\n- Undated imported history.\n\n## [0.0.1] - 2026-10-01\n\n- Dated history.\n' "$RELEASE_VERSION" "$RELEASE_DATE" "$RELEASE_PREVIOUS" > expected-notes
bash "$adapter" prepare > prepared.log 2>&1
cmp expected-notes CHANGELOG.md
bash "$adapter" check > checked.log 2>&1
cp Cargo.toml Cargo.lock CHANGELOG.md committed/

for operation in check commit-check; do
    status=0
    ADAPTER_FAILED_READER=read-cargo-workspace-version.sh bash "$adapter" "$operation" \
        > "failed-prepared-$operation.log" 2>&1 || status=$?
    [[ "$status" == 23 ]] || exit 1
    cmp committed/Cargo.toml Cargo.toml
    cmp committed/Cargo.lock Cargo.lock
    cmp committed/CHANGELOG.md CHANGELOG.md
done

# Recovery must also reject a failed lookup of an otherwise matching commit.
cp "$receipt" saved-receipt
rm -f query-count
status=0
ADAPTER_GIT_QUERY="rev-parse $RELEASE_COMMIT^{commit}" \
    bash "$adapter" committed-check > failed-committed-query.log 2>&1 || status=$?
[[ "$status" == 23 ]] || exit 1
cmp saved-receipt "$receipt"
cmp source/CHANGELOG.md "$receipt.notes"
cmp committed/Cargo.toml Cargo.toml
cmp committed/Cargo.lock Cargo.lock
cmp committed/CHANGELOG.md CHANGELOG.md

# Recovery observes the selected committed payload beneath newer working notes;
# it must not reinterpret current files or accept edits to the selected history.
printf '# Newer working notes\n' > CHANGELOG.md
printf '# Newer working metadata\n' > Cargo.toml
printf '# Newer working lockfile\n' > Cargo.lock
bash "$adapter" committed-check > committed.log 2>&1
printf '\nAltered historical notes.\n' >> committed/CHANGELOG.md
if bash "$adapter" committed-check > rejected-committed.log 2>&1; then
    echo 'Committed check accepted an altered release payload' >&2; exit 1
fi
cp expected-notes committed/CHANGELOG.md
printf '\n' >> "$receipt"
if bash "$adapter" committed-check > rejected-receipt.log 2>&1; then
    echo 'Committed check accepted conflicting validation evidence' >&2; exit 1
fi

# With no pending section, the previous undated entry remains history and the
# shared finalizer inserts exactly one candidate during preparation.
cp source/Cargo.toml source/Cargo.lock .
printf '# Changelog\n\n## [%s]\n\n- Undated imported history.\n' "$RELEASE_PREVIOUS" > CHANGELOG.md
: > "$ADAPTER_EVENTS"
bash "$adapter" preflight > history-only-preflight.log 2>&1
cmp accepted-events "$ADAPTER_EVENTS"
bash "$adapter" verify > history-only-verify.log 2>&1
bash "$adapter" prepare > history-only-prepare.log 2>&1
printf '# Changelog\n\n## [%s] - %s\n\n## [%s]\n\n- Undated imported history.\n' "$RELEASE_VERSION" "$RELEASE_DATE" "$RELEASE_PREVIOUS" > expected-notes
cmp expected-notes CHANGELOG.md

# Horizontal whitespace is accepted without changing historical bytes or adding
# a final newline. Admission and preparation must agree on the same candidate.
cp source/Cargo.toml source/Cargo.lock .
printf '# Changelog\n\n##\t [%s] \t\n\n- Candidate.\n\n## [%s]\n\n- History without final LF.' "$RELEASE_VERSION" "$RELEASE_PREVIOUS" > CHANGELOG.md
bash "$adapter" preflight > whitespace-preflight.log 2>&1
bash "$adapter" verify > whitespace-verify.log 2>&1
bash "$adapter" prepare > whitespace-prepare.log 2>&1
printf '# Changelog\n\n## [%s] - %s\n\n- Candidate.\n\n## [%s]\n\n- History without final LF.' "$RELEASE_VERSION" "$RELEASE_DATE" "$RELEASE_PREVIOUS" > expected-notes
cmp expected-notes CHANGELOG.md
echo 'Release adapter checks passed (setup, qualification and Git effects substituted).'
completed=true
