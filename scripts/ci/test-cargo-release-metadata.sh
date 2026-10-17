#!/usr/bin/env bash
# Shared companions: scripts/ci/cargo-release-metadata.sh
set -Eeuo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/cargo-release-metadata-test.XXXXXX")"
complete=false
finish() {
    local status=$?
    [[ "$complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else printf 'failed Cargo metadata fixture retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
REAL_CARGO="$(command -v cargo)"
export REAL_CARGO
REAL_CP="$(command -v cp)"
export REAL_CP
REAL_GIT="$(command -v git)"
export REAL_GIT
mkdir "$fixture/bin"
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${ORDER_EVENTS:-}" ]]; then
    printf '%s\n' "$*" >> "$ORDER_EVENTS"
fi
case "$1" in
    set-version) echo "preparation must not dispatch manifest mutation tools" >&2; exit 91 ;;
    sort)
        [[ "$*" == "sort --workspace --check" ]] || exit 91
        if [[ -n "${ORDER_EVENTS:-}" ]]; then
            printf '%s\n' "$PWD" > "$ORDER_EVENTS.root"
            cat Cargo.toml > "$ORDER_EVENTS.manifest"
        fi
        [[ "${CASE:-}" != sort-failure ]] || exit 24 ;;
    metadata)
        if [[ "${CASE:-}" == prepared-local-failure ]] && grep -q '^version = "0.10.0"$' Cargo.toml; then
            "$REAL_CARGO" "$@"
            exit 25
        fi
        if [[ "${CASE:-}" == interrupt ]] && grep -q 'version = "1.2.4"' Cargo.toml; then
            kill -TERM "$PPID"; exit 1
        fi
        if [[ "${CASE:-}" == graph-failure || "${CASE:-}" == graph-empty-failure ]]; then
            [[ "$CASE" != graph-failure ]] || "$REAL_CARGO" "$@"
            echo 'injected offline metadata failure' >&2
            exit 37
        fi
        if [[ "${CASE:-}" == check-failure || "${CASE:-}" == restore-failure ]] && grep -q 'version = "1.2.4"' Cargo.toml; then exit 25; fi
        exec "$REAL_CARGO" "$@"
        ;;
    *) exec "$REAL_CARGO" "$@" ;;
esac
STUB
chmod +x "$fixture/bin/cargo"
cat > "$fixture/bin/git" <<'GIT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${GIT_FAILURE:-}" == type && "$*" == cat-file\ -t\ * ]]; then
    printf 'commit\n'; exit 43
fi
if [[ "${GIT_FAILURE:-}" == export && "$1" == archive ]]; then
    "$REAL_GIT" "$@"; exit 43
fi
exec "$REAL_GIT" "$@"
GIT
chmod +x "$fixture/bin/git"
cat > "$fixture/bin/cp" <<'CP'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${CASE:-}" == restore-failure && "$1" == -p && "$2" == */cargo-release-backup.*/Cargo.lock ]]; then
    echo 'controlled lockfile restoration failure' >&2
    exit 43
fi
exec "$REAL_CP" "$@"
CP
chmod +x "$fixture/bin/cp"
REAL_PERL="$(command -v perl)"
export REAL_PERL
cat > "$fixture/bin/perl" <<'PERL'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
    */render-cargo-workspace-version.pl)
        if [[ "${CASE:-}" == renderer-failure ]]; then "$REAL_PERL" "$@"; exit 23; fi ;;
