#!/bin/sh
set -eu

# Deterministic subprocess fixture. Test inputs are passed as positional arguments
# and quoted environment values; this fixture never contacts a network service.
if [ -n "${FIXTURE_MARKER-}" ]; then
    printf 'invoked\n' >> "$FIXTURE_MARKER"
fi

case "${1-}" in
    --version)
        if [ "${FIXTURE_VERSION_MODE-}" = invalid-utf8 ]; then
            printf '\377'
        elif [ "${FIXTURE_VERSION_MODE-}" = fail ]; then
            printf '%s\n' 'test-host-tool version 1'
            printf 'version failed' >&2
            exit 23
        else
            printf '%s\n' "${FIXTURE_VERSION-test-host-tool version 1}"
        fi
        ;;
    --arguments)
        shift
        printf '[%s]' "$@"
        ;;
    --environment)
        printf '%s|%s' "${FIXTURE_VALUE-absent}" "${HOME-absent}"
        ;;
    --cwd)
        pwd -P
        ;;
    --stdin)
        if read -r _line; then exit 24; fi
        printf 'stdin closed'
        ;;
    --fail)
        printf '%s' "${FIXTURE_SECRET-failure stdout}"
        printf 'failure stderr' >&2
        exit 23
        ;;
    --flood-stdout)
        while :; do printf '0123456789abcdef'; done
        ;;
    --flood-stderr)
        while :; do printf '0123456789abcdef' >&2; done
        ;;
    --both)
        count=0
        while [ "$count" -lt 10000 ]; do
            printf '0123456789abcdef'
            printf 'fedcba9876543210' >&2
            count=$((count + 1))
        done
        ;;
    --wait)
        while :; do :; done
        ;;
    --closed-wait)
        exec 1>&- 2>&-
        while :; do :; done
        ;;
    --descendant)
        /bin/sleep 1 &
        exit 0
        ;;
    *)
        case "${FIXTURE_CANDID_MODE-normal}" in
            invalid-utf8) printf '\377' ;;
            empty) : ;;
            unterminated) printf 'service : {}' ;;
            source-path) printf '%s' "$1" >&2; printf 'service : {}\n' ;;
            mutate) printf 'x' >> "$1"; printf 'service : {}\n' ;;
            remove) rm -- "$1"; printf 'service : {}\n' ;;
            grow) printf '0123456789abcdef' >> "$1"; printf 'service : {}\n' ;;
            fail) printf 'extract failed' >&2; exit 23 ;;
            *) printf '//  \r\nservice : {  \r\n  query : () -> () query;\t\r\n}' ;;
        esac
        ;;
esac
