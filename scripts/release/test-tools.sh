#!/usr/bin/env bash
set -euo pipefail

# Exercise Host's adapter contract with a substitute shared installer. The
# canonical installer fixtures own receipts, storage admission and retention.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/host-release-tools.XXXXXX")"
completed=false
finish() {
    local status=$?
    # Bash 3.2 may enter EXIT with status zero after a nounset error.
    [[ "$completed" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else echo "Release tool fixture retained: $fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
fixture="$(cd -P "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"
export TMPDIR="${fixture%/*}"
mkdir -p "$fixture/tooling/scripts/release" "$fixture/tooling/scripts/dev" \
    "$fixture/tooling/ci" "$fixture/bin" "$fixture/consumer with spaces" "$fixture/selected tools"
cp "$root/scripts/release/tools.sh" "$fixture/tooling/scripts/release/"
cp "$root/ci/release-tools.env" "$fixture/tooling/ci/"
export RELEASE_TOOL_TEST_ROOT="$fixture"
# shellcheck source=/dev/null
source "$root/ci/release-tools.env"
export HOST_CARGO_EDIT_VERSION
cat > "$fixture/tooling/scripts/dev/install-rust-tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 10 || $# == 11 ]] || exit 1
[[ "$1" == --consumer && "$2" == "$PWD" && "$3" == --package && "$4" == cargo-edit &&
   "$5" == --version && "$6" == "$HOST_CARGO_EDIT_VERSION" && "$7" == --bin &&
   "$8" == cargo-set-version && "$9" == --profile && "${10}" == release ]] || exit 1
selected="$RELEASE_TOOL_TEST_ROOT/selected tools/cargo-set-version"
if [[ $# == 11 ]]; then
    [[ "${11}" == --check && -x "$selected" ]] || exit 1
    [[ "${RELEASE_TOOL_TEST_REFUSE:-0}" == 0 ]] || exit "$RELEASE_TOOL_TEST_REFUSE"
else
    [[ "${RELEASE_TOOL_TEST_FAIL:-0}" == 0 ]] || exit "$RELEASE_TOOL_TEST_FAIL"
    if [[ ! -e "$selected" ]]; then
        printf '%s\n' install >> "$RELEASE_TOOL_TEST_ROOT/installs"
        cp "$RELEASE_TOOL_TEST_ROOT/tool" "$selected"
        chmod +x "$selected"
    fi
fi
printf '%s\n' "$selected"
STUB
cat > "$fixture/tool" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == 'set-version --version' ]]; then
    printf 'cargo-edit-set-version %s\n' "${RELEASE_TOOL_TEST_VERSION:-$HOST_CARGO_EDIT_VERSION}"
else
    printf '%s\0' "$@" > "$RELEASE_TOOL_TEST_ROOT/arguments"
    exit "${RELEASE_TOOL_TEST_RUN_STATUS:-0}"
fi
TOOL
cat > "$fixture/bin/cargo-set-version" <<'STUB'
#!/usr/bin/env bash
echo forbidden-global-fallback >> "$RELEASE_TOOL_TEST_ROOT/global"
exit 91
STUB
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$root/.tools/host/bin:$PATH"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
cd -P "$fixture/consumer with spaces"
selector="$fixture/tooling/scripts/release/tools.sh"
if bash "$selector" check > "$fixture/missing.log" 2>&1; then exit 1; fi
[[ ! -e "$fixture/installs" && ! -e "$fixture/global" ]] || exit 1
selected="$(bash "$selector" install)"
[[ -x "$selected" && "$selected" == "$fixture/selected tools/cargo-set-version" ]] || exit 1
[[ "$(bash "$selector" check)" == "$selected" ]] || exit 1
[[ "$(bash "$selector" install)" == "$selected" ]] || exit 1
[[ "$(wc -l < "$fixture/installs")" -eq 1 ]] || exit 1
bash "$selector" run --workspace --offline 'literal argument'
printf '%s\0' set-version --workspace --offline 'literal argument' > "$fixture/expected"
cmp "$fixture/expected" "$fixture/arguments"
status=0
RELEASE_TOOL_TEST_RUN_STATUS=37 bash "$selector" run --help || status=$?
[[ "$status" == 37 ]] || exit 1
if RELEASE_TOOL_TEST_VERSION=0.0.0 bash "$selector" check > "$fixture/version.log" 2>&1; then exit 1; fi
cp "$fixture/arguments" "$fixture/before-refusal"
status=0
RELEASE_TOOL_TEST_REFUSE=29 bash "$selector" run --help > "$fixture/refused.log" 2>&1 || status=$?
[[ "$status" == 29 ]] || exit 1
cmp "$fixture/before-refusal" "$fixture/arguments"
[[ ! -e "$fixture/global" && "$(wc -l < "$fixture/installs")" -eq 1 ]] || exit 1
mkdir "$fixture/failed-consumer"
cd "$fixture/failed-consumer"
status=0
RELEASE_TOOL_TEST_FAIL=73 bash "$selector" install > "$fixture/install-failure.log" 2>&1 || status=$?
[[ "$status" == 73 ]] || exit 1
[[ ! -e "$fixture/global" && "$(wc -l < "$fixture/installs")" -eq 1 ]] || exit 1
echo 'Host release-tool arguments, returned path, reuse and failure propagation passed (substitute shared installer)'
completed=true
