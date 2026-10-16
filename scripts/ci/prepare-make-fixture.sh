#!/usr/bin/env bash
# Shared companions: make/consumer.mk
set -euo pipefail
# Copy only the common Make test skeleton. The caller owns its Makefile,
# catalogs, substituted leaf commands, assertions and disposable destination.
[[ $# == 2 && -d "$1" && -d "$2" ]] || {
    echo 'usage: prepare-make-fixture.sh SOURCE EXISTING-FIXTURE' >&2; exit 2;
}
source_path="$1" fixture_path="$2"
[[ "$source_path" == /* ]] || source_path="$PWD/$source_path"
[[ "$fixture_path" == /* ]] || fixture_path="$PWD/$fixture_path"
for directory in "$source_path" "$fixture_path"; do
    case "$directory" in *$'\n'*|*$'\r'*) echo 'fixture directory contains CR/LF' >&2; exit 2 ;; esac
done
source_root="$(cd -P "$source_path" && printf '%s/.' "$PWD")"
source_root="${source_root%/.}"
fixture_root="$(cd -P "$fixture_path" && printf '%s/.' "$PWD")"
fixture_root="${fixture_root%/.}"
for directory in "$source_root" "$fixture_root"; do
    case "$directory" in *$'\n'*|*$'\r'*) echo 'resolved fixture directory contains CR/LF' >&2; exit 2 ;; esac
done
[[ "$source_root" != "$fixture_root" ]] || exit 2
payloads=(make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk
    scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh)
# Admit every input and output before creating directories or copying a file.
for directory in make scripts scripts/ci; do
    [[ ! -L "$fixture_root/$directory" && ( ! -e "$fixture_root/$directory" || -d "$fixture_root/$directory" ) ]] || exit 2
done
for path in "${payloads[@]}"; do
    [[ -f "$source_root/$path" && -r "$source_root/$path" && ! -L "$source_root/$path" && ! -L "$fixture_root/$path" && ( ! -e "$fixture_root/$path" || -f "$fixture_root/$path" ) ]] || exit 2
done
mkdir -p "$fixture_root/make" "$fixture_root/scripts/ci"
for path in "${payloads[@]}"; do
    cp -p "$source_root/$path" "$fixture_root/$path"
done
