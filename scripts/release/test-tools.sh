#!/usr/bin/env bash
set -euo pipefail

# Exercise the real Host selector and shared installer, substituting Cargo's
# installation effect. Receipts, checksums, reuse and refusal are real.
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
mkdir -p "$fixture/tooling/scripts/release" "$fixture/tooling/scripts/dev" \
    "$fixture/tooling/scripts/ci" "$fixture/tooling/ci" "$fixture/bin" "$fixture/consumer with spaces"
cp "$root/scripts/release/tools.sh" "$fixture/tooling/scripts/release/"
cp "$root/scripts/dev/install-rust-tools.sh" "$fixture/tooling/scripts/dev/"
cp "$root/scripts/ci/verify-file-checksum.sh" "$fixture/tooling/scripts/ci/"
cp "$root/ci/release-tools.env" "$fixture/tooling/ci/"
export RELEASE_TOOL_TEST_ROOT="$fixture"
RELEASE_TOOL_TEST_HOST="$(rustc -vV | sed -n 's/^host: //p')"
export RELEASE_TOOL_TEST_HOST
# shellcheck source=/dev/null
source "$root/ci/release-tools.env"
export HOST_CARGO_EDIT_VERSION
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == install && "$2" == cargo-edit ]]
printf '%s\n' install >> "$RELEASE_TOOL_TEST_ROOT/installs"
[[ "${RELEASE_TOOL_TEST_FAIL:-0}" == 0 ]] || exit "$RELEASE_TOOL_TEST_FAIL"
destination=''; selected=''; target=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --root) destination="$2"; shift ;;
        --version) selected="$2"; shift ;;
        --bin) target="$2"; shift ;;
    esac
    shift
done
[[ "$selected" == "=$HOST_CARGO_EDIT_VERSION" && "$target" == cargo-set-version ]]
mkdir -p "$destination/bin"
cat > "$destination/bin/$target" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == 'set-version --version' ]]; then
    printf 'cargo-edit-set-version %s\n' "${RELEASE_TOOL_TEST_VERSION:-$HOST_CARGO_EDIT_VERSION}"
else
    printf '%s\0' "$@" > "$RELEASE_TOOL_TEST_ROOT/arguments"
    exit "${RELEASE_TOOL_TEST_RUN_STATUS:-0}"
fi
TOOL
chmod +x "$destination/bin/$target"
jq -n --arg identity "cargo-edit $HOST_CARGO_EDIT_VERSION (registry+https://github.com/rust-lang/crates.io-index)" \
    --arg version "$selected" --arg host "$RELEASE_TOOL_TEST_HOST" \
    '{installs:{($identity):{version_req:$version,bins:["cargo-set-version"],profile:"release",target:$host,rustc:"fixture compiler"}}}' \
    > "$destination/.crates2.json"
STUB
cat > "$fixture/bin/cargo-set-version" <<'STUB'
#!/usr/bin/env bash
echo forbidden-global-fallback >> "$RELEASE_TOOL_TEST_ROOT/global"
exit 91
STUB
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$root/.tools/host/bin:$PATH"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
cd "$fixture/consumer with spaces"
selector="$fixture/tooling/scripts/release/tools.sh"
if bash "$selector" check > "$fixture/missing.log" 2>&1; then exit 1; fi
[[ ! -e "$fixture/installs" && ! -e "$fixture/global" ]]
selected="$(bash "$selector" install)"
[[ -x "$selected" && "$selected" == "$PWD/"* ]]
[[ "$(bash "$selector" check)" == "$selected" ]]
[[ "$(bash "$selector" install)" == "$selected" ]]
[[ "$(wc -l < "$fixture/installs")" == 1 ]]
bash "$selector" run --workspace --offline 'literal argument'
printf '%s\0' set-version --workspace --offline 'literal argument' > "$fixture/expected"
cmp "$fixture/expected" "$fixture/arguments"
status=0
RELEASE_TOOL_TEST_RUN_STATUS=37 bash "$selector" run --help || status=$?
[[ "$status" == 37 ]]
if RELEASE_TOOL_TEST_VERSION=0.0.0 bash "$selector" check > "$fixture/version.log" 2>&1; then exit 1; fi
cp -p "$selected" "$fixture/original-tool"
cp "$fixture/arguments" "$fixture/before-refusal"
printf '\n# changed\n' >> "$selected"
if bash "$selector" run --help > "$fixture/changed.log" 2>&1; then exit 1; fi
cmp "$fixture/before-refusal" "$fixture/arguments"
[[ ! -e "$fixture/global" && "$(wc -l < "$fixture/installs")" == 1 ]]
cp -p "$fixture/original-tool" "$selected"
mkdir "$fixture/failed-consumer"
cd "$fixture/failed-consumer"
status=0
RELEASE_TOOL_TEST_FAIL=73 bash "$selector" install > "$fixture/install-failure.log" 2>&1 || status=$?
[[ "$status" == 73 ]]
shopt -s nullglob
retained=(.tools/rust/build/cargo-attempt.*/install.log)
[[ ${#retained[@]} == 1 && ! -e "$fixture/global" ]]
echo 'Host release-tool selection, reuse, offline refusal and retained failures passed (substitute Cargo install)'
completed=true
