#!/usr/bin/env bash
set -euo pipefail

# Exercise the consumer adapter with real locked/offline Cargo metadata. Git,
# setup, fetching, qualification and version mutation are substitutes: no
# commits, tags or release pushes. CI does not need cargo-edit for this fixture.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-host-release-adapter.XXXXXX")"
finish() {
    local status=$?
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else echo "Release adapter fixtures retained: $fixture" >&2; fi
}
trap finish EXIT
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
ADAPTER_REAL_CARGO="$(command -v cargo)"
export ADAPTER_REAL_CARGO
export ADAPTER_METADATA_REWRITE="$root/scripts/release/metadata-version.awk"
export ADAPTER_EVENTS="$fixture/events"
export RELEASE_SOURCE=0000000000000000000000000000000000000000
export RELEASE_DATE=2026-10-06 RELEASE_KIND=minor
RELEASE_PREVIOUS="$(cd "$root" && bash scripts/release/adapter.sh version)"
export RELEASE_PREVIOUS
RELEASE_VERSION="$(bash "$root/scripts/ci/next-release-version.sh" "$RELEASE_PREVIOUS" "$RELEASE_KIND")"
export RELEASE_VERSION
mkdir "$fixture/bin"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/README.md" "$fixture/"
for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
    mkdir -p "$fixture/crates/$package/src"
    cp "$root/crates/$package/Cargo.toml" "$fixture/crates/$package/"
    : > "$fixture/crates/$package/src/lib.rs"
done
printf '# Changelog\n\n## [%s]\n\n- Fixture notes.\n' "$RELEASE_VERSION" > "$fixture/CHANGELOG.md"
printf '\n## [%s]\n\n- Undated imported history.\n\n## [0.0.1] - 2026-10-01\n\n- Dated history.\n' "$RELEASE_PREVIOUS" >> "$fixture/CHANGELOG.md"
mkdir "$fixture/source" "$fixture/committed"
cp "$fixture/Cargo.toml" "$fixture/Cargo.lock" "$fixture/CHANGELOG.md" "$fixture/source/"
export ADAPTER_SOURCE="$fixture/source" ADAPTER_COMMITTED="$fixture/committed"
export RELEASE_COMMIT=1111111111111111111111111111111111111111
cat > "$fixture/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    'rev-parse HEAD') echo "$RELEASE_SOURCE" ;;
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
    'diff --name-only -z --cached HEAD --'|'ls-files --others --exclude-standard -z') ;;
    'diff --name-only -z --')
        if [[ -n "${ADAPTER_DIRTY_PATH:-}" ]]; then printf '%s\0' "$ADAPTER_DIRTY_PATH"; fi ;;
    *) echo "Unexpected Git operation: $*" >&2; exit 2 ;;
esac
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >> "$ADAPTER_EVENTS"
case "$*" in
    'metadata --no-deps --format-version 1 --locked --offline')
        "$ADAPTER_REAL_CARGO" "$@"
        exit "${ADAPTER_METADATA_RESULT:-0}" ;;
    'set-version --help'|'fetch --locked --offline') ;;
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
    '--no-print-directory install-host-tools'|'--no-print-directory host-tools-check'|'--no-print-directory dependency-pins-check'|'--no-print-directory ci'|--no-print-directory\ msrv\ PACKAGE=ic-host-*)
        printf 'make %s\n' "$*" >> "$ADAPTER_EVENTS" ;;
    *) echo "Unexpected Make operation: $*" >&2; exit 2 ;;
esac
STUB
chmod +x "$fixture/bin/git" "$fixture/bin/cargo" "$fixture/bin/make"
export PATH="$fixture/bin:$PATH"
cd "$fixture"
# The copied member manifests retain publish=false. Preflight must reach setup
# and dependency admission without modifying their metadata or creating intent.
: > "$ADAPTER_EVENTS"
bash "$root/scripts/release/adapter.sh" preflight > accepted.log 2>&1
cat > expected <<'EVENTS'
cargo metadata --no-deps --format-version 1 --locked --offline
make --no-print-directory install-host-tools
make --no-print-directory host-tools-check
make --no-print-directory dependency-pins-check
cargo set-version --help
cargo fetch --locked --offline
EVENTS
cmp expected "$ADAPTER_EVENTS"
cp expected accepted-events
cmp Cargo.toml "$root/Cargo.toml"
cmp Cargo.lock "$root/Cargo.lock"
[[ ! -e release-state ]] || exit 1

# A failed Cargo query may print valid metadata: its failure must still stop
# preflight before any setup or fetch is attempted.
: > "$ADAPTER_EVENTS"
if ADAPTER_METADATA_RESULT=17 bash "$root/scripts/release/adapter.sh" preflight > rejected-metadata.log 2>&1; then
    echo 'Preflight accepted failed metadata validation' >&2; exit 1
fi
printf '%s\n' 'cargo metadata --no-deps --format-version 1 --locked --offline' > expected
cmp expected "$ADAPTER_EVENTS"

