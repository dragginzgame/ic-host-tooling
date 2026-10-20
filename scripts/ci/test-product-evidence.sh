#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/product-evidence.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Product evidence fixtures retained"' EXIT
repository="$fixture/repository"
temporary="$fixture/runner"
mkdir -p "$repository" "$temporary"
bash "$root/scripts/ci/archive-product-evidence.sh" "$repository" "$temporary" "$fixture/empty.tar.gz"
[[ ! -e "$fixture/empty.tar.gz" ]] || exit 1
mkdir -p "$temporary/wasm-qualification/optimizer" \
    "$repository/target/package-qualification-123-0/target/debug" \
    "$repository/target/publish/admission/check.partial" \
    "$repository/target/debug"
printf 'failed optimizer\n' > "$temporary/wasm-qualification/optimizer/qualification.log"
printf 'wasm bytes\n' > "$temporary/wasm-qualification/optimizer/output.wasm"
printf 'fetch failed\n' > "$temporary/wasm-qualification/dependencies-fetch.log"
printf 'failed package\n' > "$repository/target/package-qualification-123-0/test-targets.stderr"
printf 'retained hidden input\n' > "$repository/target/package-qualification-123-0/.input"
printf 'cache\n' > "$repository/target/package-qualification-123-0/target/debug/cache"
printf 'unrelated cache\n' > "$repository/target/debug/cache"
printf 'partial admission\n' > "$repository/target/publish/admission/check.partial/metadata.log"
bash "$root/scripts/ci/archive-product-evidence.sh" "$repository" "$temporary" "$fixture/evidence.tar.gz"
mkdir "$fixture/extracted"
tar -xzf "$fixture/evidence.tar.gz" -C "$fixture/extracted"
for path in wasm-qualification/optimizer/qualification.log \
    wasm-qualification/optimizer/output.wasm wasm-qualification/dependencies-fetch.log; do
    cmp "$temporary/$path" "$fixture/extracted/$path"
done
for path in target/package-qualification-123-0/test-targets.stderr \
    target/package-qualification-123-0/.input target/publish/admission/check.partial/metadata.log; do
    cmp "$repository/$path" "$fixture/extracted/$path"
done
[[ ! -e "$fixture/extracted/target/package-qualification-123-0/target" &&
   ! -e "$fixture/extracted/target/debug" ]] || exit 1
# Shared archive admission refuses redirected ancestors, retaining original inputs.
mv "$repository/target" "$fixture/retained-target"
ln -s "$fixture/retained-target" "$repository/target"
if bash "$root/scripts/ci/archive-product-evidence.sh" "$repository" "$temporary" \
    "$fixture/redirected.tar.gz" > "$fixture/redirected.log" 2>&1; then exit 1; fi
[[ -f "$fixture/retained-target/package-qualification-123-0/test-targets.stderr" ]] || exit 1
echo 'Product evidence round-trip, partial attempts and cache exclusions passed'
fixture_complete=true
