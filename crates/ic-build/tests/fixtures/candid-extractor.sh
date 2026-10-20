#!/bin/sh
set -eu

# Deterministic subprocess fixture. Test inputs are passed as positional arguments
# and quoted environment values; this fixture never contacts a network service.
if [ -n "${FIXTURE_MARKER-}" ]; then
    printf 'invoked\n' >> "$FIXTURE_MARKER"
fi

case "${1-}" in
    --version)
        printf '%s\n' 'test-host-tool version 1'
        ;;
    /*)
        case "${FIXTURE_CANDID_MODE-normal}" in
            invalid-utf8) printf '\377' ;;
            empty) : ;;
            unterminated) printf 'service : {}' ;;
            source-path) printf '%s' "$1" >&2; printf 'service : {}\n' ;;
            mutate) printf 'x' >> "$1"; printf 'service : {}\n' ;;
            remove) rm -- "$1"; printf 'service : {}\n' ;;
            grow) printf '0123456789abcdef' >> "$1"; printf 'service : {}\n' ;;
            fail) printf 'extract failed' >&2; exit 23 ;;
            normal) printf '//  \r\nservice : {  \r\n  query : () -> () query;\t\r\n}' ;;
            *) printf 'unexpected extraction mode\n' >&2; exit 64 ;;
        esac
        ;;
    *) printf 'expected an absolute source path\n' >&2; exit 64 ;;
esac
