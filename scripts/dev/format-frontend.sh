#!/usr/bin/env bash
set -euo pipefail

# Run a prepared Prettier against index-snapshot inputs, or tracked inputs when
# invoked explicitly outside the hook. Product scope/configuration stays local.
if [[ $# != 2 || ( "$1" != --write && "$1" != --check ) ]]; then
    echo 'usage: format-frontend.sh --write|--check <frontend-directory>' >&2
    exit 2
fi
mode="$1"
scope="${2%/}"
case "$scope" in
    ''|/*|.|..|../*|*/../*|*/..|./*|*/./*|*/.|*//*) echo 'frontend scope must be a relative directory' >&2; exit 2 ;;
esac
list="$(mktemp "${TMPDIR:-/tmp}/frontend-format.XXXXXX")"
trap 'rm -f "$list"' EXIT
if [[ "$mode" == --write && -n "${SHARED_TOOLING_FORMAT_FILES:-}" ]]; then
    cp "$SHARED_TOOLING_FORMAT_FILES" "$list"
else
    [[ -d "$scope" ]] || { echo 'frontend scope directory does not exist' >&2; exit 2; }
    git --literal-pathspecs ls-files -z -- "$scope/" > "$list"
fi
cursor="$scope"
while [[ "$cursor" != . ]]; do
    [[ ! -L "$cursor" ]] || { echo 'frontend scope may not traverse symlinks' >&2; exit 1; }
    cursor="$(dirname "$cursor")"
done
files=()
count=0
while IFS= read -r -d '' path; do
    [[ "$path" == "$scope/"* ]] || continue
    case "$path" in
        ../*|*/../*|*/..|*//*) echo 'invalid formatter path' >&2; exit 2 ;;
        *.js|*.jsx|*.ts|*.tsx|*.json|*.css|*.scss|*.html|*.md|*.yaml|*.yml) ;;
        *) continue ;;
    esac
    cursor="$path"
    while [[ "$cursor" != . ]]; do
        [[ ! -L "$cursor" ]] || { echo 'formatter inputs may not traverse symlinks' >&2; exit 1; }
        cursor="$(dirname "$cursor")"
    done
    [[ -f "$path" ]] || continue
    files[count]="$path"
    count=$((count + 1))
done < "$list"
[[ "$count" -gt 0 ]] || exit 0
case "${PRETTIER_BIN:-}" in
    /*) [[ -x "$PRETTIER_BIN" ]] || { echo 'prepared Prettier executable is missing' >&2; exit 1; } ;;
    *) echo 'PRETTIER_BIN must select an explicitly prepared absolute executable path' >&2; exit 1 ;;
esac
if ! actual_version="$("$PRETTIER_BIN" --version)" || \
    [[ -z "${PRETTIER_VERSION:-}" || "$actual_version" != "$PRETTIER_VERSION" ]]; then
    echo 'prepared Prettier does not match the selected lockfile version' >&2
    exit 1
fi
"$PRETTIER_BIN" "$mode" -- "${files[@]}"