esac
exec "$REAL_PERL" "$@"
PERL
chmod +x "$fixture/bin/perl"
export PATH="$fixture/bin:$PATH"
export RELEASE_PREVIOUS=1.2.3 RELEASE_VERSION=1.2.4 RELEASE_DATE=2026-10-13
helper="$ROOT/scripts/ci/cargo-release-metadata.sh"
make_workspace() {
    local root="$1" package
    mkdir -p "$root/primary/src" "$root/secondary/src"
    cat > "$root/Cargo.toml" <<'TOML'
[workspace]
members = ["primary", "secondary"]
resolver = "3"
[workspace.package]
  version = "1.2.3" # Preserve this literal layout.
edition = "2024"
[workspace.dependencies.example]
version = "1.2.3"
TOML
    for package in primary secondary; do
        cat > "$root/$package/Cargo.toml" <<TOML
[package]
name = "$package"
version.workspace = true
edition.workspace = true
TOML
        printf 'pub fn value() -> u64 { 0 }\n' > "$root/$package/src/lib.rs"
    done
    chmod 0640 "$root/Cargo.toml"
    printf '# Changelog\n\n## [1.2.4]\n\n- Prepared change.\n\n## [1.2.3] - 2026-10-12\n\n- Previous.\n' > "$root/CHANGELOG.md"
    (cd "$root" && "$REAL_CARGO" generate-lockfile --offline > /dev/null)
}
mkdir "$fixture/success"
make_workspace "$fixture/success"
(cd "$fixture/success" && bash "$helper" prepare primary > "$fixture/success.log" 2>&1)
(cd "$fixture/success" && bash "$helper" check primary)
(cd "$fixture/success" && ORDER_EVENTS="$fixture/current-order.events" bash "$helper" check primary --manifest-order)
[[ "$(cat "$fixture/current-order.events.root")" == "$fixture/success" ]] || exit 1
cmp "$fixture/success/Cargo.toml" "$fixture/current-order.events.manifest"
for package in primary secondary; do
    jq -e --arg name "$package" '.packages[] | select(.name == $name) | .version == "1.2.4"' \
        < <(cd "$fixture/success" && "$REAL_CARGO" metadata --offline --locked --format-version 1) > /dev/null
done
# Exercise real Cargo resolution across semver boundaries. An owned dependency
# alias/requirement must follow its path target, not an unrelated registry row.
for mode in patch minor major zero-minor zero-major unicode; do
    root="$fixture/owned-$mode"
    [[ "$mode" != unicode ]] || root="$root space-é"
    make_workspace "$root"
    previous=1.2.3 candidate=1.2.4
    case "$mode" in
        minor) candidate=1.3.0 ;;
        major) candidate=2.0.0 ;;
        zero-minor) previous=0.9.6 candidate=0.10.0 ;;
        zero-major) previous=0.9.6 candidate=1.0.0 ;;
        unicode) previous=0.9.6 candidate=0.10.0 ;;
    esac
    primary_dir=primary
    if [[ "$mode" == unicode ]]; then
        primary_dir='primary-é'
        mv "$root/primary" "$root/$primary_dir"
    fi
    # Bare, renamed, quoted and table-form requirements exercise the literal
    # writer independently; exact/caret/tilde operators retain their policy.
    cat > "$root/Cargo.toml" <<TOML
