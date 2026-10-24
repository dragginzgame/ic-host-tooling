#!/usr/bin/env bash
# Shared companions: ci/release-delivery.txt ci/release-editor.env scripts/release/editor.sh scripts/ci/check-release-delivery.sh scripts/ci/read-cargo-workspace-version.sh scripts/ci/check-release-source.sh scripts/ci/next-release-version.sh scripts/ci/finalize-release-changelog.awk scripts/ci/read-cargo-source-context.sh scripts/ci/release-receipts.sh scripts/ci/release-metadata-transaction.sh
set -euo pipefail

# Consumer-owned Cargo adapter. Requires Bash 3.2, Git, Make, awk, the selected
# literal metadata renderer, prepared jq/Mike Farah yq and the provisioned Rust
# toolchains. Release preflight prepares the selected locked dependency cache;
# verification uses only offline inputs.
# Git effects belong to the shared runner, never to this adapter.
operation="${1:-}"
[[ $# -eq 1 ]] || exit 2
# Helper code belongs to this adapter; Cargo data belongs to the selected checkout.
tooling_root="${BASH_SOURCE[0]}"
[[ "$tooling_root" == /* ]] || tooling_root="$PWD/$tooling_root"
tooling_root="$(cd -P "${tooling_root%/*}/../.." && printf '%s/.' "$PWD")"
tooling_root="${tooling_root%/.}"
bash "$tooling_root/scripts/ci/check-release-delivery.sh" \
    "$tooling_root/ci/release-delivery.txt" "${RELEASE_DELIVERY-direct}"
fail() { echo "release metadata refused: $1" >&2; exit 1; }
select_editor() {
    # shellcheck source=ci/release-editor.env
    source "$tooling_root/ci/release-editor.env"
    metadata_editor="$(bash "$tooling_root/scripts/release/editor.sh" check)" || return
    [[ "$metadata_editor" == /* && -f "$metadata_editor" && -x "$metadata_editor" ]] ||
        fail 'release editor must be an admitted absolute executable'
}
version() {
    local observed
    if ! command -v "${YQ:-yq}" >/dev/null 2>&1 || ! command -v jq >/dev/null 2>&1; then
        fail 'host parsers are not prepared; run make install-tools && make tools-check before releasing'
    fi
    observed="$(bash "$tooling_root/scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" || return
    printf '%s\n' "$observed"
}
local_packages() {
    "${YQ:-yq}" -p toml -o json '.' | jq -er --arg version "${RELEASE_PREVIOUS:?}" '
        [.package[] | select(.source == null)] as $local |
        ($local | map(.name)) as $names |
        if ($names | length) > 0 and ($names | unique | length) == ($names | length)
           and all($names[]; test("^[A-Za-z0-9][A-Za-z0-9_-]*$"))
           and all($local[]; .version == $version)
        then $names | sort | join(" ") else error("invalid local package identities") end'
}
project_manifest() (
    temporary="$(mktemp -d "${TMPDIR:-/tmp}/host-release-manifest.XXXXXX")" || return
    trap 'rm -rf "$temporary"' EXIT
    "${YQ:-yq}" -p toml -o json '.' "$1" > "$temporary/manifest.json" || return
    # Host selects explicit members and requires agreement with its local lock set.
    # The source adapter prepares identities from the selected Git source.
    jq -ej '.workspace.members |
        if type == "array" and length > 0 and all(.[];
            type == "string" and (contains("\u0000") | not))
        then .[] + "\u0000" else error("expected explicit workspace member paths") end' \
        "$temporary/manifest.json" > "$temporary/directories" || return
    members=()
    while IFS= read -r -d '' directory; do
        members+=("$directory")
    done < "$temporary/directories"
    bash "$tooling_root/scripts/ci/read-cargo-source-context.sh" \
        . "$RELEASE_SOURCE" "$temporary/manifest.json" "${members[@]}" \
        > "$temporary/context.json" || return
    jq -e --arg packages "$3" '.members | (map(.name) | sort) == ($packages | split(" "))' \
        "$temporary/context.json" > /dev/null || return
    "$metadata_editor" --max-input-bytes "$HOST_RELEASE_EDITOR_MAX_BYTES" manifest \
        "$1" "$RELEASE_PREVIOUS" "$2" "$temporary/context.json"
)
admit_files() {
    local allowed=(--allow CHANGELOG.md)
    if [[ "$operation" != preflight && "$operation" != verify ]]; then
        allowed+=(--allow Cargo.toml --allow Cargo.lock)
    fi
    bash "$tooling_root/scripts/ci/check-release-source.sh" "${allowed[@]}" ||
        fail 'commit refused source paths before releasing'
}
selections() {
    local selected_version
    [[ "${RELEASE_SOURCE:?}" =~ ^[0-9a-f]{40,64}$ ]] || fail "invalid source identity"
    [[ "${RELEASE_DATE:?}" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || fail "invalid release date"
    selected_version="$(bash "$tooling_root/scripts/ci/next-release-version.sh" "${RELEASE_PREVIOUS:?}" "${RELEASE_KIND:?}")" || exit $?
    [[ "$selected_version" == "${RELEASE_VERSION:?}" ]] || fail "conflicting release selection"
    state="$(git rev-parse --git-path release-state)"
    [[ ! -L "$state" ]] || fail "symlinked release evidence directory"
    receipt="$state/$RELEASE_VERSION.validation"
    notes="$receipt.notes"
}
finalize_notes() {
    awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" \
        -v date="$RELEASE_DATE" \
        -f "$tooling_root/scripts/ci/finalize-release-changelog.awk" "$1"
}
receipt_header() {
    printf '%s\n' release-validation-1 "$RELEASE_SOURCE" "$RELEASE_KIND" \
        "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "$RELEASE_DATE"
}
check_receipt() {
    [[ -f "$receipt" && ! -L "$receipt" && -f "$notes" && ! -L "$notes" ]] || fail "source-bound validation evidence is missing"
    release_receipt_check "$receipt" receipt_header
}
check_payload() (
    local directory="$1" file packages scratch
    local selected=()
    scratch="$(mktemp -d "$state/$RELEASE_VERSION.payload.XXXXXX")" || return
    trap 'rm -rf "$scratch"' EXIT
    git show "$RELEASE_SOURCE:Cargo.lock" > "$scratch/Cargo.lock" || return
    packages="$(local_packages < "$scratch/Cargo.lock")" || return
    read -r -a selected <<< "$packages"
    for file in Cargo.toml Cargo.lock; do
        [[ -f "$directory/$file" && ! -L "$directory/$file" ]] || fail "missing or symlinked Cargo metadata"
    done
    git show "$RELEASE_SOURCE:Cargo.toml" > "$scratch/Cargo.toml" || return
    project_manifest "$scratch/Cargo.toml" "$RELEASE_VERSION" "$packages" |
        cmp - "$directory/Cargo.toml"
    "$metadata_editor" --max-input-bytes "$HOST_RELEASE_EDITOR_MAX_BYTES" lock \
        "$scratch/Cargo.lock" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "${selected[@]}" |
        cmp - "$directory/Cargo.lock"
    finalize_notes "$notes" | cmp - "$directory/CHANGELOG.md"
    # Only the root manifest changes during release preparation. Check those
    # exact candidate/saved bytes; member formatting was qualified at the source.
    cargo sort --grouped --check "$directory"
)
check_prepared() {
    local observed_version
    check_receipt
    observed_version="$(version)" || exit $?
    [[ "$observed_version" == "$RELEASE_VERSION" ]] || fail "workspace version differs from candidate"
    check_payload .
    cargo metadata --no-deps --format-version 1 --locked --offline > /dev/null
    admit_files
}
check_committed() (
    # Recovery can select an older release beneath newer committed fixes.
    # Its evidence binds the selected payload, never the current working files.
    check_receipt
    [[ "${RELEASE_COMMIT:-}" =~ ^[0-9a-f]{40,64}$ ]] || fail "invalid release commit identity"
    observed_commit="$(git rev-parse "$RELEASE_COMMIT^{commit}")" || exit $?
    [[ "$observed_commit" == "$RELEASE_COMMIT" ]] || fail "release commit is unavailable"
    scratch="$(mktemp -d "$state/$RELEASE_VERSION.committed.XXXXXX")"
    trap 'rm -rf "$scratch"' EXIT
    for file in Cargo.toml Cargo.lock CHANGELOG.md; do
        git show "$RELEASE_COMMIT:$file" > "$scratch/$file"
    done
    check_payload "$scratch"
)
render_prepared_metadata() {
    local original="$1" candidate="$2" packages
    local selected=()
    packages="$(local_packages < "$original/Cargo.lock")" || return
    read -r -a selected <<< "$packages"
    project_manifest "$original/Cargo.toml" "$RELEASE_VERSION" "$packages" > "$candidate/Cargo.toml" || return
    "$metadata_editor" --max-input-bytes "$HOST_RELEASE_EDITOR_MAX_BYTES" lock \
        "$original/Cargo.lock" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "${selected[@]}" > "$candidate/Cargo.lock" || return
    finalize_notes "$notes" > "$candidate/CHANGELOG.md" || return
    "${YQ:-yq}" -p toml -o json '.' "$candidate/Cargo.toml" > /dev/null
}
validate_prepared_metadata() {
    # A separate strict shell preserves every nested observation failure when
    # the transaction invokes its callback in a conditional status context.
    bash "$tooling_root/scripts/release/adapter.sh" check
}
case "$operation" in
    version) version; exit ;;
    preflight|verify|prepare|check|commit-check|committed-check) selections ;;
    *) echo 'usage: adapter.sh version|preflight|verify|prepare|check|commit-check|committed-check' >&2; exit 2 ;;
esac
# shellcheck source=scripts/ci/release-receipts.sh
source "$tooling_root/scripts/ci/release-receipts.sh"
# shellcheck source=scripts/ci/release-metadata-transaction.sh
source "$tooling_root/scripts/ci/release-metadata-transaction.sh"
case "$operation" in
    preflight|verify)
        observed_source="$(git rev-parse HEAD)" || exit $?
        [[ "$observed_source" == "$RELEASE_SOURCE" ]] || fail "source changed"
        observed_version="$(version)" || exit $?
        [[ "$observed_version" == "$RELEASE_PREVIOUS" ]] || fail "workspace version changed"
        for file in Cargo.toml Cargo.lock CHANGELOG.md; do
            [[ -f "$file" && ! -L "$file" ]] || fail "missing or symlinked release metadata"
        done
        # TOML reading belongs to the shared helper. Keep this consumer's exact
        # version-only metadata layout and synchronized path requirements here.
        packages="$(local_packages < Cargo.lock)" || exit $?
        admit_files
        # The runner replays preflight while reconciling saved preparation.
        # Only a fresh attempt may set up tools or prepare the dependency cache.
        fresh_tool_setup=false
        if [[ "$operation" == preflight && ! -e "$state/$RELEASE_VERSION.plan" && ! -L "$state/$RELEASE_VERSION.plan" ]]; then
            fresh_tool_setup=true
        fi
        editor_setup=false
        if [[ "$fresh_tool_setup" == true ]]; then
            editor_status=0
            select_editor || editor_status=$?
            # Busy admission is not missing setup; never install over its owner.
            [[ "$editor_status" != 75 ]] || exit "$editor_status"
            if [[ "$editor_status" != 0 ]]; then
                # A first release needs the renderer before layout comparison.
                # Source admission precedes setup; already prepared tools allow
                # unsupported metadata to refuse before any setup dispatch.
                make --no-print-directory install-tools
                editor_setup=true
                select_editor
            fi
        else
            select_editor
        fi
        project_manifest Cargo.toml "$RELEASE_PREVIOUS" "$packages" |
            cmp - Cargo.toml || fail "unsupported or unsynchronized release metadata"
        admit_files
        # Git releases and registry publication are separate effects. Validate
        # the workspace without requiring its packages to permit publication.
        cargo metadata --no-deps --format-version 1 --locked --offline |
            jq -e --arg packages "$packages" '. as $m |
                [.packages[] | select(.id as $id | $m.workspace_members | index($id)) | .name] |
                sort == ($packages | split(" "))' > /dev/null ||
            fail "locked workspace metadata validation failed"
        finalize_notes CHANGELOG.md > /dev/null
        # Selected releases prepare the complete common toolset in shared order.
        # Standalone verification checks the prepared set without installation.
        if [[ "$fresh_tool_setup" == true && "$editor_setup" == false ]]; then
            make --no-print-directory install-tools
        fi
        CARGO_NET_OFFLINE=true make --no-print-directory tools-check
        # Match the global ordering used by make fmt and fmt-check. Inline role
        # labels need no grouped-mode override and survive this read-only check.
        cargo sort --workspace --grouped --check
        CARGO_NET_OFFLINE=true make --no-print-directory dependency-pins-check
        if [[ "$fresh_tool_setup" == true ]]; then
            # The selected release prepares its existing graph before validation.
            # Cargo still honours explicit offline environment/configuration.
            cargo fetch --locked
        else
            cargo fetch --locked --offline
        fi
        if [[ "$operation" == verify ]]; then
            mkdir -p "$state"
            scratch="$(mktemp -d "$state/$RELEASE_VERSION.validation.XXXXXX")"
            trap 'rm -rf "$scratch"' EXIT
            cp -p CHANGELOG.md "$scratch/notes"
            before="$(git diff --binary HEAD | git hash-object --stdin)" || exit $?
            release_receipt_archive "$state/$RELEASE_VERSION.validation-attempt.XXXXXX" \
                "$receipt" "$notes" "$receipt.log"
            {
                receipt_header
                CARGO_NET_OFFLINE=true make --no-print-directory ci
                CARGO_NET_OFFLINE=true make --no-print-directory msrv-qualification
            } 2>&1 | tee -a "$receipt.log"
            observed_source="$(git rev-parse HEAD)" || exit $?
            after="$(git diff --binary HEAD | git hash-object --stdin)" || exit $?
            [[ "$observed_source" == "$RELEASE_SOURCE" && "$after" == "$before" ]] || fail "source changed during validation"
            admit_files
            cmp "$scratch/notes" CHANGELOG.md
            release_receipt_write "$notes" cat "$scratch/notes"
            release_receipt_write "$receipt" receipt_header
        fi
        ;;
    prepare)
        observed_source="$(git rev-parse HEAD)" || exit $?
        [[ "$observed_source" == "$RELEASE_SOURCE" ]] || fail "preparation source changed"
        observed_version="$(version)" || exit $?
        [[ "$observed_version" == "$RELEASE_PREVIOUS" ]] || fail "preparation workspace version changed"
        check_receipt
        cmp "$notes" CHANGELOG.md
        select_editor
        release_metadata_prepare "$state/$RELEASE_VERSION.prepare.XXXXXX" \
            render_prepared_metadata validate_prepared_metadata Cargo.toml Cargo.lock CHANGELOG.md
        ;;
    check) select_editor; check_prepared ;;
    committed-check) select_editor; check_committed ;;
    commit-check)
        select_editor
        check_prepared
        # The common runner fixes the index tree; also reject unstaged edits and
        # staging outside the explicit file set before it creates the commit.
        git diff --quiet -- || fail "unstaged release changes"
        git diff --cached --exit-code "$RELEASE_SOURCE" -- . ':!Cargo.toml' ':!Cargo.lock' ':!CHANGELOG.md' > /dev/null || fail "unrelated staged files"
        ;;
esac
