#!/usr/bin/env bash
set -euo pipefail

# Real locked/offline Cargo metadata; Git observations and publication are
# substitutes. No registry requests, commits, tags or uploads occur.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture_parent="$(mktemp -d "${TMPDIR:-/tmp}/ic-host-publish-test.XXXXXX")"
# Git emits this literal checkout name followed by its own record newline.
fixture="$fixture_parent/checkout"$'\n'
completed=false
finish() {
    local status=$?
    # Bash 3.2 may enter EXIT with status zero after a nounset error.
    [[ "$completed" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture_parent"
    else echo "Publication fixtures retained: $fixture_parent" >&2; fi
    exit "$status"
}
trap finish EXIT
mkdir "$fixture"
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
    'rev-parse --verify HEAD')
        if [[ -e "$PUBLISH_FIXTURE/source-changed" ]]; then
            echo 1111111111111111111111111111111111111111
        else echo 0000000000000000000000000000000000000000; fi ;;
    'status --porcelain --untracked-files=all')
        if [[ "${PUBLISH_DIRTY:-0}" == 1 ]]; then echo ' M Cargo.toml'; fi ;;
    init\ --bare\ *) mkdir -p "$3" ;;
    -C\ *\ fetch\ --no-tags\ --depth=1\ *)
        [[ "$6" == https://github.com/dragginzgame/ic-host-tooling &&
           "$7" == 0000000000000000000000000000000000000000 ]] || exit 1
        printf '%s\n' "$*" >> "$PUBLISH_FIXTURE/fetch-calls"
        exit "${PUBLISH_FETCH_RESULT:-0}" ;;
    -C\ *\ rev-parse\ --verify\ *)
        echo "${PUBLISH_RETRIEVED_SOURCE:-0000000000000000000000000000000000000000}" ;;
    *) echo "Unexpected Git operation: $*" >&2; exit 2 ;;
esac
if [[ "$*" == "${PUBLISH_GIT_QUERY:-}" ]]; then
    count=0
    if [[ -f "$PUBLISH_FIXTURE/query-count" ]]; then read -r count < "$PUBLISH_FIXTURE/query-count"; fi
    count=$((count + 1))
    printf '%s\n' "$count" > "$PUBLISH_FIXTURE/query-count"
    if [[ "$count" == "$PUBLISH_GIT_FAILURE_AT" ]]; then exit 23; fi
