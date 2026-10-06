#!/usr/bin/env bash
set -euo pipefail

# Consumer-owned Cargo adapter. Requires Bash 3.2, Git, Make, awk, cargo-edit
# (cargo set-version) and the provisioned Rust toolchains/dependency cache.
# Git effects belong to the shared runner, never to this adapter.
operation="${1:-}"
[[ $# -eq 1 ]] || exit 2
# Helper code belongs to this adapter; Cargo data belongs to the selected checkout.
tooling_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
fail() { echo "release metadata refused: $1" >&2; exit 1; }
version() {
    awk -v mode=manifest -v read_version=1 -f "$tooling_root/scripts/release/metadata-version.awk" Cargo.toml
}
admit_files() {
    local path paths rejection=""
    paths="$(mktemp "${TMPDIR:-/tmp}/ic-host-release-paths.XXXXXX")"
    # Inventory both sides independently: restoring a working file to HEAD
    # must not conceal a different staged payload.
    git diff --name-only -z --cached HEAD -- > "$paths"
    git diff --name-only -z -- >> "$paths"
    git ls-files --others --exclude-standard -z >> "$paths"
    while IFS= read -r -d '' path; do
        case "$path" in
            CHANGELOG.md) ;;
            Cargo.toml|Cargo.lock) [[ "$operation" != preflight && "$operation" != verify ]] || {
                rejection="commit Cargo metadata before releasing"; break;
            } ;;
            *) rejection="uncommitted non-release path: $path"; break ;;
        esac
    done < "$paths"
    rm -f "$paths"
    [[ -z "$rejection" ]] || fail "$rejection"
}
selections() {
    [[ "${RELEASE_SOURCE:?}" =~ ^[0-9a-f]{40,64}$ ]] || fail "invalid source identity"
    [[ "${RELEASE_DATE:?}" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || fail "invalid release date"
    [[ "$(bash "$tooling_root/scripts/ci/next-release-version.sh" "${RELEASE_PREVIOUS:?}" "${RELEASE_KIND:?}")" == "${RELEASE_VERSION:?}" ]] || fail "conflicting release selection"
    state="$(git rev-parse --git-path release-state)"
    [[ ! -L "$state" ]] || fail "symlinked release evidence directory"
    receipt="$state/$RELEASE_VERSION.validation"
    notes="$receipt.notes"
}
finalize_notes() {
    local input="$1" heading
    # Retain the consumer's top-entry identity boundary. The shared helper owns
    # candidate selection, complete-ledger conflict checks and heading rewriting.
    heading="$(awk '/^## / { print; exit }' "$input")"
    case "$heading" in
        '## [Draft]'|"## [$RELEASE_VERSION]"|"## [$RELEASE_PREVIOUS]"|"## [$RELEASE_PREVIOUS] - "*) ;;
        *) fail "top changelog entry conflicts with selected release" ;;
    esac
    # A candidate belongs at the top; do not let the shared selector relocate
    # a current release entry from history during preparation or recovery.
    awk -v heading="## [$RELEASE_VERSION]" '
        /^## / { if (++headings > 1 && ($0 == "## [Draft]" || $0 == heading || index($0, heading " - ") == 1)) exit 1 }
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
    local mode file
    check_receipt
    [[ "$(version)" == "$RELEASE_VERSION" ]] || fail "workspace version differs from candidate"
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
    [[ "$(git rev-parse "$RELEASE_COMMIT^{commit}")" == "$RELEASE_COMMIT" ]] || fail "release commit is unavailable"
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
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" && "$(version)" == "$RELEASE_PREVIOUS" ]] || fail "source/version changed"
        for file in Cargo.toml Cargo.lock CHANGELOG.md; do
            [[ -f "$file" && ! -L "$file" ]] || fail "missing or symlinked release metadata"
        done
        [[ "$(awk -v mode=lock -v read_version=1 -f "$tooling_root/scripts/release/metadata-version.awk" Cargo.lock)" == "$RELEASE_PREVIOUS" ]] || fail "lockfile version differs"
        admit_files
        # Git releases and registry publication are separate effects. Validate
        # the workspace without requiring its packages to permit publication.
        cargo metadata --no-deps --format-version 1 --locked --offline > /dev/null ||
            fail "locked workspace metadata validation failed"
        finalize_notes CHANGELOG.md > /dev/null
        # Library qualification needs only the pinned host parsers. The IC
        # executable bundle remains explicit setup; verification stays offline.
        if [[ "$operation" == preflight ]]; then
            make --no-print-directory install-host-tools
        fi
        CARGO_NET_OFFLINE=true make --no-print-directory host-tools-check
        CARGO_NET_OFFLINE=true make --no-print-directory dependency-pins-check
        cargo set-version --help > /dev/null
        cargo fetch --locked --offline
        if [[ "$operation" == verify ]]; then
            mkdir -p "$state"
            scratch="$(mktemp -d "$state/$RELEASE_VERSION.validation.XXXXXX")"
            trap 'rm -rf "$scratch"' EXIT
            cp -p CHANGELOG.md "$scratch/notes"
            before="$(git diff --binary HEAD | git hash-object --stdin)"
            {
                receipt_header
                CARGO_NET_OFFLINE=true make --no-print-directory ci
                for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do
                    CARGO_NET_OFFLINE=true make --no-print-directory msrv PACKAGE="$package"
                done
            } 2>&1 | tee -a "$receipt.log"
            [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" && "$(git diff --binary HEAD | git hash-object --stdin)" == "$before" ]] || fail "source changed during validation"
            admit_files
            cmp "$scratch/notes" CHANGELOG.md
            receipt_header > "$scratch/receipt"
            mv "$scratch/notes" "$notes"
            mv "$scratch/receipt" "$receipt"
        fi
        ;;
    prepare)
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" && "$(version)" == "$RELEASE_PREVIOUS" ]] || fail "preparation source/version changed"
        check_receipt
        cmp "$notes" CHANGELOG.md
        scratch="$(mktemp -d "$state/$RELEASE_VERSION.prepare.XXXXXX")"
        trap 'rm -rf "$scratch"' EXIT
        cp -p CHANGELOG.md "$scratch/notes"
        finalize_notes "$notes" > "$scratch/notes"
        cargo set-version --workspace --offline "$RELEASE_VERSION"
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