[workspace]
members = ["$primary_dir", "secondary"]
resolver = "3"
[workspace.package]
version = "$previous"
edition = "2024"
[workspace.metadata]
description = '''
[workspace.dependencies]
primary = { version = "DO NOT EDIT" }
'''
[workspace.dependencies]
primary = { version = "${previous%.*}", path = "$primary_dir" } # preserve comment
"alias" = { package = "primary", path = "./$primary_dir", version = '^$previous' }
exact = { version = "=$previous", path = "$primary_dir", package = "primary" }
custom = { version = ">=0.1, <9", path = "$primary_dir", package = "primary" }
unrelated = { version = "${previous%.*}", path = "../unowned-primary", package = "primary" }
external = { version = "$previous" }
[workspace.dependencies."tilde"]
package = "primary"
path = "$primary_dir"
version = "~$previous"
TOML
    mkdir -p "$fixture/unowned-primary/src"
    printf '[package]\nname="primary"\nversion="%s"\nedition="2024"\n[workspace]\n' "$previous" > "$fixture/unowned-primary/Cargo.toml"
    printf '#![no_std]\n' > "$fixture/unowned-primary/src/lib.rs"
    edge=primary
    case "$mode" in minor) edge='alias' ;; major) edge=tilde ;; zero-major) edge=exact ;; esac
    printf '\n[dependencies]\n%s.workspace = true\n' "$edge" >> "$root/secondary/Cargo.toml"
    printf '# Changelog\n\n## [%s]\n\n- Change.\n\n## [%s] - 2026-10-12\n\n- Previous.\n' "$candidate" "$previous" > "$root/CHANGELOG.md"
    (cd "$root" && "$REAL_CARGO" generate-lockfile --offline > /dev/null)
    mkdir "$root/originals"
    for file in Cargo.toml Cargo.lock CHANGELOG.md; do cp -p "$root/$file" "$root/originals/$file"; done
    "${YQ:-yq}" -p toml -o json '.' "$root/Cargo.toml" > "$root/before.json"
    (cd "$root" && RELEASE_PREVIOUS="$previous" RELEASE_VERSION="$candidate" bash "$helper" prepare primary) > "$fixture/owned-$mode.log" 2>&1
    (cd "$root" && RELEASE_VERSION="$candidate" bash "$helper" check primary)
    jq --arg version "$candidate" --arg short "${candidate%.*}" '
        .workspace.package.version = $version |
        .workspace.dependencies.primary.version = $short |
        .workspace.dependencies.alias.version = ("^" + $version) |
        .workspace.dependencies.exact.version = ("=" + $version) |
        .workspace.dependencies.tilde.version = ("~" + $version)' "$root/before.json" > "$root/expected.json"
    "${YQ:-yq}" -p toml -o json '.' "$root/Cargo.toml" > "$root/after.json"
    jq -e --slurpfile expected "$root/expected.json" '. == $expected[0]' "$root/after.json" > /dev/null
    # Identical bytes outside version literals, including opaque descriptions.
    grep -Fqx 'primary = { version = "DO NOT EDIT" }' "$root/Cargo.toml"
    grep -Fqx "primary = { version = \"${candidate%.*}\", path = \"$primary_dir\" } # preserve comment" "$root/Cargo.toml"
    if [[ "$mode" == zero-minor ]]; then
        for file in Cargo.toml Cargo.lock CHANGELOG.md; do cp -p "$root/originals/$file" "$root/$file"; done
        status=0
        (cd "$root" && CASE=prepared-local-failure RELEASE_PREVIOUS="$previous" RELEASE_VERSION="$candidate" bash "$helper" prepare primary) \
            > "$fixture/owned-restoration.log" 2>&1 || status=$?
        [[ "$status" == 25 ]] || exit 1
        for file in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$root/$file" "$root/originals/$file"; done
    fi
done
# A caller-owned range excluding the candidate must fail and restore all bytes.
root="$fixture/restrictive-local-range"
make_workspace "$root"
printf '\n[workspace.dependencies.primary]\npath="primary"\nversion=">=1.2, <1.3"\n' >> "$root/Cargo.toml"
printf '\n[dependencies]\nprimary.workspace=true\n' >> "$root/secondary/Cargo.toml"
(cd "$root" && "$REAL_CARGO" generate-lockfile --offline > /dev/null)
mkdir "$root/originals"
for file in Cargo.toml Cargo.lock CHANGELOG.md; do cp -p "$root/$file" "$root/originals/$file"; done
printf '# Changelog\n\n## [1.3.0]\n\n- Change.\n\n## [1.2.3] - 2026-10-12\n\n- Previous.\n' > "$root/CHANGELOG.md"
cp -p "$root/CHANGELOG.md" "$root/originals/CHANGELOG.md"
if (cd "$root" && RELEASE_VERSION=1.3.0 bash "$helper" prepare primary) > "$fixture/restrictive.log" 2>&1; then
    echo 'restrictive local range admitted' >&2; exit 1