# Keep the independent dirty-source admission boundary ahead of setup.
: > "$ADAPTER_EVENTS"
if ADAPTER_DIRTY_PATH=crates/ic-host-tools/src/lib.rs bash "$root/scripts/release/adapter.sh" preflight > rejected-source.log 2>&1; then
    echo 'Preflight accepted uncommitted source' >&2; exit 1
fi
[[ ! -s "$ADAPTER_EVENTS" ]] || exit 1

# Complete-ledger selection rejects duplicates and competing future candidates.
# Each failure must stop before setup and leave the changelog untouched.
for scenario in duplicate competing misplaced finalized; do
    case "$scenario" in
        duplicate)
            cp source/CHANGELOG.md CHANGELOG.md
            printf '\n## [%s]\n\n- Duplicate.\n' "$RELEASE_VERSION" >> CHANGELOG.md ;;
        competing)
            cp source/CHANGELOG.md CHANGELOG.md
            future="$(bash "$root/scripts/ci/next-release-version.sh" "$RELEASE_VERSION" minor)"
            printf '\n## [%s]\n\n- Competing draft.\n' "$future" >> CHANGELOG.md ;;
        misplaced)
            printf '# Changelog\n\n## [%s]\n\n- History.\n\n## [%s]\n\n- Misplaced draft.\n' "$RELEASE_PREVIOUS" "$RELEASE_VERSION" > CHANGELOG.md ;;
        finalized)
            printf '# Changelog\n\n## [%s] - %s\n\n- Finalized.\n' "$RELEASE_VERSION" "$RELEASE_DATE" > CHANGELOG.md ;;
    esac
    cp CHANGELOG.md before
    : > "$ADAPTER_EVENTS"
    if bash "$root/scripts/release/adapter.sh" preflight > "rejected-$scenario.log" 2>&1; then
        echo "Preflight accepted $scenario changelog" >&2; exit 1
    fi
    cmp expected "$ADAPTER_EVENTS"
    cmp before CHANGELOG.md
done
cp source/CHANGELOG.md CHANGELOG.md

# Verification qualifies the source without installing tools. The gate and
# MSRV checks are substitutes; saved notes and receipt handling are real.
: > "$ADAPTER_EVENTS"
bash "$root/scripts/release/adapter.sh" verify > verified.log 2>&1
cat > expected <<'EVENTS'
cargo metadata --no-deps --format-version 1 --locked --offline
make --no-print-directory host-tools-check
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

# The prepared fixture must preserve both undated and dated historical notes
# through saved receipts; real locked/offline Cargo admits its updated graph.
printf '# Changelog\n\n## [%s] - %s\n\n- Fixture notes.\n\n## [%s]\n\n- Undated imported history.\n\n## [0.0.1] - 2026-10-01\n\n- Dated history.\n' "$RELEASE_VERSION" "$RELEASE_DATE" "$RELEASE_PREVIOUS" > expected-notes
bash "$root/scripts/release/adapter.sh" prepare > prepared.log 2>&1
cmp expected-notes CHANGELOG.md
bash "$root/scripts/release/adapter.sh" check > checked.log 2>&1
cp Cargo.toml Cargo.lock CHANGELOG.md committed/

# Recovery observes the selected committed payload beneath newer working notes;
# it must not reinterpret current files or accept edits to the selected history.
printf '# Newer working notes\n' > CHANGELOG.md
printf '# Newer working metadata\n' > Cargo.toml
printf '# Newer working lockfile\n' > Cargo.lock
bash "$root/scripts/release/adapter.sh" committed-check > committed.log 2>&1
printf '\nAltered historical notes.\n' >> committed/CHANGELOG.md
if bash "$root/scripts/release/adapter.sh" committed-check > rejected-committed.log 2>&1; then
    echo 'Committed check accepted an altered release payload' >&2; exit 1
fi
cp expected-notes committed/CHANGELOG.md
printf '\n' >> "$receipt"
if bash "$root/scripts/release/adapter.sh" committed-check > rejected-receipt.log 2>&1; then
    echo 'Committed check accepted conflicting validation evidence' >&2; exit 1
fi

# With no pending section, the previous undated entry remains history and the
# shared finalizer inserts exactly one candidate during preparation.
cp source/Cargo.toml source/Cargo.lock .
printf '# Changelog\n\n## [%s]\n\n- Undated imported history.\n' "$RELEASE_PREVIOUS" > CHANGELOG.md
: > "$ADAPTER_EVENTS"
bash "$root/scripts/release/adapter.sh" preflight > history-only-preflight.log 2>&1
cmp accepted-events "$ADAPTER_EVENTS"
bash "$root/scripts/release/adapter.sh" verify > history-only-verify.log 2>&1
bash "$root/scripts/release/adapter.sh" prepare > history-only-prepare.log 2>&1
printf '# Changelog\n\n## [%s] - %s\n\n## [%s]\n\n- Undated imported history.\n' "$RELEASE_VERSION" "$RELEASE_DATE" "$RELEASE_PREVIOUS" > expected-notes
cmp expected-notes CHANGELOG.md
echo 'Release adapter checks passed (setup, qualification and Git effects substituted).'
