#!/usr/bin/env bash
set -euo pipefail

# Thin Cargo workspace publication adapter. Requires prepared Cargo 1.99, Git,
# jq, tar and tee. Cargo owns packaging, dependency ordering, verification, upload
# and index waiting. The caller owns credentials and partial-upload recovery.
mode="${1:-}"
[[ $# == 1 && ( "$mode" == publish || "$mode" == check ) ]] || {
    echo 'usage: workspace.sh publish|check' >&2; exit 2;
}
fail() { echo "publication refused: $1" >&2; exit 1; }
# Preserve path newlines, removing only Git's terminating record newline.
root="$(git rev-parse --show-toplevel && printf '/.')"
root="${root%$'\n/.'}"
cd "$root"
source="$(git rev-parse --verify HEAD)"
[[ "$source" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || fail 'invalid source identity'
dirty="$(git status --porcelain --untracked-files=all)"
[[ "$mode" != publish || -z "$dirty" ]] || fail 'commit workspace changes before publishing'
[[ -f LICENSE && ! -L LICENSE ]] || fail 'expected a regular root license notice'
for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
    notice="crates/$package/LICENSE"
    [[ -f "$notice" && ! -L "$notice" ]] || fail "expected a regular license notice for $package"
    cmp -s LICENSE "$notice" || fail "license notice differs from the root notice for $package"
done
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
      and ($packages | all(.repository == "https://github.com/dragginzgame/ic-host-tooling"))
' "$attempt/metadata.json" > /dev/null || fail 'expected four synchronized crates.io packages'
repository="$(jq -r '.packages[0].repository' "$attempt/metadata.json")"
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
if [[ "$mode" == publish ]]; then
    # Inspect Cargo's actual package provenance before upload. Package preparation
    # resolves dependencies but does not upload or build; publish still owns
    # compilation, dependency order and all registry effects.
    package_arguments=(package --manifest-path "$root/Cargo.toml" --workspace \
        --registry crates-io --all-features --locked --no-verify --target-dir "$root/target")
    printf '%s\0' "${package_arguments[@]}" > "$attempt/package-arguments"
    cargo "${package_arguments[@]}" > "$attempt/package.log" 2>&1 || fail 'package preparation failed; no upload attempted'
    for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
        archive="target/package/$package-$version.crate"
        [[ -f "$archive" && ! -L "$archive" ]] || fail "missing regular package archive for $package"
        tar -xOf "$archive" "$package-$version/.cargo_vcs_info.json" \
            > "$attempt/$package-vcs.json" 2> "$attempt/$package-vcs.log" || fail "missing package provenance for $package"
        jq -e --arg source "$source" --arg path "crates/$package" \
            '.git.sha1 == $source and (.git.dirty // false) == false and .path_in_vcs == $path' \
            "$attempt/$package-vcs.json" > /dev/null || fail "package source identity differs for $package"
    done
    # An empty object store establishes remote retrieval rather than merely
    # finding an object already present in this checkout. Retain it as evidence.
    printf '%s\n' "$repository" "$source" > "$attempt/source-request"
    git init --bare "$attempt/source.git" > "$attempt/source-init.log" 2>&1 || fail 'cannot prepare source evidence'
    GIT_TERMINAL_PROMPT=0 git -C "$attempt/source.git" fetch --no-tags --depth=1 \
        "$repository" "$source" > "$attempt/source-fetch.log" 2>&1 || fail 'packaged source is not retrievable from the declared repository'
    retrieved="$(git -C "$attempt/source.git" rev-parse --verify 'FETCH_HEAD^{commit}')" || fail 'retrieved source is not a commit'
    [[ "$retrieved" == "$source" ]] || fail 'retrieved source differs from package provenance'
    printf '%s\n' "$retrieved" > "$attempt/source-verified"
fi
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
