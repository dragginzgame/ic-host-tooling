#!/usr/bin/env bash
# Shared companions: scripts/ci/check-native-host.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/native-host-test.XXXXXX")"
fixture="$(cd -P "$fixture" && pwd)"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Native host fixtures retained"' EXIT
mkdir "$fixture/bin"
cat > "$fixture/bin/uname" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$1" >> "$HOST_OBSERVATIONS"
case "$1" in
    -s) printf '%s\n' "$HOST_OS"; exit "${HOST_OS_STATUS:-0}" ;;
    -m) printf '%s\n' "$HOST_ARCH"; exit "${HOST_ARCH_STATUS:-0}" ;;
    *) exit 99 ;;
esac
STUB
cat > "$fixture/bin/sw_vers" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$1" >> "$HOST_OBSERVATIONS"
[[ "$1" == -productVersion ]] || exit 99
printf '%s\n' "$HOST_VERSION"
exit "${HOST_VERSION_STATUS:-0}"
STUB
chmod +x "$fixture/bin/uname" "$fixture/bin/sw_vers"
export PATH="$fixture/bin:$PATH" HOST_OBSERVATIONS="$fixture/observations"
for scenario in linux darwin-intel darwin-arm wrong-os wrong-arch wrong-version malformed-version \
    failed-os failed-arch failed-version missing-major extra-major bad-major bad-os bad-arch; do
    export HOST_OS=Darwin HOST_ARCH=arm64 HOST_VERSION=15.0.1
    export HOST_OS_STATUS=0 HOST_ARCH_STATUS=0 HOST_VERSION_STATUS=0
    args=(Darwin arm64 15)
    expected=0
    case "$scenario" in
        linux) HOST_OS=Linux; HOST_ARCH=x86_64; args=(Linux x86_64) ;;
        darwin-intel) HOST_ARCH=x86_64; args=(Darwin x86_64 15) ;;
        darwin-arm) ;;
        wrong-os) HOST_OS=Linux; expected=1 ;;
        wrong-arch) HOST_ARCH=x86_64; expected=1 ;;
        wrong-version) HOST_VERSION=14.9; expected=1 ;;
        malformed-version) HOST_VERSION=15.garbage; expected=1 ;;
        failed-os) HOST_OS_STATUS=43; expected=43 ;;
        failed-arch) HOST_ARCH_STATUS=44; expected=44 ;;
        failed-version) HOST_VERSION_STATUS=45; expected=45 ;;
        missing-major) args=(Darwin arm64); expected=2 ;;
        extra-major) args=(Linux x86_64 15); expected=2 ;;
        bad-major) args=(Darwin arm64 015); expected=2 ;;
        bad-os) args=(Other arm64); expected=2 ;;
        bad-arch) args=(Darwin 'arm 64' 15); expected=2 ;;
    esac
    : > "$HOST_OBSERVATIONS"
    status=0
    "$BASH" "$ROOT/scripts/ci/check-native-host.sh" "${args[@]}" > "$fixture/$scenario.log" 2>&1 || status=$?
    [[ "$status" == "$expected" ]] || exit 1
    if [[ "$scenario" == wrong-os || "$scenario" == failed-os ]]; then
        printf '%s\n' -s > "$fixture/expected-observations"
        cmp "$fixture/expected-observations" "$HOST_OBSERVATIONS"
    elif [[ "$scenario" == linux || "$scenario" == wrong-arch || "$scenario" == wrong-version || "$scenario" == failed-arch ]]; then
        printf '%s\n' -s -m > "$fixture/expected-observations"
        if [[ "$scenario" == wrong-version ]]; then printf '%s\n' -productVersion >> "$fixture/expected-observations"; fi
        cmp "$fixture/expected-observations" "$HOST_OBSERVATIONS"
    elif [[ "$expected" == 2 ]]; then
        [[ ! -s "$HOST_OBSERVATIONS" ]] || exit 1
    fi
done
fixture_complete=true
echo 'Native host admission, failed observations and explicit selection passed'
