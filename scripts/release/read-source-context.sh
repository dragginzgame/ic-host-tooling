#!/usr/bin/env bash
set -euo pipefail

# Preserve source-bound release admission from Shared PR #269, revision
# 4eabe7f771907ccebfebceafffe10657b911c0eb; released Shared 0.24.0 omits it.
# Host owns this adapter until an equivalent canonical helper is available.
# Read selected member identities from one local Git commit for a Cargo renderer.
# Requires Git, jq and prepared Mike Farah yq. No Cargo, fetch or Git mutation.
# The caller admits the root manifest JSON and selects explicit member directories,
# package eligibility, version policy and the renderer; this is not TOML editing.
[[ $# -ge 4 ]] || {
    echo 'usage: read-source-context.sh REPOSITORY COMMIT MANIFEST_JSON MEMBER_DIRECTORY...' >&2
    exit 2
}
repository="$1" source="$2" manifest="$3"
shift 3
fail() { echo "Cargo source context refused: $1" >&2; exit 1; }
case "$repository" in ''|*$'\r'*|*$'\n'*) fail 'invalid repository directory' ;; esac
repository="$(CDPATH='' cd -P -- "$repository" && printf '%s/.' "$PWD")"
repository="${repository%/.}"
case "$repository" in *$'\r'*|*$'\n'*) fail 'invalid resolved repository directory' ;; esac
[[ "$source" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || fail 'expected a full commit identity'
observed="$(git -C "$repository" rev-parse --verify "$source^{commit}")"
[[ "$observed" == "$source" ]] || fail 'source is not the selected commit'
[[ -f "$manifest" && ! -L "$manifest" ]] || fail 'expected regular manifest JSON'
jq -es 'length == 1 and (.[0] | type == "object")' "$manifest" > /dev/null
for directory in "$@"; do
    # Explicit paths only: never expand globs or resolve against today's members.
    jq -en --arg path "$directory" '
        $path | length > 0 and (startswith("/") | not)
        and (test("[\u0000\r\n*?\\[\\]\\\\]") | not)
        and (split("/") | all(. != "" and . != "." and . != ".."))
    ' > /dev/null || fail 'expected an explicit relative member directory'
done
temporary="$(mktemp -d "${TMPDIR:-/tmp}/cargo-source-context.XXXXXX")"
trap 'rm -rf -- "$temporary"' EXIT
: > "$temporary/members.jsonl"
for directory in "$@"; do
    git -C "$repository" show "$source:$directory/Cargo.toml" > "$temporary/member.toml"
    "${YQ:-yq}" -p toml -o json '.' "$temporary/member.toml" |
        jq -ec --arg path "/release-source/$directory/Cargo.toml" '
            .package.name | if type == "string" and test("^[A-Za-z0-9][A-Za-z0-9_-]*$")
            then {name: ., manifest_path: $path} else error("invalid source member identity") end
        ' >> "$temporary/members.jsonl"
done
# Emit nothing until every member observation and the final identity check passes.
jq -es --slurpfile manifest "$manifest" '
    if (map(.name) | unique | length) == length
    then {source_root: "/release-source", manifest: $manifest[0], members: .}
    else error("duplicate source member identity") end
' "$temporary/members.jsonl"
