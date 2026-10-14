#!/usr/bin/env bash
set -euo pipefail

# Consumer-owned Cargo adapter. Requires Bash 3.2, Git, Make, awk, the selected
# local cargo-set-version, prepared jq/Mike Farah yq and the provisioned Rust
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
fail() { echo "release metadata refused: $1" >&2; exit 1; }
version() {
    local observed
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
    receipt_header | cmp - "$receipt"
}
archive_validation_evidence() {
    local file archive=""
    # Admission precedes any copy or log append. The release runner owns the
    # enclosing release lock; standalone callers must exclude concurrent writers.
    for file in "$receipt" "$notes" "$receipt.log"; do
        if [[ -e "$file" || -L "$file" ]]; then
            [[ -f "$file" && ! -L "$file" ]] || fail 'validation evidence is not a regular file'
        fi
    done
    for file in "$receipt" "$notes" "$receipt.log"; do
        [[ -e "$file" ]] || continue
        if [[ -z "$archive" ]]; then
            archive="$(mktemp -d "$state/$RELEASE_VERSION.validation-attempt.XXXXXX")" ||
                fail 'cannot create validation evidence archive'
        fi
        cp -p "$file" "$archive/${file##*/}" || fail 'cannot preserve validation evidence'
        cmp "$file" "$archive/${file##*/}" || fail 'validation evidence copy differs'
    done
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
    git show "$RELEASE_SOURCE:Cargo.toml" |
        awk -v mode=manifest -v packages="$packages" -v replacement="$RELEASE_VERSION" -f "$tooling_root/scripts/release/metadata-version.awk" |
        cmp - "$directory/Cargo.toml"
    perl "$tooling_root/scripts/ci/rewrite-local-lock-versions.pl" \
        "$scratch/Cargo.lock" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "${selected[@]}" |
        cmp - "$directory/Cargo.lock"
    finalize_notes "$notes" | cmp - "$directory/CHANGELOG.md"
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
case "$operation" in
    version) version; exit ;;
    preflight|verify|prepare|check|commit-check|committed-check) selections ;;
    *) echo 'usage: adapter.sh version|preflight|verify|prepare|check|commit-check|committed-check' >&2; exit 2 ;;
esac
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
        awk -v mode=manifest -v packages="$packages" -v replacement="$RELEASE_PREVIOUS" \
            -f "$tooling_root/scripts/release/metadata-version.awk" Cargo.toml |
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
        if [[ "$operation" == preflight ]]; then
            make --no-print-directory install-tools
        fi
        CARGO_NET_OFFLINE=true make --no-print-directory tools-check
        CARGO_NET_OFFLINE=true make --no-print-directory dependency-pins-check
        bash "$tooling_root/scripts/release/tools.sh" run --help > /dev/null
        if [[ "$operation" == preflight ]]; then
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
            archive_validation_evidence
            {
                receipt_header
                CARGO_NET_OFFLINE=true make --no-print-directory ci
                CARGO_NET_OFFLINE=true make --no-print-directory msrv-workspace
            } 2>&1 | tee -a "$receipt.log"
            observed_source="$(git rev-parse HEAD)" || exit $?
            after="$(git diff --binary HEAD | git hash-object --stdin)" || exit $?
            [[ "$observed_source" == "$RELEASE_SOURCE" && "$after" == "$before" ]] || fail "source changed during validation"
            admit_files
            cmp "$scratch/notes" CHANGELOG.md
            receipt_header > "$scratch/receipt"
            mv "$scratch/notes" "$notes"
            mv "$scratch/receipt" "$receipt"
        fi
        ;;
    prepare)
        observed_source="$(git rev-parse HEAD)" || exit $?
        [[ "$observed_source" == "$RELEASE_SOURCE" ]] || fail "preparation source changed"
        observed_version="$(version)" || exit $?
        [[ "$observed_version" == "$RELEASE_PREVIOUS" ]] || fail "preparation workspace version changed"
        check_receipt
        cmp "$notes" CHANGELOG.md
        scratch="$(mktemp -d "$state/$RELEASE_VERSION.prepare.XXXXXX")"
        trap 'rm -rf "$scratch"' EXIT
        cp -p CHANGELOG.md "$scratch/notes"
        finalize_notes "$notes" > "$scratch/notes"
        bash "$tooling_root/scripts/release/tools.sh" run --workspace --offline "$RELEASE_VERSION"
        mv "$scratch/notes" CHANGELOG.md
        check_prepared
        ;;
    check) check_prepared ;;
    committed-check) check_committed ;;
    commit-check)
        check_prepared
        # The common runner fixes the index tree; also reject unstaged edits and
        # staging outside the explicit file set before it creates the commit.
        git diff --quiet -- || fail "unstaged release changes"
        git diff --cached --exit-code "$RELEASE_SOURCE" -- . ':!Cargo.toml' ':!Cargo.lock' ':!CHANGELOG.md' > /dev/null || fail "unrelated staged files"
        ;;
esac
