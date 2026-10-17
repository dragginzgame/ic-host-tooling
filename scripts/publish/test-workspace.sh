#!/usr/bin/env bash
set -euo pipefail

# Real locked/offline Cargo metadata; Git observations and publication are
# substitutes. No registry requests, commits, tags or uploads occur.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
FIXTURE_ROOT="$(fixture_create "${TMPDIR:-/tmp}/ic-host-publish-test.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$FIXTURE_ROOT" "Publication fixtures retained"' EXIT
export TMPDIR="${FIXTURE_ROOT%/*}"
# Spaces are supported; forbidden directory bytes are tested as refusals below.
fixture="$FIXTURE_ROOT/checkout with spaces"
mkdir "$fixture"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
mkdir "$fixture/bin"
bash "$root/scripts/ci/prepare-workspace-fixture.sh" "$root" "$fixture"
cd "$fixture"
# Fixture mechanics follow independently prepared Cargo members/paths. Host's
# adapter retains the separate explicit publication eligibility policy.
real_cargo="$(command -v cargo)"
capture_metadata() {
    "$real_cargo" metadata --no-deps --format-version 1 --locked --offline > metadata.json
    jq -r --arg root "$fixture/" '
    . as $m | [.packages[] | select(.id as $id | $m.workspace_members | index($id))]
    | sort_by(.name)[]
    | [.name, (.manifest_path | if startswith($root) then
        ltrimstr($root) | rtrimstr("/Cargo.toml") else error("fixture manifest escapes root") end)]
    | @tsv
    ' metadata.json > fixture-packages.tsv
}
capture_metadata
cp metadata.json original-metadata.json
cp Cargo.toml original-Cargo.toml
export PUBLISH_FIXTURE="$fixture" REAL_PUBLISH_TEE
REAL_PUBLISH_TEE="$(command -v tee)"
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
        if [[ "${PUBLISH_CORE_RACE:-0}" == 1 && -d target/publish/.cargo-publication.lock ]]; then echo ' M raced'; fi
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
        while IFS=$'\t' read -r package package_root; do
            directory="target/package/$package-$version"
            mkdir -p "$directory"
            jq -n --arg source "${PUBLISH_PACKAGE_SOURCE:-0000000000000000000000000000000000000000}" \
                --arg path "${PUBLISH_PACKAGE_PATH:-$package_root}" --argjson dirty "${PUBLISH_PACKAGE_DIRTY:-false}" \
                '{git: {sha1: $source, dirty: $dirty}, path_in_vcs: $path}' > "$directory/.cargo_vcs_info.json"
            if [[ "${PUBLISH_VCS_MISSING:-0}" == 1 ]]; then rm "$directory/.cargo_vcs_info.json"; fi
            tar -czf "$directory.crate" -C target/package "$package-$version"
        done < "$PUBLISH_FIXTURE/fixture-packages.tsv"
        if [[ "${PUBLISH_CHANGE_SOURCE:-0}" == 1 ]]; then : > "$PUBLISH_FIXTURE/source-changed"; fi ;;
    publish)
        # Intent must exist before dispatch, including when Cargo then fails.
        intents=(target/publish/admission/*/intent)
        [[ -f "${intents[0]}" ]] || exit 1
        for intent in "${intents[@]}"; do
            tail -n +6 "$intent" | LC_ALL=C sort > "$PUBLISH_FIXTURE/intent-packages"
            jq -r '. as $m | .packages[] | select(.id as $id | $m.workspace_members | index($id)) | .name' \
                "$PUBLISH_FIXTURE/metadata.json" | LC_ALL=C sort > "$PUBLISH_FIXTURE/metadata-packages"
            cmp "$PUBLISH_FIXTURE/intent-packages" "$PUBLISH_FIXTURE/metadata-packages"
        done
        core_intents=(target/publish/*/intent)
        [[ -f "${core_intents[0]}" ]] || exit 1
        for intent in "${core_intents[@]}"; do
            tail -n +4 "$intent" | LC_ALL=C sort > "$PUBLISH_FIXTURE/core-packages"
            version="$(jq -r '.packages[0].version' "$PUBLISH_FIXTURE/metadata.json")"
            sed "s/\$/@$version/" "$PUBLISH_FIXTURE/metadata-packages" > "$PUBLISH_FIXTURE/exact-packages"
            cmp "$PUBLISH_FIXTURE/core-packages" "$PUBLISH_FIXTURE/exact-packages"
        done
        printf '%s\0' "$@" > "$PUBLISH_FIXTURE/arguments"
        mkdir -p target/package
        printf 'retained package\n' > target/package/fixture.crate
        echo 'Substitute Cargo publication output'
        if [[ "${PUBLISH_INTERRUPT:-0}" == 1 ]]; then kill -TERM "$PPID"; fi
        exit "${PUBLISH_CARGO_RESULT:-0}" ;;
    *) echo "Unexpected Cargo operation: $*" >&2; exit 2 ;;
