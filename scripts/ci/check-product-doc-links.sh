#!/usr/bin/env bash
set -euo pipefail

# Include maintained product docs, including new untracked docs, while respecting
# Git's exclusions for generated output and local tool/build directories.
documents=()
inventory="$(mktemp)"
trap 'rm -f "$inventory"' EXIT
git ls-files --cached --others --exclude-standard -z -- '*.md' > "$inventory"
while IFS= read -r -d '' document; do
    case "$document" in
        README.md|AGENTS.md|CHANGELOG.md|docs/*|crates/*|apps/*|testing/*)
            documents+=("$document") ;;
    esac
done < "$inventory"
[[ ${#documents[@]} != 0 ]] || { echo 'No product documentation selected' >&2; exit 1; }
perl scripts/ci/check-documentation-links.pl --root "$PWD" "${documents[@]}"
