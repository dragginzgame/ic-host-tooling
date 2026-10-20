#!/usr/bin/env bash
set -euo pipefail

operation="${1:-}"
case "$operation" in publish|publish-check) ;; *) echo 'usage: publish.sh publish|publish-check' >&2; exit 2 ;; esac
[[ $# -eq 1 ]] || exit 2
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
cd "$root"
bash scripts/ci/check-release-source.sh
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
tag="refs/tags/v$version"
[[ "$(git cat-file -t "$tag")" == tag ]] || {
    echo "publication requires the annotated release tag v$version" >&2; exit 1;
}
[[ "$(git rev-parse "$tag^{commit}")" == "$(git rev-parse HEAD)" ]] || {
    echo "publication requires HEAD at release v$version" >&2; exit 1;
}
# Cargo also enforces the package's registry eligibility and credential selection.
make --no-print-directory -C "$root" clippy
arguments=(publish -p ic-build --locked --registry crates-io)
[[ "$operation" != publish-check ]] || arguments+=(--dry-run)
exec cargo "${arguments[@]}"
