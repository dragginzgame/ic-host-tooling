#!/usr/bin/env bash
# Shared companions: scripts/ci/collect-tool-evidence.sh scripts/ci/formatting-evidence.sh
set -euo pipefail

# Testkit owns CI roots/retention; Shared owns tool selection and archive orchestration.
# Requires Bash 3.2, tar/gzip and the CI workspace/temp/outcome environment.
[[ $# -eq 1 ]] || { echo 'expected checks or portable evidence selection' >&2; exit 2; }
mode="$1"
case "$mode" in checks|portable) ;; *) echo 'unknown evidence selection' >&2; exit 2 ;; esac
workspace="${GITHUB_WORKSPACE:?}"
[[ "$workspace" == /* ]] || workspace="$PWD/$workspace"
workspace="$(cd -P "$workspace" && printf '%s/.' "$PWD")"
workspace="${workspace%/.}"
temporary="${RUNNER_TEMP:?}"
[[ "$temporary" == /* ]] || temporary="$PWD/$temporary"
temporary="$(cd -P "$temporary" && printf '%s/.' "$PWD")"
temporary="${temporary%/.}"
archiver="$0"
[[ "$archiver" == /* ]] || archiver="$PWD/$archiver"
helpers="${archiver%/*}"
outcomes="${IC_TESTKIT_FAILURE_STEPS:?}"
bundle="$temporary/ic-testkit-$mode-failure"
# A previous attempt is evidence, not an output to overwrite.
mkdir "$bundle"
printf '%s\n' "$outcomes" > "$bundle/step-outcomes.json"
arguments=("$bundle" step-outcomes.json)
names=(ic-testkit-tools-install.log)
if [[ "$mode" == checks ]]; then
    names+=(ic-testkit-validation.log)
else
    names+=(ic-testkit-portable-fixtures ic-testkit-archive-proof ic-testkit-archive-downloaded)
fi
for name in "${names[@]}"; do
    if [[ -e "$temporary/$name" || -L "$temporary/$name" ]]; then
        arguments+=("$temporary" "$name")
    fi
done
# Shared formatter failures retain exact logs under the workflow temp root.
# shellcheck source=scripts/ci/formatting-evidence.sh
source "$helpers/formatting-evidence.sh"
select_formatting_evidence "$temporary"
arguments+=(${formatting_evidence[@]+"${formatting_evidence[@]}"})
# Retain a redirected parent itself and unrelated logs, without traversing it.
collection_status=0
for parent in target .tools; do
    if [[ -L "$workspace/$parent" ]]; then
        printf 'linked evidence parent: %s\n' "$parent" >> "$bundle/collection.log"
        printf 'linked evidence parent: %s\n' "$parent" >&2
        arguments+=("$workspace" "$parent")
        collection_status=1
    fi
done
if [[ "$collection_status" != 0 ]]; then arguments+=("$bundle" collection.log); fi
if [[ ! -L "$workspace/target" ]] &&
    [[ -e "$workspace/target/validation-failures" || -L "$workspace/target/validation-failures" ]]; then
    arguments+=("$workspace" target/validation-failures)
fi
diagnostics="$bundle/tool-evidence"
mkdir "$diagnostics"
# Shared owns selection receipts, failed-selector admission and archive status.
# Keep Testkit's original outcomes and linked-parent diagnostic as explicit inputs.
tool_status=0
TAR_OPTIONS='' bash "$helpers/collect-tool-evidence.sh" \
    "$bundle/evidence.tar.gz.partial" "$workspace" "$diagnostics" compact \
    "$workspace/ci/tool-versions.env" "$workspace/ci/ic-tools.tsv" \
    "${arguments[@]}" >/dev/null || tool_status=$?
[[ "$tool_status" != 2 ]] || exit "$tool_status"
mv "$bundle/evidence.tar.gz.partial" "$bundle/evidence.tar.gz"
printf '%s\n' "$bundle/evidence.tar.gz"
# CI publishes the archive path before propagating incomplete collection.
[[ "$tool_status" == 0 ]] || exit "$tool_status"
exit "$collection_status"