fi
for file in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$root/$file" "$root/originals/$file"; done
for mode in renderer-failure check-failure interrupt literal-version unknown-local missing-member; do
    root="$fixture/$mode"
    make_workspace "$root"
    case "$mode" in
        literal-version) sed 's/version.workspace = true/version = "1.2.3"/' "$root/secondary/Cargo.toml" > "$fixture/literal"; cp "$fixture/literal" "$root/secondary/Cargo.toml" ;;
        unknown-local) printf '\n[[package]]\nname = "outside"\nversion = "1.2.3"\n' >> "$root/Cargo.lock" ;;
        missing-member) mv "$root/secondary/Cargo.toml" "$root/secondary/retained.toml" ;;
    esac
    mkdir "$root/originals"
    for file in Cargo.toml Cargo.lock CHANGELOG.md; do cp -p "$root/$file" "$root/originals/$file"; done
    status=0
    (cd "$root" && CASE="$mode" bash "$helper" prepare primary) > "$fixture/$mode.log" 2>&1 || status=$?
    [[ "$status" != 0 ]] || { echo "metadata failure case accepted: $mode" >&2; exit 1; }
    case "$mode" in renderer-failure) [[ "$status" == 23 ]] ;; interrupt) [[ "$status" == 143 ]] ;; check-failure) [[ "$status" == 25 ]] ;; esac
    for file in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$root/$file" "$root/originals/$file"; done
    if [[ "$mode" != missing-member ]]; then
        grep -F 'evidence retained:' "$fixture/$mode.log" > /dev/null
    fi
done
# Successful preparation preserves all unrelated bytes and metadata modes.
grep -Fqx '  version = "1.2.4" # Preserve this literal layout.' "$fixture/success/Cargo.toml"
grep -Fqx 'version = "1.2.3"' "$fixture/success/Cargo.toml"
[[ "$(perl -e 'printf "%o", (stat($ARGV[0]))[2]&07777' "$fixture/success/Cargo.toml")" == 640 ]] || exit 1