fi
STUB
cat > bin/cargo <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$1" >> "$PUBLISH_FIXTURE/calls"
case "$1" in
    metadata) cat "$PUBLISH_FIXTURE/metadata.json"; exit "${PUBLISH_METADATA_RESULT:-0}" ;;
    package)
        [[ "${PUBLISH_PACKAGE_RESULT:-0}" == 0 ]] || exit "$PUBLISH_PACKAGE_RESULT"
        printf '%s\0' "$@" > "$PUBLISH_FIXTURE/package-arguments"
        mkdir -p target/package
        version="$(jq -r '.packages[0].version' "$PUBLISH_FIXTURE/metadata.json")"
        for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
            directory="target/package/$package-$version"
            mkdir -p "$directory"
            jq -n --arg source "${PUBLISH_PACKAGE_SOURCE:-0000000000000000000000000000000000000000}" \
                --arg path "${PUBLISH_PACKAGE_PATH:-crates/$package}" --argjson dirty "${PUBLISH_PACKAGE_DIRTY:-false}" \
                '{git: {sha1: $source, dirty: $dirty}, path_in_vcs: $path}' > "$directory/.cargo_vcs_info.json"
            if [[ "${PUBLISH_VCS_MISSING:-0}" == 1 ]]; then rm "$directory/.cargo_vcs_info.json"; fi
            tar -czf "$directory.crate" -C target/package "$package-$version"
        done
        if [[ "${PUBLISH_CHANGE_SOURCE:-0}" == 1 ]]; then : > "$PUBLISH_FIXTURE/source-changed"; fi ;;
    publish)
        # Intent must exist before dispatch, including when Cargo then fails.
        intents=(target/publish/*/intent)
        [[ -f "${intents[0]}" ]] || exit 1
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
[[ ! -e fetch-calls && ! -e package-arguments ]] || exit 1

: > calls
invoke publish > publish.log 2>&1
printf '%s\0' publish --manifest-path "$fixture/Cargo.toml" --workspace \
    --registry crates-io --all-features --locked --target-dir "$fixture/target" > expected-arguments
cmp expected-arguments arguments
printf 'metadata\npackage\npublish\n' > expected-calls
cmp expected-calls calls
printf '%s\0' package --manifest-path "$fixture/Cargo.toml" --workspace \
    --registry crates-io --all-features --locked --no-verify --target-dir "$fixture/target" > expected-package-arguments
cmp expected-package-arguments package-arguments
attempts=(target/publish/publish.*)
[[ ${#attempts[@]} == 1 ]] || exit 1
attempt="${attempts[0]}"
successful_attempt="$attempt"
[[ -f "$attempt/source-verified" && -f "$attempt/source-request" && -f "$attempt/ic-host-fs-vcs.json" ]] || exit 1

# Stop at Cargo's failure and retain the attempt's intent, log and artifacts.
: > calls
status=0
PUBLISH_CARGO_RESULT=17 invoke publish > failed.log 2>&1 || status=$?
[[ "$status" == 17 ]] || exit 1
cmp expected-calls calls
attempts=(target/publish/publish.*)
[[ ${#attempts[@]} == 2 ]] || exit 1
for attempt in "${attempts[@]}"; do
    [[ "$attempt" == "$successful_attempt" ]] || break
done
[[ -f "$attempt/intent" && -f "$attempt/arguments" && -f "$attempt/cargo.log" && "$(cat "$attempt/status")" == 17 ]] || exit 1
[[ -f target/package/fixture.crate && ! -e target/publish/lock ]] || exit 1

# Plausible stdout from a failed metadata query is never upload authority.
: > calls
if PUBLISH_METADATA_RESULT=17 invoke publish > metadata-failure.log 2>&1; then exit 1; fi
printf 'metadata\n' > expected-calls
cmp expected-calls calls
for scenario in disabled mixed-version extra-package wrong-repository; do
    case "$scenario" in
        disabled) jq '.packages[0].publish = []' original-metadata.json > metadata.json ;;
        mixed-version) jq '.packages[0].version = "99.0.0"' original-metadata.json > metadata.json ;;
        extra-package) jq '.packages[0].name = "unexpected-owner"' original-metadata.json > metadata.json ;;
        wrong-repository) jq '.packages[0].repository = "https://github.com/other/repository"' original-metadata.json > metadata.json ;;
    esac
    : > calls
    if invoke publish > "$scenario.log" 2>&1; then exit 1; fi
    cmp expected-calls calls
done
cp original-metadata.json metadata.json
# Neither stale archives nor plausible failed retrieval can authorize upload.
for scenario in package-failed vcs-missing vcs-dirty vcs-mismatch vcs-path fetch-failed fetch-mismatch source-changed; do
    : > calls
    status=0
    case "$scenario" in
        package-failed) PUBLISH_PACKAGE_RESULT=17 invoke publish > "$scenario.log" 2>&1 || status=$? ;;
        vcs-missing) PUBLISH_VCS_MISSING=1 invoke publish > "$scenario.log" 2>&1 || status=$? ;;
        vcs-dirty) PUBLISH_PACKAGE_DIRTY=true invoke publish > "$scenario.log" 2>&1 || status=$? ;;
        vcs-mismatch) PUBLISH_PACKAGE_SOURCE=1111111111111111111111111111111111111111 invoke publish > "$scenario.log" 2>&1 || status=$? ;;
        vcs-path) PUBLISH_PACKAGE_PATH=wrong/path invoke publish > "$scenario.log" 2>&1 || status=$? ;;
        fetch-failed) PUBLISH_FETCH_RESULT=17 invoke publish > "$scenario.log" 2>&1 || status=$? ;;
        fetch-mismatch) PUBLISH_RETRIEVED_SOURCE=1111111111111111111111111111111111111111 invoke publish > "$scenario.log" 2>&1 || status=$? ;;
        source-changed) PUBLISH_CHANGE_SOURCE=1 invoke publish > "$scenario.log" 2>&1 || status=$? ;;
    esac
    [[ "$status" != 0 ]] || exit 1
    printf 'metadata\npackage\n' > expected-calls
    cmp expected-calls calls
    [[ ! -e target/publish/lock ]] || exit 1
    rm -f source-changed
done
# Failed observations must stop both modes, even with expected identity stdout
# or an empty clean-status result. Later failures retain their attempt evidence.
for mode in check publish; do
    for query in 'rev-parse --verify HEAD' 'status --porcelain --untracked-files=all'; do
        for occurrence in 1 2; do
            : > calls
            rm -f query-count arguments
            status=0
            PUBLISH_GIT_QUERY="$query" PUBLISH_GIT_FAILURE_AT="$occurrence" \
                invoke "$mode" > observation-failure.log 2>&1 || status=$?
            [[ "$status" == 23 ]] || { echo 'Failed Git observation was not propagated' >&2; exit 1; }
            : > expected-calls
            if [[ "$occurrence" == 2 ]]; then
                printf 'metadata\n' >> expected-calls
                if [[ "$mode" == publish ]]; then printf 'package\n' >> expected-calls; fi
                # The fixture root includes a newline; resolve by the relative attempt suffix.
                attempt="$(sed -n 's,^/target/publish/,target/publish/,p' observation-failure.log)"
                [[ -f "$attempt/intent" && ! -e "$attempt/cargo.log" && ! -e "$attempt/status" ]] || exit 1
            fi
            cmp expected-calls calls
            [[ ! -e target/publish/lock && ! -e arguments ]] || exit 1
        done
    done
done
mkdir target/publish/lock
refuse publish
[[ -d target/publish/lock ]] || exit 1
rmdir target/publish/lock
printf 'consumer artifact\n' > expected-retained
cmp expected-retained target/retained
echo 'Publication command checks passed (Git and upload effects substituted).'
completed=true
