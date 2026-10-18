#!/usr/bin/env bash
# Shared companions: scripts/ci/run-cargo-publication.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/publication-attempt.XXXXXX")"
fixture="$(cd -P "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"
fixture_complete=false
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
trap 'fixture_finish "$?" "$fixture" "Publication attempt fixture retained"' EXIT
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
mkdir "$fixture/bin" "$fixture/evidence" "$fixture/workspace"
export PUBLICATION_FIXTURE="$fixture"
export REAL_TEE
REAL_TEE="$(command -v tee)"
cat > "$fixture/bin/git" <<'STUB'
#!/usr/bin/env bash
set -eu
case "$1" in
    rev-parse)
        if [[ "$2" == --show-toplevel ]]; then printf '%s\n' "$PUBLICATION_FIXTURE/workspace";
        else printf '%040d\n' 1; fi ;;
    status)
        if [[ "${PUBLICATION_RACE:-0}" == 1 && -d "$PUBLICATION_FIXTURE/evidence/.cargo-publication.lock" ]]; then echo ' M Cargo.toml';
        elif [[ "${PUBLICATION_DIRTY:-0}" == 1 ]]; then echo ' M README.md'; fi ;;
    *) exit 99 ;;
esac
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
printf '%s\0' "$@" > "$PUBLICATION_FIXTURE/cargo-arguments"
echo cargo-output
if [[ "${PUBLICATION_SIGNAL:-0}" == 1 ]]; then kill -TERM "$PPID"; fi
exit "${PUBLICATION_CARGO_STATUS:-0}"
STUB
cat > "$fixture/bin/tee" <<'STUB'
#!/usr/bin/env bash
"$REAL_TEE" "$@"
exit "${PUBLICATION_LOG_STATUS:-0}"
STUB
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$PATH"
cd "$fixture/workspace"
source_id="$(printf '%040d' 1)"
runner="$ROOT/scripts/ci/run-cargo-publication.sh"
run() { bash "$runner" "$1" "$fixture/evidence/$2" "$source_id" crates-io "${@:3}"; }
refuse() {
    local expected="$1" status=0
    shift
    "$@" > "$fixture/refusal.log" 2>&1 || status=$?
    [[ "$status" == "$expected" ]] || { cat "$fixture/refusal.log"; echo "expected $expected, got $status" >&2; exit 1; }
}
run check dry --package library@1.2.3 --package cli@2.0.0 --all-features --allow-dirty > /dev/null
printf '%s\0' publish --locked --registry crates-io --package library@1.2.3 --package cli@2.0.0 --all-features --allow-dirty --dry-run > "$fixture/expected"
cmp "$fixture/expected" "$fixture/cargo-arguments"
printf '%s\0' cargo > "$fixture/expected-command"
cat "$fixture/expected" >> "$fixture/expected-command"
cmp "$fixture/expected-command" "$fixture/evidence/dry/arguments"
[[ "$(cat "$fixture/evidence/dry/status")" == 0 && ! -e "$fixture/evidence/.cargo-publication.lock" ]] || exit 1
cp "$fixture/evidence/dry/intent" "$fixture/saved-intent"
rm "$fixture/cargo-arguments"
refuse 2 run check dry --package library@1.2.3
cmp "$fixture/saved-intent" "$fixture/evidence/dry/intent"
[[ ! -e "$fixture/cargo-arguments" ]] || exit 1
for options in '--workspace' '--dry-run' '--config' '--token'; do
    refuse 2 run check invalid --package library@1.2.3 "$options"
done
refuse 2 run publish invalid --package library@1.2.3 --allow-dirty
refuse 2 run check invalid --package library
refuse 2 run check invalid --package library@1.2.3 --package library@1.2.4
refuse 2 bash "$runner" check "$fixture/evidence/invalid" "$(printf '%040d' 2)" crates-io --package library@1.2.3
[[ ! -e "$fixture/evidence/invalid" && ! -e "$fixture/cargo-arguments" ]] || exit 1
printf '[workspace]\n' > "$fixture/outside.toml"
refuse 2 run check invalid --package library@1.2.3 --manifest-path "$fixture/outside.toml"
ln -s "$fixture/outside.toml" Cargo.toml
refuse 2 run check invalid --package library@1.2.3 --manifest-path Cargo.toml
rm Cargo.toml
printf '[workspace]\n' > Cargo.toml
run check manifest --package library@1.2.3 --manifest-path Cargo.toml --target-dir "$fixture/workspace" > /dev/null
export PUBLICATION_DIRTY=1
refuse 2 run publish dirty --package library@1.2.3
run check dirty --package library@1.2.3 --allow-dirty > /dev/null
unset PUBLICATION_DIRTY
for pair in '43 0' '0 17' '43 17'; do
    read -r PUBLICATION_CARGO_STATUS PUBLICATION_LOG_STATUS <<< "$pair"
    export PUBLICATION_CARGO_STATUS PUBLICATION_LOG_STATUS
    expected="$PUBLICATION_CARGO_STATUS"; [[ "$expected" != 0 ]] || expected="$PUBLICATION_LOG_STATUS"
    name="status-$PUBLICATION_CARGO_STATUS-$PUBLICATION_LOG_STATUS"
    refuse "$expected" run publish "$name" --package library@1.2.3 --no-verify
    [[ "$(cat "$fixture/evidence/$name/cargo-status")" == "$PUBLICATION_CARGO_STATUS" ]] || exit 1
    [[ "$(cat "$fixture/evidence/$name/log-status")" == "$PUBLICATION_LOG_STATUS" ]] || exit 1
    [[ "$(cat "$fixture/evidence/$name/status")" == "$expected" ]] || exit 1
    [[ -s "$fixture/evidence/$name/cargo.log" && ! -e "$fixture/evidence/.cargo-publication.lock" ]] || exit 1
done
unset PUBLICATION_CARGO_STATUS PUBLICATION_LOG_STATUS
rm "$fixture/cargo-arguments"
mkdir "$fixture/evidence/.cargo-publication.lock"
refuse 1 run check occupied --package library@1.2.3
[[ ! -e "$fixture/evidence/occupied" && ! -e "$fixture/cargo-arguments" ]] || exit 1
rmdir "$fixture/evidence/.cargo-publication.lock"
export PUBLICATION_RACE=1
refuse 2 run publish raced --package library@1.2.3
[[ -s "$fixture/evidence/raced/intent" && ! -e "$fixture/evidence/raced/dispatch" && ! -e "$fixture/cargo-arguments" ]] || exit 1
[[ -e "$fixture/evidence/.cargo-publication.lock/owner" ]] || exit 1
rm "$fixture/evidence/.cargo-publication.lock/owner"
rmdir "$fixture/evidence/.cargo-publication.lock"
unset PUBLICATION_RACE
for separator in $'\n' $'\r'; do
    bad="$fixture/bad${separator}path"
    mkdir "$bad"
    ln -s "$bad" "$fixture/alias"
    refuse 2 bash "$runner" check "$bad/../normalized" "$source_id" crates-io --package library@1.2.3
    refuse 2 bash "$runner" check "$fixture/alias/attempt" "$source_id" crates-io --package library@1.2.3
    rm "$fixture/alias"
done
export PUBLICATION_SIGNAL=1
refuse 143 run publish interrupted --package library@1.2.3
[[ -s "$fixture/evidence/interrupted/dispatch" && ! -e "$fixture/evidence/interrupted/status" && -e "$fixture/evidence/.cargo-publication.lock/owner" ]] || exit 1
unset PUBLICATION_SIGNAL
fixture_complete=true
echo 'Cargo publication intent, selection, status, interruption and no-replay fixtures passed'
