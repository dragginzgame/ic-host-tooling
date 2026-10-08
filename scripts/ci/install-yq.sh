#!/usr/bin/env bash
# Shared companions: scripts/ci/install-ci-tool.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
exec bash "$SCRIPT_DIR/install-ci-tool.sh" yq "$@"
