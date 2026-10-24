#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
[[ $# -le 1 ]] || exit 2
# An explicit old script permits before/after regression qualification.
policy="${1:-$root/scripts/ci/check-tools-features.sh}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/tooling-layering.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Layering fixture retained"' EXIT
export LAYER_PROBE_REAL_CARGO
LAYER_PROBE_REAL_CARGO="$(command -v cargo)"
mkdir "$fixture/bin"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
selected='' next=false features=false
for argument in "$@"; do
    if [[ "$next" == true ]]; then selected="$argument"; next=false; fi
    case "$argument" in
        -p|--package) next=true ;;
        --features|--all-features) features=true ;;
    esac
done
if [[ "$1" == tree && "$selected" == "$LAYER_PROBE_ROOT" && "$features" == "$LAYER_PROBE_FEATURES" ]]; then
    # Keep the real checker's native target and normal/build edge selection.
    [[ " $* " == *' --edges normal,build '* ]] || exit 91
    : > "$LAYER_PROBE_OBSERVED"
    exec "$LAYER_PROBE_REAL_CARGO" "$@" --manifest-path "$LAYER_PROBE_MANIFEST"
fi
exec "$LAYER_PROBE_REAL_CARGO" "$@"
CARGO
chmod +x "$fixture/bin/cargo"
# Model old published identities with tiny local packages: the policy must match
# canonical names across versions and paths, not just the current workspace ID.
while read -r owner kind forbidden version expected profile; do
    directory="$fixture/$owner-$forbidden-$profile"
    mkdir -p "$directory/src" "$directory/middle/src" "$directory/higher/src" "$directory/clap/src"
    printf '' > "$directory/src/lib.rs"
    printf '' > "$directory/middle/src/lib.rs"
    printf '' > "$directory/higher/src/lib.rs"
    printf '' > "$directory/clap/src/lib.rs"
    cat > "$directory/Cargo.toml" <<TOML
[package]
name = "$owner"
version = "0.0.0"
edition = "2024"
[workspace]
resolver = "3"
[features]
cli = ["dep:clap"]
[$kind]
bridge = { path = "middle" }
[dependencies.clap]
path = "clap"
optional = true
TOML
    cat > "$directory/clap/Cargo.toml" <<'TOML'
[package]
name = "clap"
version = "4.6.7"
edition = "2024"
TOML
    cat > "$directory/middle/Cargo.toml" <<TOML
[package]
name = "bridge"
version = "0.0.0"
edition = "2024"
[dependencies]
renamed = { package = "$forbidden", path = "../higher" }
TOML
    cat > "$directory/higher/Cargo.toml" <<TOML
[package]
name = "$forbidden"
version = "$version"
edition = "2024"
TOML
    "$LAYER_PROBE_REAL_CARGO" generate-lockfile --offline --manifest-path "$directory/Cargo.toml" > "$directory/lock.log" 2>&1
    status=0
    enabled=false
    [[ "$profile" != cli ]] || enabled=true
    PATH="$fixture/bin:$PATH" LAYER_PROBE_ROOT="$owner" LAYER_PROBE_FEATURES="$enabled" \
        LAYER_PROBE_MANIFEST="$directory/Cargo.toml" LAYER_PROBE_OBSERVED="$directory/observed" \
        bash "$policy" > "$directory/policy.log" 2>&1 || status=$?
    [[ "$status" != 0 && -f "$directory/observed" ]] || exit 1
    # Retain the real introducing graph and require the specific policy refusal.
    grep -F "v$version" "$directory/policy.log" > /dev/null
    grep -F "${expected//_/ } $forbidden" "$directory/policy.log" > /dev/null
done <<'CASES'
ic-build dependencies ic-testkit 0.54.1 forbidden_pattern_matches default
ic-build dependencies ic-testkit 0.54.1 forbidden_pattern_matches cli
ic-build dependencies ic-wire 0.6.13 forbidden_pattern_matches default
ic-host-artifacts build-dependencies ic-build 0.12.0 forbidden default
ic-host-artifacts dependencies ic-platform 0.6.14 forbidden default
ic-host-tools dependencies canic 0.1.0 forbidden_pattern_matches default
CASES
echo 'Host/Build layering rejects renamed transitive normal/build higher-layer identities'
fixture_complete=true
