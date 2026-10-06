#!/usr/bin/env bash
set -euo pipefail

# Real locked/offline Cargo metadata; Git observations and publication are
# substitutes. No registry requests, commits, tags or uploads occur.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-host-publish-test.XXXXXX")"
finish() {
    local status=$?
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else echo "Publication fixtures retained: $fixture" >&2; fi
}
trap finish EXIT
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
mkdir "$fixture/bin"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/README.md" "$root/LICENSE" "$fixture/"
for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
    mkdir -p "$fixture/crates/$package/src"
    cp "$root/crates/$package/Cargo.toml" "$fixture/crates/$package/"
    cp "$root/LICENSE" "$fixture/crates/$package/LICENSE"
    : > "$fixture/crates/$package/src/lib.rs"
done
cd "$fixture"
cargo metadata --no-deps --format-version 1 --locked --offline > metadata.json
cp metadata.json original-metadata.json
export PUBLISH_FIXTURE="$fixture"
cat > bin/git <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    'rev-parse --show-toplevel') echo "$PUBLISH_FIXTURE" ;;
    'rev-parse --verify HEAD') echo 0000000000000000000000000000000000000000 ;;
    'status --porcelain --untracked-files=all')
        if [[ "${PUBLISH_DIRTY:-0}" == 1 ]]; then echo ' M Cargo.toml'; fi ;;
    *) echo "Unexpected Git operation: $*" >&2; exit 2 ;;
esac
STUB
cat > bin/cargo <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$1" >> "$PUBLISH_FIXTURE/calls"
case "$1" in
    metadata) cat "$PUBLISH_FIXTURE/metadata.json"; exit "${PUBLISH_METADATA_RESULT:-0}" ;;
    publish)
        # Intent must exist before dispatch, including when Cargo then fails.
        intents=(target/publish/*/intent)
        [[ -f "${intents[0]}" ]]
        printf '%s\0' "$@" > "$PUBLISH_FIXTURE/arguments"
        mkdir -p target/package
        printf 'retained package\n' > target/package/fixture.crate
        echo 'Substitute Cargo publication output'
        exit "${PUBLISH_CARGO_RESULT:-0}" ;;
    *) echo "Unexpected Cargo operation: $*" >&2; exit 2 ;;
esac
STUB
chmod +x bin/git bin/cargo
export PATH="$fixture/bin:$PATH"
mkdir target
printf 'consumer artifact\n' > target/retained
invoke() { bash "$root/scripts/publish/workspace.sh" "$@"; }
refuse() {
    : > calls
    if invoke "$@" > refusal.log 2>&1; then
        echo 'Publication unexpectedly succeeded' >&2; exit 1
    fi
    [[ ! -s calls ]] || exit 1
}
# A dry run can review dirty work; an upload cannot admit it.
PUBLISH_DIRTY=1 refuse publish
# License copies must remain regular and identical before either Cargo entry.
notice=crates/ic-host-artifacts/LICENSE
for scenario in missing changed symlink; do
    case "$scenario" in
        missing) rm "$notice" ;;
        changed) printf '\nchanged notice\n' >> "$notice" ;;
        symlink) rm "$notice"; ln -s ../../LICENSE "$notice" ;;
    esac
    refuse check
    refuse publish
    rm -f "$notice"
    cp LICENSE "$notice"
done
PUBLISH_DIRTY=1 invoke check > check.log 2>&1
printf '%s\0' publish --manifest-path "$fixture/Cargo.toml" --workspace \
    --registry crates-io --all-features --locked --target-dir "$fixture/target" \
    --dry-run --allow-dirty > expected-arguments
cmp expected-arguments arguments

: > calls
invoke publish > publish.log 2>&1
printf '%s\0' publish --manifest-path "$fixture/Cargo.toml" --workspace \
    --registry crates-io --all-features --locked --target-dir "$fixture/target" > expected-arguments
cmp expected-arguments arguments
printf 'metadata\npublish\n' > expected-calls
cmp expected-calls calls

# Stop at Cargo's failure and retain the attempt's intent, log and artifacts.
: > calls
status=0
PUBLISH_CARGO_RESULT=17 invoke publish > failed.log 2>&1 || status=$?
[[ "$status" == 17 ]] || exit 1
cmp expected-calls calls
attempt="$(awk '/^Publication evidence: / { sub(/^Publication evidence: /, ""); print }' failed.log)"
[[ -f "$attempt/intent" && -f "$attempt/arguments" && -f "$attempt/cargo.log" && "$(cat "$attempt/status")" == 17 ]]
[[ -f target/package/fixture.crate && ! -e target/publish/lock ]]

# Plausible stdout from a failed metadata query is never upload authority.
: > calls
if PUBLISH_METADATA_RESULT=17 invoke publish > metadata-failure.log 2>&1; then exit 1; fi
printf 'metadata\n' > expected-calls
cmp expected-calls calls
for scenario in disabled mixed-version extra-package; do
    case "$scenario" in
        disabled) jq '.packages[0].publish = []' original-metadata.json > metadata.json ;;
        mixed-version) jq '.packages[0].version = "99.0.0"' original-metadata.json > metadata.json ;;
        extra-package) jq '.packages[0].name = "unexpected-owner"' original-metadata.json > metadata.json ;;
    esac
    : > calls
    if invoke publish > "$scenario.log" 2>&1; then exit 1; fi
    cmp expected-calls calls
done
cp original-metadata.json metadata.json
mkdir target/publish/lock
refuse publish
[[ -d target/publish/lock ]] || exit 1
rmdir target/publish/lock
printf 'consumer artifact\n' > expected-retained
cmp expected-retained target/retained
echo 'Publication command checks passed (Git and upload effects substituted).'
