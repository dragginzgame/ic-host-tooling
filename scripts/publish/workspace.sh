#!/usr/bin/env bash
set -euo pipefail

# Thin Cargo workspace publication adapter. Requires prepared Cargo 1.99, Git,
# jq and tee. Cargo owns packaging, dependency ordering, verification, upload
# and index waiting. The caller owns credentials and partial-upload recovery.
mode="${1:-}"
[[ $# == 1 && ( "$mode" == publish || "$mode" == check ) ]] || {
    echo 'usage: workspace.sh publish|check' >&2; exit 2;
}
fail() { echo "publication refused: $1" >&2; exit 1; }
root="$(git rev-parse --show-toplevel)"
cd "$root"
source="$(git rev-parse --verify HEAD)"
[[ "$source" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || fail 'invalid source identity'
dirty="$(git status --porcelain --untracked-files=all)"
[[ "$mode" != publish || -z "$dirty" ]] || fail 'commit workspace changes before publishing'
[[ ! -L target && ! -L target/publish ]] || fail 'symlinked publication evidence directory'
umask 077
mkdir -p target/publish
lock=target/publish/lock
mkdir "$lock" 2>/dev/null || fail 'publication lock is occupied; inspect its owner before removing a stale lock'
trap 'rm -f "$lock/owner"; rmdir "$lock"' EXIT
printf '%s\n' "$$" > "$lock/owner"
attempt="$(mktemp -d "target/publish/$mode.XXXXXX")"
echo "Publication evidence: $root/$attempt"
printf '%s\n' "$dirty" > "$attempt/working-status"
cargo metadata --manifest-path "$root/Cargo.toml" --no-deps --format-version 1 \
    --locked --offline > "$attempt/metadata.json" 2> "$attempt/metadata.log" ||
    fail 'locked workspace metadata validation failed'
jq -e '
    . as $workspace
    | [.packages[] | select(.id as $id | $workspace.workspace_members | index($id))] as $packages
    | ($packages | map(.name) | sort) ==
        ["ic-host-artifacts", "ic-host-fs", "ic-host-process", "ic-host-tools"]
      and ($packages | map(.version) | unique | length) == 1
      and ($packages | all(.publish == ["crates-io"]))
' "$attempt/metadata.json" > /dev/null || fail 'expected four synchronized crates.io packages'
version="$(jq -r '.packages[0].version' "$attempt/metadata.json")"
[[ "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || fail 'expected a stable workspace version'
arguments=(publish --manifest-path "$root/Cargo.toml" --workspace --registry crates-io \
    --all-features --locked --target-dir "$root/target")
if [[ "$mode" == check ]]; then arguments+=(--dry-run --allow-dirty); fi
# Persist the selected source, version, target and exact argv before Cargo can
# upload. Never treat a failed/lost reply as proof that a package is absent.
printf '%s\n' publication-1 "$mode" "$source" "$version" crates-io \
    ic-host-artifacts ic-host-fs ic-host-process ic-host-tools > "$attempt/intent"
printf '%s\0' "${arguments[@]}" > "$attempt/arguments"
[[ "$(git rev-parse --verify HEAD)" == "$source" &&
    "$(git status --porcelain --untracked-files=all)" == "$dirty" ]] || fail 'source changed before publication'
status=0
cargo "${arguments[@]}" 2>&1 | tee "$attempt/cargo.log" || status=$?
printf '%s\n' "$status" > "$attempt/status"
if [[ "$status" != 0 ]]; then
    if [[ "$mode" == check ]]; then
        echo "Cargo dry run failed; no upload occurred. Evidence: $root/$attempt" >&2
    else
        echo "Cargo failed; retain $root/$attempt and reconcile registry state before another upload." >&2
    fi
fi
exit "$status"
