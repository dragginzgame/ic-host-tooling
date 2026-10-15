#!/usr/bin/env bash
set -euo pipefail

# Cargo workspace publication admission. Requires prepared Cargo 1.99, Git,
# jq and tar; Shared Tooling owns attempt recording and Cargo/tee dispatch.
# Cargo owns packaging, dependency ordering, verification, upload
# and index waiting. The caller owns credentials and partial-upload recovery.
mode="${1:-}"
[[ $# == 1 && ( "$mode" == publish || "$mode" == check ) ]] || {
    echo 'usage: workspace.sh publish|check' >&2; exit 2;
}
fail() { echo "publication refused: $1" >&2; exit 1; }
tooling_root="${BASH_SOURCE[0]}"
[[ "$tooling_root" == /* ]] || tooling_root="$PWD/$tooling_root"
tooling_root="$(cd -P "${tooling_root%/*}/../.." && printf '%s/.' "$PWD")"
tooling_root="${tooling_root%/.}"
# Publication eligibility is explicit policy, distinct from workspace membership.
packages=(ic-host-artifacts ic-host-fs ic-host-process ic-host-text ic-host-tools ic-host-wasm-inspect)
package_path() {
    case "$1" in
        ic-host-wasm-inspect) printf 'apps/%s\n' "$1" ;;
        *) printf 'crates/%s\n' "$1" ;;
    esac
}
# Preserve path newlines, removing only Git's terminating record newline.
root="$(git rev-parse --show-toplevel && printf '/.')"
root="${root%$'\n/.'}"
case "$root" in *$'\r'*|*$'\n'*) fail 'directory paths must not contain CR/LF' ;; esac
cd -P -- "$root"
root="$PWD"
case "$root" in *$'\r'*|*$'\n'*) fail 'directory paths must not contain CR/LF' ;; esac
source="$(git rev-parse --verify HEAD)" || exit $?
[[ "$source" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || fail 'invalid source identity'
dirty="$(git status --porcelain --untracked-files=all)" || exit $?
[[ "$mode" != publish || -z "$dirty" ]] || fail 'commit workspace changes before publishing'
[[ -f LICENSE && ! -L LICENSE ]] || fail 'expected a regular root license notice'
for package in "${packages[@]}"; do
    notice="$(package_path "$package")/LICENSE"
    [[ -f "$notice" && ! -L "$notice" ]] || fail "expected a regular license notice for $package"
    cmp -s LICENSE "$notice" || fail "license notice differs from the root notice for $package"
done
[[ ! -L target && ! -L target/publish && ! -L target/publish/admission ]] || fail 'symlinked publication evidence directory'
umask 077
mkdir -p target/publish/admission
[[ ! -e target/publish/.cargo-publication.lock && ! -L target/publish/.cargo-publication.lock ]] ||
    fail 'Cargo attempt lock is occupied; reconcile retained evidence before removing a stale lock'
lock=target/publish/lock
mkdir "$lock" 2>/dev/null || fail 'publication lock is occupied; inspect its owner before removing a stale lock'
trap 'rm -f "$lock/owner"; rmdir "$lock"' EXIT
printf '%s\n' "$$" > "$lock/owner"
attempt="$(mktemp -d "target/publish/admission/$mode.XXXXXX")"
publication_attempt="$root/target/publish/${attempt##*/}"
printf '%s\n' "$publication_attempt" > "$attempt/publication-attempt"
echo "Publication admission evidence: $root/$attempt"
printf '%s\n' "$dirty" > "$attempt/working-status"
cargo metadata --manifest-path "$root/Cargo.toml" --no-deps --format-version 1 \
    --locked --offline > "$attempt/metadata.json" 2> "$attempt/metadata.log" ||
    fail 'locked workspace metadata validation failed'
jq -e '
    . as $workspace
    | [.packages[] | select(.id as $id | $workspace.workspace_members | index($id))] as $packages
    | ($packages | map(.name) | sort) == ($ARGS.positional | sort)
      and ($packages | map(.version) | unique | length) == 1
      and ($packages | all(.publish == ["crates-io"]))
      and ($packages | all(.repository == "https://github.com/dragginzgame/ic-host-tooling"))
      and ($packages | any(.name == "ic-host-wasm-inspect" and
          any(.targets[]; .name == "ic-host-wasm-inspect" and (.kind | index("bin")))))
' "$attempt/metadata.json" --args "${packages[@]}" > /dev/null || fail 'expected selected synchronized crates.io packages'
repository="$(jq -r '.packages[0].repository' "$attempt/metadata.json")"
version="$(jq -r '.packages[0].version' "$attempt/metadata.json")"
[[ "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || fail 'expected a stable workspace version'
# Persist Host's admitted package/provenance selection before package preparation.
# Shared Tooling records the exact Cargo command in its separate fresh attempt.
printf '%s\n' publication-1 "$mode" "$source" "$version" crates-io \
    "${packages[@]}" > "$attempt/intent"
if [[ "$mode" == publish ]]; then
    # Inspect Cargo's actual package provenance before upload. Package preparation
    # resolves dependencies but does not upload or build; publish still owns
    # compilation, dependency order and all registry effects.
    package_arguments=(package --manifest-path "$root/Cargo.toml" --workspace \
        --registry crates-io --all-features --locked --no-verify --target-dir "$root/target")
    printf '%s\0' "${package_arguments[@]}" > "$attempt/package-arguments"
    cargo "${package_arguments[@]}" > "$attempt/package.log" 2>&1 || fail 'package preparation failed; no upload attempted'
    for package in "${packages[@]}"; do
        archive="target/package/$package-$version.crate"
        [[ -f "$archive" && ! -L "$archive" ]] || fail "missing regular package archive for $package"
        tar -xOf "$archive" "$package-$version/.cargo_vcs_info.json" \
            > "$attempt/$package-vcs.json" 2> "$attempt/$package-vcs.log" || fail "missing package provenance for $package"
        jq -e --arg source "$source" --arg path "$(package_path "$package")" \
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
observed_source="$(git rev-parse --verify HEAD)" || exit $?
observed_dirty="$(git status --porcelain --untracked-files=all)" || exit $?
[[ "$observed_source" == "$source" && "$observed_dirty" == "$dirty" ]] || fail 'source changed before publication'
arguments=()
for package in "${packages[@]}"; do arguments+=(--package "$package@$version"); done
arguments+=(--manifest-path "$root/Cargo.toml" --target-dir "$root/target" --all-features)
[[ "$mode" != check ]] || arguments+=(--allow-dirty)
bash "$tooling_root/scripts/ci/run-cargo-publication.sh" "$mode" "$publication_attempt" \
    "$source" crates-io "${arguments[@]}"