esac
STUB
cat > bin/tee <<'STUB'
#!/usr/bin/env bash
"$REAL_PUBLISH_TEE" "$@"
exit "${PUBLISH_LOG_RESULT:-0}"
STUB
chmod +x bin/git bin/cargo bin/tee
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
for separator in $'\n' $'\r'; do
    bad="$FIXTURE_ROOT/bad${separator}checkout"
    mkdir "$bad"
    PUBLISH_FIXTURE="$bad" refuse check
    rg 'CR/LF' refusal.log > /dev/null
done
# Package license checks follow metadata admission and precede package/upload.
IFS=$'\t' read -r _ notice_root < fixture-packages.tsv
notice="$notice_root/LICENSE"
for scenario in missing changed symlink; do
    case "$scenario" in
        missing) rm "$notice" ;;
        changed) printf '\nchanged notice\n' >> "$notice" ;;
        symlink) rm "$notice"; ln -s ../../LICENSE "$notice" ;;
    esac
    for mode in check publish; do
        : > calls
        if invoke "$mode" > refusal.log 2>&1; then exit 1; fi
        printf 'metadata\n' > expected-calls
        cmp expected-calls calls
    done
    rm -f "$notice"
    cp LICENSE "$notice"
done
PUBLISH_DIRTY=1 invoke check > check.log 2>&1
version="$(jq -r '.packages[0].version' metadata.json)"
selectors=()
while IFS=$'\t' read -r package _; do
    selectors+=(--package "$package@$version")
done < fixture-packages.tsv
printf '%s\0' publish --locked --registry crates-io "${selectors[@]}" \
    --manifest-path "$fixture/Cargo.toml" --target-dir "$fixture/target" \
    --all-features --allow-dirty --dry-run > expected-arguments
cmp expected-arguments arguments
[[ ! -e fetch-calls && ! -e package-arguments ]] || exit 1

: > calls
invoke publish > publish.log 2>&1
printf '%s\0' publish --locked --registry crates-io "${selectors[@]}" \
    --manifest-path "$fixture/Cargo.toml" --target-dir "$fixture/target" \
    --all-features > expected-arguments
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
admission="target/publish/admission/${attempt##*/}"
[[ -f "$admission/source-verified" && -f "$admission/source-request" && -f "$admission/ic-host-fs-vcs.json" ]] || exit 1
[[ "$(cat "$admission/publication-attempt")" == "$fixture/$attempt" ]] || exit 1

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

# Shared owns status precedence and the complete Cargo/logger failure matrix.
# Host proves delegation preserves its status and releases the preparation lock.
: > calls
status=0
PUBLISH_CARGO_RESULT=0 PUBLISH_LOG_RESULT=23 \
    invoke publish > logger-failure.log 2>&1 || status=$?
[[ "$status" == 23 ]] || exit 1
cmp expected-calls calls
core="$(sed -n 's/^Publication evidence: //p' logger-failure.log)"
[[ "$(cat "$core/status")" == "$status" &&
   ! -e target/publish/lock && ! -e target/publish/.cargo-publication.lock ]] || exit 1

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
# Package directories follow real Cargo observations after an allowed layout move.
mv crates/ic-host-artifacts 'apps/relocated artifacts'
sed 's|crates/ic-host-artifacts|apps/relocated artifacts|g' original-Cargo.toml > Cargo.toml
capture_metadata
invoke check > relocated-check.log 2>&1
invoke publish > relocated-publish.log 2>&1
mv 'apps/relocated artifacts' crates/ic-host-artifacts
cp original-Cargo.toml Cargo.toml
capture_metadata
cmp original-metadata.json metadata.json

