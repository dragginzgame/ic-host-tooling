#!/bin/sh
# Deterministic Git substitute; no repository or network effects.
set -eu
if [ "$#" = 1 ] && [ "$1" = --version ]; then
    printf 'test-host-tool version 1\n'
    exit 0
fi
[ "$1" = --no-optional-locks ] && [ "$2" = -c ] && [ "$3" = core.fsmonitor=false ] || exit 90
if [ -n "${FIXTURE_MARKER:-}" ]; then
    printf '%s ' "$@" >> "$FIXTURE_MARKER"
    printf '\n' >> "$FIXTURE_MARKER"
fi
case "$4" in
    rev-parse)
        [ "$#" = 6 ] && [ "$5" = --verify ] || exit 91
        case "$6" in
            HEAD) ;;
            'HEAD^{tree}')
                if [ "${FIXTURE_MODE:-}" = fail-tree ]; then
                    printf 'retained tree diagnostic\n' >&2
                    exit 23
                fi
                ;;
            *) exit 92 ;;
        esac
        case "${FIXTURE_MODE:-}" in
            invalid-id) printf 'not an object id\n' ;;
            uppercase-id) printf 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n' ;;
            sha256) printf '%064d\n' 0 ;;
            *) printf '%040d\n' 0 ;;
        esac
        ;;
    status)
        [ "$#" = 8 ] && [ "$5" = --porcelain=v1 ] && [ "$6" = -z ] || exit 93
        case "$7" in --untracked-files=no|--untracked-files=normal|--untracked-files=all) ;; *) exit 94 ;; esac
        case "$8" in --ignore-submodules=none|--ignore-submodules=untracked|--ignore-submodules=dirty|--ignore-submodules=all) ;; *) exit 95 ;; esac
        case "${FIXTURE_MODE:-}" in
            clean|sha256) ;;
            bad-status) printf '?? unterminated' ;;
            *) printf '?? space \377path\000' ;;
        esac
        ;;
    *) exit 96 ;;
esac
