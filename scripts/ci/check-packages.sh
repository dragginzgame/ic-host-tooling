#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.."

[[ $# == 1 ]] || exit 2
case "$1" in
  package)
    cargo run -p ic-testkit-dev --bin check_packages --locked --offline
    ;;
  publish-dry-run)
    cargo publish -p ic-testkit-candid -p ic-testkit --locked --dry-run --allow-dirty
    ;;
  *) echo 'expected package or publish-dry-run' >&2; exit 2 ;;
esac
