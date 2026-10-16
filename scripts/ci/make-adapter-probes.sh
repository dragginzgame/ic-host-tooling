#!/usr/bin/env bash
# Caller-owned Makefiles, target rosters and effect stubs; no real release effects.
make_probe_target_admission() {
    [[ $# -ge 5 ]] || return 2
    local executable="$1" makefile="$2" directory="$3" events="$4"
    shift 4
    local target mode status
    [[ -f "$makefile" && -d "$directory" ]] || return 2
    # Consumer targets exercise each unsafe behavior once. The producer owns
    # aliases, inherited/replaced flags and hidden-MFLAGS combinations below.
    for target in "$@"; do
        for mode in -i -n -t -q; do
            : > "$events" || return
            status=0
            "$executable" --no-print-directory -f "$makefile" "$mode" "$target" \
                > "$directory/output" 2>&1 || status=$?
            if [[ "$status" != 2 || -s "$events" ]]; then
                printf 'Make target admission failed: %s %s (status %s)\n' "$target" "$mode" "$status" >&2
                return 1
            fi
        done
    done
}

make_probe_execution_modes() {
    [[ $# -ge 5 ]] || return 2
    local executable="$1" makefile="$2" directory="$3" events="$4"
    shift 4
    local target mode selection status assignment
    local flags=()
    [[ -f "$makefile" && -d "$directory" ]] || return 2
    for target in "$@"; do
        for mode in -i --ignore-errors -n --dry-run --just-print --recon -t --touch -q --question -kin; do
            for selection in direct inherited cleared replaced both-hidden; do
                : > "$events" || return
                status=0
                if [[ "$selection" == inherited ]]; then
                    _shared_make_execution_checked=yes MAKEFLAGS="$mode" "$executable" --no-print-directory \
                        -f "$makefile" "$target" > "$directory/output" 2>&1 || status=$?
                else
                    flags=()
                    case "$selection" in
                        cleared) flags=(MAKEFLAGS=) ;;
                        replaced) flags=(MAKEFLAGS=--no-print-directory) ;;
                        both-hidden) flags=(MAKEFLAGS= MFLAGS=) ;;
                    esac
                    "$executable" --no-print-directory -f "$makefile" "$mode" "$target" \
                        ${flags[@]+"${flags[@]}"} > "$directory/output" 2>&1 || status=$?
                fi
                if [[ "$status" != 2 || -s "$events" ]]; then
                    printf 'Make admission failed: %s %s %s (status %s)\n' "$target" "$mode" "$selection" "$status" >&2
                    return 1
                fi
            done
        done
    done
    for assignment in 'MFLAGS :=' 'override MFLAGS :='; do
        printf '%s\n' "$assignment" > "$directory/hidden-mflags.mk" || return
        : > "$events" || return
        status=0
        "$executable" --no-print-directory -i -f "$directory/hidden-mflags.mk" -f "$makefile" \
            "$1" MAKEFLAGS= > "$directory/output" 2>&1 || status=$?
        [[ "$status" == 2 && ! -s "$events" ]] || return 1
    done
}

make_probe_parallel_dispatch() {
    [[ $# -ge 5 ]] || return 2
    local executable="$1" makefile="$2" directory="$3" events="$4" target
    shift 4
    local parallel=(-j4)
    if "$executable" --help | grep -q -- --jobserver-style; then parallel+=(--jobserver-style=pipe); fi
    for target in "$@"; do
        : > "$events" || return
        "$executable" "${parallel[@]}" --no-print-directory -f "$makefile" "$target" \
            > "$directory/output" 2>&1 || return
        [[ -s "$events" ]] || return 1
    done
}