# Plausible metadata cannot redirect license/provenance reads outside the checkout.
mkdir "$FIXTURE_ROOT/outside"
cp LICENSE "$FIXTURE_ROOT/outside/LICENSE"
cp crates/ic-host-artifacts/Cargo.toml "$FIXTURE_ROOT/outside/Cargo.toml"
ln -s "$FIXTURE_ROOT/outside" apps/package-alias
for path in "$FIXTURE_ROOT/outside/Cargo.toml" "$fixture/../outside/Cargo.toml" \
    "$fixture/crates//ic-host-artifacts/Cargo.toml" \
    "$fixture/crates/./ic-host-artifacts/Cargo.toml" \
    "$fixture/"$'line\nfeed/Cargo.toml' "$fixture/"$'carriage\rreturn/Cargo.toml' \
    "$fixture/apps/package-alias/Cargo.toml"; do
    jq --arg path "$path" '(.packages[] | select(.name == "ic-host-artifacts").manifest_path) = $path' \
        original-metadata.json > metadata.json
    for mode in check publish; do
        : > calls
        if invoke "$mode" > invalid-path.log 2>&1; then exit 1; fi
        printf 'metadata\n' > expected-calls
        cmp expected-calls calls
    done
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
        for occurrence in 1 2 3 4; do
            : > calls
            rm -f query-count arguments
            status=0
            PUBLISH_GIT_QUERY="$query" PUBLISH_GIT_FAILURE_AT="$occurrence" \
                invoke "$mode" > observation-failure.log 2>&1 || status=$?
            [[ "$status" == 23 ]] || { echo 'Failed Git observation was not propagated' >&2; exit 1; }
            : > expected-calls
            if [[ "$occurrence" != 1 ]]; then
                printf 'metadata\n' >> expected-calls
                if [[ "$mode" == publish ]]; then printf 'package\n' >> expected-calls; fi
                admission="$(sed -n 's/^Publication admission evidence: //p' observation-failure.log)"
                core="$(cat "$admission/publication-attempt")"
                [[ -f "$admission/intent" && ! -e "$core/cargo.log" && ! -e "$core/status" ]] || exit 1
                if [[ "$occurrence" == 4 ]]; then
                    [[ -f "$core/intent" && ! -e "$core/dispatch" &&
                       -f target/publish/.cargo-publication.lock/owner ]] || exit 1
                    rm target/publish/.cargo-publication.lock/owner
                    rmdir target/publish/.cargo-publication.lock
                fi
            fi
            cmp expected-calls calls
            [[ ! -e target/publish/lock && ! -e arguments ]] || exit 1
        done
    done
done

# Source drift after shared intent refuses dispatch and blocks a fresh attempt.
: > calls
status=0
PUBLISH_CORE_RACE=1 invoke check > core-race.log 2>&1 || status=$?
[[ "$status" == 2 && -f target/publish/.cargo-publication.lock/owner ]] || exit 1
printf 'metadata\n' > expected-race-calls
cmp expected-race-calls calls
refuse publish
[[ -f target/publish/.cargo-publication.lock/owner ]] || exit 1
rm target/publish/.cargo-publication.lock/owner
rmdir target/publish/.cargo-publication.lock

# Interrupted dispatch leaves named uncertainty; neither lock nor evidence is retried.
status=0
PUBLISH_INTERRUPT=1 invoke check > interrupted.log 2>&1 || status=$?
[[ "$status" == 143 ]] || exit 1
core="$(sed -n 's/^Publication evidence: //p' interrupted.log)"
[[ -s "$core/dispatch" && ! -e "$core/status" && -f target/publish/.cargo-publication.lock/owner ]] || exit 1
refuse publish
[[ -f target/publish/.cargo-publication.lock/owner && -s "$core/dispatch" ]] || exit 1
rm target/publish/.cargo-publication.lock/owner
rmdir target/publish/.cargo-publication.lock
mkdir target/publish/lock
refuse publish
[[ -d target/publish/lock ]] || exit 1
rmdir target/publish/lock
printf 'consumer artifact\n' > expected-retained
cmp expected-retained target/retained
echo 'Publication command checks passed (Git and upload effects substituted).'
fixture_complete=true