# Generic notes and failure restoration belong to the metadata producer.
for mode in undated-history history-no-lf dated-notes conflicting-date conflicting-notes restore-failure missing-date missing-version; do
    root="$fixture/$mode"
    make_workspace "$root"
    case "$mode" in
        undated-history) perl -pi -e 's/## \[1\.2\.3\] - 2026-10-12/## [1.2.3]/' "$root/CHANGELOG.md" ;;
        history-no-lf) perl -0pi -e 's/\n\z//' "$root/CHANGELOG.md" ;;
        dated-notes) perl -pi -e 's/## \[1\.2\.4\]/## [1.2.4] - 2026-10-13/' "$root/CHANGELOG.md" ;;
        conflicting-date) perl -pi -e 's/## \[1\.2\.4\]/## [1.2.4] - 2026-10-11/' "$root/CHANGELOG.md" ;;
        conflicting-notes) perl -pi -e 's/## \[1\.2\.4\]/## [9.9.9]/' "$root/CHANGELOG.md" ;;
    esac
    mkdir "$root/originals" "$root/attempts"
    for file in Cargo.toml Cargo.lock CHANGELOG.md; do cp -p "$root/$file" "$root/originals/$file"; done
    status=0
    case "$mode" in
        missing-date) (cd "$root" && env -u RELEASE_DATE TMPDIR="$root/attempts" bash "$helper" prepare primary) ;;
        missing-version) (cd "$root" && env -u RELEASE_VERSION TMPDIR="$root/attempts" bash "$helper" prepare primary) ;;
        *) (cd "$root" && CASE="$mode" TMPDIR="$root/attempts" bash "$helper" prepare primary) ;;
    esac > "$fixture/$mode.log" 2>&1 || status=$?
    case "$mode" in
        undated-history|history-no-lf)
            [[ "$status" == 0 ]] || exit 1
            perl -pe 's/## \[1\.2\.4\](?: - 2026-10-13)?/## [1.2.4] - 2026-10-13/' \
                "$root/originals/CHANGELOG.md" > "$root/expected-notes"
            cmp "$root/expected-notes" "$root/CHANGELOG.md"
            (cd "$root" && bash "$helper" check primary)
            ;;
        restore-failure)
            [[ "$status" == 1 ]] || exit 1
            for file in Cargo.toml CHANGELOG.md; do cmp "$root/originals/$file" "$root/$file"; done
            if cmp -s "$root/originals/Cargo.lock" "$root/Cargo.lock"; then exit 1; fi
            set -- "$root/attempts/"cargo-release-backup.*
            [[ $# == 1 ]] || exit 1
            cmp "$1/candidate.lock" "$root/Cargo.lock"
            for file in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$root/originals/$file" "$1/$file"; done
            grep -F 'metadata restoration incomplete; originals retained:' "$fixture/$mode.log" > /dev/null
            ;;
        *)
            [[ "$status" != 0 ]] || exit 1
            for file in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$root/originals/$file" "$root/$file"; done
            ;;
    esac
done
# Check a frozen source even when newer working source is incomplete or different.
cd "$fixture/success"
git init -q
git -c user.name=fixture -c user.email=fixture@example.invalid add Cargo.toml Cargo.lock CHANGELOG.md primary secondary
git -c user.name=fixture -c user.email=fixture@example.invalid commit -qm prepared
commit="$(git rev-parse HEAD)"
index_before="$(git write-tree)"
printf 'broken current metadata\n' > Cargo.toml
mv secondary/Cargo.toml secondary/current-manifest.retained
RELEASE_COMMIT="$commit" bash "$helper" check primary
[[ "$(git rev-parse HEAD)" == "$commit" && "$(git write-tree)" == "$index_before" ]]
[[ "$(cat Cargo.toml)" == 'broken current metadata' && ! -e secondary/Cargo.toml ]]
for failure in type export; do
    status=0
    GIT_FAILURE="$failure" RELEASE_COMMIT="$commit" bash "$helper" check primary > "$fixture/git-$failure.log" 2>&1 || status=$?
    [[ "$status" == 43 ]] || { echo 'failed Git output was accepted or its status lost' >&2; exit 1; }
done
# Manifest ordering is opt-in and belongs to the same selected source as graph
# admission. A poisoned working tree must never be used for the committed check.
git show "$commit:Cargo.toml" > "$fixture/committed-manifest"
for mode in default ordered sort-failure; do
    events="$fixture/order-$mode.events"
    status=0
    if [[ "$mode" == default ]]; then
        CASE=sort-failure ORDER_EVENTS="$events" RELEASE_COMMIT="$commit" \
            bash "$helper" check primary > "$fixture/order-$mode.log" 2>&1 || status=$?
        [[ "$status" == 0 && ! -e "$events.root" ]] || exit 1
    else
        CASE="$mode" ORDER_EVENTS="$events" RELEASE_COMMIT="$commit" \
            bash "$helper" check primary --manifest-order > "$fixture/order-$mode.log" 2>&1 || status=$?
        [[ "$(head -n 1 "$events")" == 'sort --workspace --check' ]] || exit 1
        [[ "$(cat "$events.root")" != "$PWD" ]] || exit 1
        cmp "$fixture/committed-manifest" "$events.manifest"
        if [[ "$mode" == ordered ]]; then
            [[ "$status" == 0 && ! -e "$(cat "$events.root")" ]] || exit 1
            grep -Fxq 'metadata --locked --offline --format-version 1' "$events"
        else
            [[ "$status" == 24 && -d "$(cat "$events.root")" ]] || exit 1
            if grep -q '^metadata ' "$events"; then exit 1; fi
            grep -Fq 'failed Cargo metadata check retained:' "$fixture/order-$mode.log"
        fi
    fi
    [[ "$(git rev-parse HEAD)" == "$commit" && "$(git write-tree)" == "$index_before" ]] || exit 1
    [[ "$(cat Cargo.toml)" == 'broken current metadata' && ! -e secondary/Cargo.toml ]] || exit 1
done
# Reject unsupported flag/operation combinations before invoking any Cargo tool.
for operation in prepare check-graph; do
    status=0
    ORDER_EVENTS="$fixture/rejected-order.events" bash "$helper" "$operation" primary --manifest-order \
        > "$fixture/order-$operation.log" 2>&1 || status=$?
    [[ "$status" == 2 && ! -e "$fixture/rejected-order.events" ]] || exit 1
done
status=0
bash "$helper" check primary --unknown > "$fixture/order-unknown.log" 2>&1 || status=$?
[[ "$status" == 2 ]] || exit 1
# Graph admission is independent of release notes and pending-version variables.
root="$fixture/graph"
make_workspace "$root"
rm "$root/CHANGELOG.md"
(cd "$root" && bash "$helper" check-graph primary)
for file in Cargo.toml Cargo.lock; do cp -p "$root/$file" "$root/$file.before"; done
for mode in graph-failure graph-empty-failure; do
    status=0
    (cd "$root" && CASE="$mode" bash "$helper" check-graph primary) > "$fixture/$mode.log" 2>&1 || status=$?
    [[ "$status" == 37 ]] || exit 1
    grep -q 'injected offline metadata failure' "$fixture/$mode.log"
    for file in Cargo.toml Cargo.lock; do cmp "$root/$file" "$root/$file.before"; done
done
[[ ! -e "$root/CHANGELOG.md" ]] || exit 1
for mode in mixed-version resolved-primary-version stale-primary stale-secondary extra-lock missing-lock unknown-primary; do
    root="$fixture/graph-$mode"
    make_workspace "$root"
    rm "$root/CHANGELOG.md"
    primary=primary
    case "$mode" in
        mixed-version)
            perl -pi -e 's/version.workspace = true/version = "1.2.2"/' "$root/secondary/Cargo.toml"
            (cd "$root" && "$REAL_CARGO" generate-lockfile --offline >/dev/null)
            ;;
        resolved-primary-version)
            perl -pi -e 's/version.workspace = true/version = "1.2.2"/' "$root/primary/Cargo.toml"
            (cd "$root" && "$REAL_CARGO" generate-lockfile --offline >/dev/null)
            ;;
        stale-primary|stale-secondary)
            package="${mode#stale-}"
            FIXTURE_PACKAGE="$package" perl -0pi -e '
                my $name = $ENV{FIXTURE_PACKAGE};
                s/(\[\[package\]\]\nname = "\Q$name\E"\nversion = ")[^"]+("\n)/${1}999.0.0$2/
                    or die "fixture package missing\n";
            ' "$root/Cargo.lock"
            ;;
        extra-lock) printf '\n[[package]]\nname = "outside"\nversion = "1.2.3"\n' >> "$root/Cargo.lock" ;;
        missing-lock) perl -0pi -e 's/\[\[package\]\]\nname = "secondary"\nversion = "1.2.3"\n//' "$root/Cargo.lock" ;;
        unknown-primary) primary=unknown ;;
    esac
    for file in Cargo.toml Cargo.lock; do cp -p "$root/$file" "$root/$file.before"; done
    if (cd "$root" && bash "$helper" check-graph "$primary") > "$fixture/graph-$mode.log" 2>&1; then
        echo "graph admission accepted $mode" >&2; exit 1
    fi
    for file in Cargo.toml Cargo.lock; do cmp "$root/$file" "$root/$file.before"; done
    [[ ! -e "$root/CHANGELOG.md" ]] || exit 1
done
echo 'Cargo metadata preparation, restoration, workspace admission and selected-commit checks passed (literal rendering; no manifest mutation tools).'
complete=true
