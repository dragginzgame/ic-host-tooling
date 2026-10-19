#!/usr/bin/env bash
set -euo pipefail

# Consumer tool selection before Shared's metadata adapter starts. Checks never
# install tools; explicit setup and standard release preflight select prepare.
[[ $# == 1 && ( "$1" == check || "$1" == prepare ) ]] || exit 2
operation="$1"
if [[ -n "${CARGO_RELEASE_METADATA_EDITOR+x}" ]]; then
    editor="$CARGO_RELEASE_METADATA_EDITOR"
    [[ "$editor" == /* && -f "$editor" && -x "$editor" &&
       "$editor" != *$'\n'* && "$editor" != *$'\r'* ]] || {
        echo 'CARGO_RELEASE_METADATA_EDITOR must select an absolute executable file' >&2
        exit 2
    }
    printf '%s\n' "$editor"
    exit
fi
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"; root="${root%/.}"
selection=(--package ic-build --version 0.19.0 --bin ic-build-release-metadata --profile debug)
[[ "$operation" != check ]] || selection+=(--check)
if bash "$root/scripts/dev/install-rust-tools.sh" "${selection[@]}"; then
    exit
else
    status=$?
    echo 'Prepare the pinned release editor with make install-release-tools, then retry.' >&2
    exit "$status"
fi
