#!/usr/bin/env bash
set -euo pipefail

# Copy Host workspace metadata and empty lib/bin entrypoints into an owned test
# directory. Callers keep publication policy, perturbations and effect stubs.
[[ $# == 2 && "$1" == /* && -d "$2" && ! -L "$2" ]] || {
    echo 'usage: prepare-workspace-fixture.sh ABSOLUTE_SOURCE OWNED_DIRECTORY' >&2
    exit 2
}
source_root="$1" destination="$2"
cp "$source_root/Cargo.toml" "$source_root/Cargo.lock" \
    "$source_root/README.md" "$source_root/LICENSE" "$destination/"
cargo metadata --manifest-path "$source_root/Cargo.toml" --no-deps \
    --format-version 1 --locked --offline > "$destination/source-metadata.json"
jq -r '. as $m | .packages[] |
    select(.id as $id | $m.workspace_members | index($id)) |
    .manifest_path, (.targets[] | select(.kind | index("lib") or index("bin") or index("rlib") or index("cdylib")) | .src_path)' \
    "$destination/source-metadata.json" |
    while IFS= read -r path; do
        case "$path" in
            "$source_root"/*) relative="${path#"$source_root"/}" ;;
            *) echo 'fixture input is outside the selected workspace' >&2; exit 1 ;;
        esac
        mkdir -p "$destination/${relative%/*}"
        if [[ "${relative##*/}" == Cargo.toml ]]; then
            cp "$path" "$destination/$relative"
            notice="${path%/*}/LICENSE"
            [[ -f "$notice" && ! -L "$notice" ]] || {
                echo 'fixture requires the real regular package license notice' >&2; exit 1;
            }
            cp "$notice" "$destination/${relative%/*}/LICENSE"
        else
            : > "$destination/$relative"
        fi
    done
