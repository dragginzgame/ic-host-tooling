#!/usr/bin/env bash
set -euo pipefail

# Real Git histories, metadata edits and tags stay inside this disposable fixture.
# Registry calls are strict substitutes. No source-checkout index/ref effects.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH
unset RELEASE_KIND RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_SOURCE RELEASE_COMMIT
editor="${CARGO_RELEASE_METADATA_EDITOR:?prepare and select the published Cargo release editor}"
limit="${CARGO_RELEASE_METADATA_MAX_BYTES:?select the release document byte allowance}"
[[ "$editor" == /* && -f "$editor" && -x "$editor" && "$limit" =~ ^[1-9][0-9]*$ ]] || exit 2
export RELEASE_DELIVERY=direct
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"; root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-build-release-adapters.XXXXXX")"
complete=false
failed_command=''
finish() {
    local status=$? log latest=''
    [[ "$complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else
        echo "Release adapter fixture retained: $fixture" >&2
        printf 'Failed adapter command: %.2000s\n' "$failed_command" >&2
        for log in "$fixture"/*.log; do
            [[ -f "$log" ]] || continue
            if [[ -z "$latest" || "$log" -nt "$latest" ]]; then latest="$log"; fi
        done
        if [[ -n "$latest" ]]; then
            printf 'Last adapter log (up to 80 lines): %s\n' "$latest" >&2
            tail -n 80 "$latest" | awk '{ print substr($0, 1, 1000) }' >&2 || :
        fi
    fi
    exit "$status"
}
trap finish EXIT
trap 'failed_command=$BASH_COMMAND' ERR
mkdir -p "$fixture/workspace" "$fixture/bin" "$fixture/tmp"
export TMPDIR="$fixture/tmp"
for path in Cargo.toml Cargo.lock CHANGELOG.md README.md LICENSE Makefile .gitignore rust-toolchain.toml crates scripts make ci; do
    cp -R "$root/$path" "$fixture/workspace/$path"
done
mkdir -p "$fixture/workspace/.tools"
cp -P "$root"/.tools/host* "$fixture/workspace/.tools/"
cp -R "$root/.tools/rust" "$fixture/workspace/.tools/rust"
export YQ="$root/.tools/host/bin/yq"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
export IC_BUILD_TEST_REAL_CARGO IC_BUILD_TEST_REAL_MAKE
IC_BUILD_TEST_REAL_CARGO="$(command -v cargo)"
IC_BUILD_TEST_REAL_MAKE="$(command -v make)"
export IC_BUILD_TEST_CARGO_CALLS="$fixture/cargo.calls"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == publish ]]; then
    printf '%s\n' "$*" >> "$IC_BUILD_TEST_CARGO_CALLS"
    exit "${IC_BUILD_TEST_PUBLISH_STATUS:-0}"
fi
if [[ "$1" == metadata && "${IC_BUILD_TEST_FAIL_METADATA:-0}" == 1 ]]; then exit 42; fi
if [[ "$1" == fetch && "${IC_BUILD_TEST_PREFLIGHT:-0}" == 1 ]]; then
    [[ -z "${IC_BUILD_RELEASE_CACHE_PREPARE+x}" ]] || exit 99
    printf 'cargo:%s\n' "$*" >> "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
    exit "${IC_BUILD_TEST_FETCH_STATUS:-0}"
fi
exec "$IC_BUILD_TEST_REAL_CARGO" "$@"
CARGO
cat > "$fixture/bin/make" <<'MAKE'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${IC_BUILD_TEST_PREFLIGHT:-0}" == 1 && $# == 4 && "$1" == --no-print-directory && "$2" == -C ]]; then
    [[ "$3" == "$IC_BUILD_TEST_PREFLIGHT_ROOT" && -z "${IC_BUILD_RELEASE_CACHE_PREPARE+x}" ]] || exit 99
    printf 'make:%s\n' "$4" >> "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
    if [[ -n "${IC_BUILD_TEST_CHANGE_PIN:-}" ]]; then
        printf '\n# changed during preflight\n' >> "$IC_BUILD_TEST_CHANGE_PIN"
    fi
    exit "${IC_BUILD_TEST_MAKE_STATUS:-0}"
fi
exec "$IC_BUILD_TEST_REAL_MAKE" "$@"
MAKE
chmod +x "$fixture/bin/cargo" "$fixture/bin/make"
export PATH="$fixture/bin:$PATH"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
cd "$fixture/workspace"
perl -0pi -e 's/^check ci:$/check:/m; s/^check-secrets:\n(?:\t[^\n]*\n)+//m; s/^check-shell:\n(?:\t[^\n]*\n)+//m; s/^check-workflows:\n(?:\t[^\n]*\n)+//m' Makefile
cat >> Makefile <<GATES

ci lint check-secrets:
	@printf '%s\\n' '\$@' >> '$fixture/gates'
check-shell check-workflows:
	@:
GATES
# The adapter fixture exercises a patch independently of the live release draft.
fixture_previous="$(make --no-print-directory -s release-version)"
fixture_version="$(bash scripts/ci/next-release-version.sh "$fixture_previous" patch)"
printf '# Changelog\n\n## [%s]\n\n- Exercise the release adapter transaction.\n' "$fixture_version" > CHANGELOG.md
git init --quiet --template=
git config user.name Fixture
git config user.email fixture@example.invalid
git config core.hooksPath /dev/null
git add -- Cargo.toml Cargo.lock CHANGELOG.md README.md LICENSE Makefile .gitignore rust-toolchain.toml crates scripts make ci
git commit --quiet -m 'Adapter fixture baseline'
export RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_KIND
RELEASE_SOURCE="$(git rev-parse HEAD)"
RELEASE_PREVIOUS="$(make --no-print-directory -s release-version)"
RELEASE_KIND=patch RELEASE_DATE=2026-10-12
RELEASE_VERSION="$(bash scripts/ci/next-release-version.sh "$RELEASE_PREVIOUS" patch)"
expect_failure() {
    if "$@" > "$fixture/refusal.log" 2>&1; then
        echo 'adapter accepted an invalid fixture' >&2; exit 1
    fi
}
# The actual logger recursively loads this fixture's gate sentinels.
# The verification adapter itself remains the production recipe.
make --no-print-directory release-verify > "$fixture/gate.log" 2>&1
printf 'ci\nlint\ncheck-secrets\n' > "$fixture/expected-gates"
cmp "$fixture/gates" "$fixture/expected-gates"
cp Makefile "$fixture/pass-Makefile"
cat >> Makefile <<'GATES'

ci:
	@exit 23
lint check-secrets:
	@exit 97
GATES
expect_failure make --no-print-directory release-verify
cp "$fixture/pass-Makefile" Makefile

make --no-print-directory release-preflight > "$fixture/preflight.log" 2>&1
# Exercise this adapter's common catalog selection and authority boundary.
export IC_BUILD_TEST_PREFLIGHT=1 IC_BUILD_TEST_PREFLIGHT_ROOT="$PWD"
export IC_BUILD_TEST_PREFLIGHT_EVENTS="$fixture/preflight.events"
for mode in 0 1; do
    : > "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
    IC_BUILD_RELEASE_CACHE_PREPARE="$mode" bash scripts/release/metadata.sh preflight > "$fixture/preflight-$mode.log" 2>&1
    if [[ "$mode" == 0 ]]; then
        printf 'cargo:fetch --locked --offline\nmake:tools-check\n' > "$fixture/preflight.expected"
    else
        printf 'cargo:fetch --locked\nmake:install-tools\nmake:tools-check\n' > "$fixture/preflight.expected"
    fi
    cmp "$fixture/preflight.expected" "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
done
cp ci/tool-versions.env "$fixture/preflight-pin"
: > "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
status=0
IC_BUILD_RELEASE_CACHE_PREPARE=1 IC_BUILD_TEST_CHANGE_PIN="$PWD/ci/tool-versions.env" \
    bash scripts/release/metadata.sh preflight > "$fixture/preflight-pin.log" 2>&1 || status=$?
cp "$fixture/preflight-pin" ci/tool-versions.env
[[ "$status" == 1 ]]
printf 'cargo:fetch --locked\nmake:install-tools\n' > "$fixture/preflight.expected"
cmp "$fixture/preflight.expected" "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
for kind in fetch make invalid; do
    : > "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
    status=0
    case "$kind" in
        fetch)
            IC_BUILD_TEST_FETCH_STATUS=43 IC_BUILD_RELEASE_CACHE_PREPARE=1 \
                bash scripts/release/metadata.sh preflight > "$fixture/preflight-fetch.log" 2>&1 || status=$?
            [[ "$status" == 43 ]]
            printf 'cargo:fetch --locked\n' > "$fixture/preflight.expected" ;;
        make)
            IC_BUILD_TEST_MAKE_STATUS=49 IC_BUILD_RELEASE_CACHE_PREPARE=1 \
                bash scripts/release/metadata.sh preflight > "$fixture/preflight-make.log" 2>&1 || status=$?
            [[ "$status" == 49 ]]
            printf 'cargo:fetch --locked\nmake:install-tools\n' > "$fixture/preflight.expected" ;;
        invalid)
            IC_BUILD_RELEASE_CACHE_PREPARE=2 bash scripts/release/metadata.sh preflight > "$fixture/preflight-invalid.log" 2>&1 || status=$?
            [[ "$status" == 2 ]]
            : > "$fixture/preflight.expected" ;;
    esac
    cmp "$fixture/preflight.expected" "$IC_BUILD_TEST_PREFLIGHT_EVENTS"
done
unset IC_BUILD_TEST_PREFLIGHT
printf '\n// unrelated\n' >> crates/ic-build/src/lib.rs
expect_failure make --no-print-directory release-preflight
git restore -- crates/ic-build/src/lib.rs
printf '\nunrelated\n' >> README.md
git add -- README.md
git show HEAD:README.md > README.md
expect_failure make --no-print-directory release-preflight
git reset --quiet HEAD -- README.md
for path in Cargo.toml Cargo.lock CHANGELOG.md; do cp "$path" "$fixture/before-${path##*/}"; done
for invalid in missing-editor relative-editor missing-limit zero-limit document-limit; do
    case "$invalid" in
        missing-editor) expect_failure env -u CARGO_RELEASE_METADATA_EDITOR make --no-print-directory release-prepare-version ;;
        relative-editor) expect_failure env CARGO_RELEASE_METADATA_EDITOR=editor make --no-print-directory release-prepare-version ;;
        missing-limit) expect_failure env -u CARGO_RELEASE_METADATA_MAX_BYTES make --no-print-directory release-prepare-version ;;
        zero-limit) expect_failure env CARGO_RELEASE_METADATA_MAX_BYTES=0 make --no-print-directory release-prepare-version ;;
        document-limit) expect_failure env CARGO_RELEASE_METADATA_MAX_BYTES=1 make --no-print-directory release-prepare-version ;;
    esac
    for path in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$path" "$fixture/before-${path##*/}"; done
done
expect_failure env IC_BUILD_TEST_FAIL_METADATA=1 make --no-print-directory release-prepare-version
for path in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$path" "$fixture/before-${path##*/}"; done
make --no-print-directory release-prepare-version > "$fixture/prepare.log" 2>&1
[[ "$(make --no-print-directory -s release-version)" == "$RELEASE_VERSION" ]]
make --no-print-directory release-prepared-check > "$fixture/prepared.log" 2>&1
# Lost preparation replies are idempotent at the saved candidate.
make --no-print-directory release-prepare-version > "$fixture/reprepare.log" 2>&1
cp Cargo.toml "$fixture/sorted-Cargo.toml"
perl -0pi -e 's/^(cargo_metadata =[^\n]*\n)(clap =[^\n]*\n)/$2$1/m' Cargo.toml
if cmp -s Cargo.toml "$fixture/sorted-Cargo.toml"; then exit 1; fi
cp Cargo.toml "$fixture/unsorted-Cargo.toml"
expect_failure make --no-print-directory fmt-check
expect_failure make --no-print-directory release-prepared-check
expect_failure make --no-print-directory release-prepare-version
cmp Cargo.toml "$fixture/unsorted-Cargo.toml"
cp "$fixture/sorted-Cargo.toml" Cargo.toml
"$YQ" -p toml -o json '.' "$fixture/before-Cargo.lock" | jq '[.package[] | select(.source != null)]' > "$fixture/external-before"
"$YQ" -p toml -o json '.' Cargo.lock | jq '[.package[] | select(.source != null)]' > "$fixture/external-after"
cmp "$fixture/external-before" "$fixture/external-after"
make --no-print-directory -s release-files > "$fixture/release-files"
printf '%s\0' Cargo.toml Cargo.lock CHANGELOG.md > "$fixture/expected-files"
cmp "$fixture/release-files" "$fixture/expected-files"
git add -- Cargo.toml Cargo.lock CHANGELOG.md
perl -0pi -e 's/^(cargo_metadata =[^\n]*\n)(clap =[^\n]*\n)/$2$1/m' Cargo.toml
git add -- Cargo.toml
expect_failure make --no-print-directory release-commit-check
cp "$fixture/sorted-Cargo.toml" Cargo.toml
git add -- Cargo.toml
make --no-print-directory release-commit-check > "$fixture/commit-check.log" 2>&1
git commit --quiet -m "Release $RELEASE_VERSION"
export RELEASE_COMMIT
RELEASE_COMMIT="$(git rev-parse HEAD)"
make --no-print-directory release-committed-check > "$fixture/committed.log" 2>&1
# Committed checks inspect the selected export, not the current manifest.
perl -0pi -e 's/^(cargo_metadata =[^\n]*\n)(clap =[^\n]*\n)/$2$1/m' Cargo.toml
make --no-print-directory release-committed-check > "$fixture/committed-dirty-manifest.log" 2>&1
git add -- Cargo.toml
unsorted_commit="$(printf 'Unsorted fixture manifest\n' | git commit-tree "$(git write-tree)" -p HEAD)"
cp "$fixture/sorted-Cargo.toml" Cargo.toml
git add -- Cargo.toml
expect_failure env RELEASE_COMMIT="$unsorted_commit" make --no-print-directory release-committed-check
make --no-print-directory release-committed-check > "$fixture/committed-restored.log" 2>&1
mkdir -p target
printf 'retained release artifact\n' > target/retained-artifact
expect_failure make --no-print-directory publish
[[ ! -e "$IC_BUILD_TEST_CARGO_CALLS" ]]
git tag -a "v$RELEASE_VERSION" -m "Release $RELEASE_VERSION"
make --no-print-directory publish-check > "$fixture/publish-check.log" 2>&1
[[ "$(cat "$IC_BUILD_TEST_CARGO_CALLS")" == 'publish -p ic-build --locked --registry crates-io --dry-run' ]]
make --no-print-directory publish > "$fixture/publish.log" 2>&1
[[ "$(tail -n 1 "$IC_BUILD_TEST_CARGO_CALLS")" == 'publish -p ic-build --locked --registry crates-io' ]]
expect_failure env IC_BUILD_TEST_PUBLISH_STATUS=24 make --no-print-directory publish
printf '\nnewer source\n' >> README.md
git add -- README.md
git commit --quiet -m 'Newer source'
expect_failure make --no-print-directory publish
# Late recovery validates the selected release commit, not the newer branch tip.
make --no-print-directory release-committed-check > "$fixture/older-commit.log" 2>&1
[[ "$(cat target/retained-artifact)" == 'retained release artifact' ]]
complete=true
echo 'IC Build metadata, gate routing, recovery and publication fixtures passed (no live release effects).'
