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
    local input="$1" heading
    # Retain the consumer's top-entry identity boundary. The shared helper owns
    # candidate selection, complete-ledger conflict checks and heading rewriting.
    heading="$(awk '/^##[ \t]+/ { sub(/[ \t]+$/, ""); gsub(/[ \t]+/, " "); print; exit }' "$input")"
    case "$heading" in
        "## [$RELEASE_VERSION]"|"## [$RELEASE_PREVIOUS]"|"## [$RELEASE_PREVIOUS] - "*) ;;
        *) fail "top changelog entry conflicts with selected release" ;;
    esac
    # A candidate belongs at the top; do not let the shared selector relocate
    # a current release entry from history during preparation or recovery.
    awk -v heading="## [$RELEASE_VERSION]" '
        /^##[ \t]+/ {
            sub(/[ \t]+$/, "")
            gsub(/[ \t]+/, " ")
            if ($0 == "## [Draft]") exit 1
            if (++headings > 1 && ($0 == heading || index($0, heading " - ") == 1)) exit 1
        }
    ' "$input" || fail "duplicate or misplaced current release entry"
    awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" \
        -v date="$RELEASE_DATE" \
        -f "$tooling_root/scripts/ci/finalize-release-changelog.awk" "$input"
}
receipt_header() {
    printf '%s\n' release-validation-1 "$RELEASE_SOURCE" "$RELEASE_KIND" \
        "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "$RELEASE_DATE"
}
check_receipt() {
    [[ -f "$receipt" && ! -L "$receipt" && -f "$notes" && ! -L "$notes" ]] || fail "source-bound validation evidence is missing"
    receipt_header | cmp - "$receipt"
}
check_prepared() {
    local mode file observed_version
    check_receipt
    observed_version="$(version)" || exit $?
    [[ "$observed_version" == "$RELEASE_VERSION" ]] || fail "workspace version differs from candidate"
    for mode in manifest lock; do
        if [[ "$mode" == manifest ]]; then file=Cargo.toml; else file=Cargo.lock; fi
        [[ -f "$file" && ! -L "$file" ]] || fail "missing or symlinked Cargo metadata"
        git show "$RELEASE_SOURCE:$file" |
            awk -v mode="$mode" -v replacement="$RELEASE_VERSION" -f "$tooling_root/scripts/release/metadata-version.awk" |
            cmp - "$file"
    done
    finalize_notes "$notes" | cmp - CHANGELOG.md
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
    for mode in manifest lock; do
        if [[ "$mode" == manifest ]]; then file=Cargo.toml; else file=Cargo.lock; fi
        git show "$RELEASE_COMMIT:$file" > "$scratch/$file"
        git show "$RELEASE_SOURCE:$file" |
            awk -v mode="$mode" -v replacement="$RELEASE_VERSION" -f "$tooling_root/scripts/release/metadata-version.awk" |
            cmp - "$scratch/$file"
    done
    git show "$RELEASE_COMMIT:CHANGELOG.md" > "$scratch/CHANGELOG.md"
    finalize_notes "$notes" | cmp - "$scratch/CHANGELOG.md"
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
        awk -v mode=manifest -v replacement="$RELEASE_PREVIOUS" \
            -f "$tooling_root/scripts/release/metadata-version.awk" Cargo.toml |
            cmp - Cargo.toml || fail "unsupported or unsynchronized release metadata"
        observed_lock_version="$(awk -v mode=lock -v read_version=1 -f "$tooling_root/scripts/release/metadata-version.awk" Cargo.lock)" || exit $?
        [[ "$observed_lock_version" == "$RELEASE_PREVIOUS" ]] || fail "lockfile version differs"
        admit_files
        # Git releases and registry publication are separate effects. Validate
        # the workspace without requiring its packages to permit publication.
        cargo metadata --no-deps --format-version 1 --locked --offline > /dev/null ||
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
            {
                receipt_header
                CARGO_NET_OFFLINE=true make --no-print-directory ci
                for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
                    CARGO_NET_OFFLINE=true make --no-print-directory msrv PACKAGE="$package"
                done
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
