# Ownership and Make admission audit

Maintainer-requested audit and repair on clean released Host 0.9.6,
`6aaa4229913bafc68a443e7e855468efcca8ee7a`. Methods are
`audits/module-surface-hardening.md` and `audits/complexity-and-technical-debt.md`
at adopted Shared `47d6ae6488b8007323fa7c2e22a6efa11d77ae63`. They are unchanged
in the subsequently adopted 0.2.8 revision
`b2646cde9abbc8861857a4379c683a0c19eba43e`. AGENTS.md and docs/extraction.md
provide local ownership and host constraints.

The bounded review covers child ownership/reaping, durable publication adapters
and Make admission. It does not repeat the previous response/stream review or
claim a complete syscall-race, native macOS or performance audit.

| Candidate / behavior axis | Authority and consumers | Disposition |
| --- | --- | --- |
| Child scope: direct or group | Spawn establishes scope; waiting and cleanup preserve it | Retain; unifying scope would change foreground and descendant ownership |
| Child lifecycle: owned, lost externally, reaped, termination started | `OwnedChild` gates handoff, signals and repeated cleanup | Retain; cached status cannot represent externally lost ownership or an exhausted reap allowance |
| Publication: pathname or held parent, replace or create-new, optional staged admission | Public adapters converge on `commit_at_with_hook` | Retain entrypoints and before/after-publication errors; parent custody and external executable admission differ |
| Make executable versus recursive command | Shared admission probes execution; caller owns recursive arguments | Adopt committed upstream correction; previous guard conflated these inputs |

Two change rehearsals explain retained structure: adding a cleanup timing choice
belongs in `CleanupPolicy`/termination, without making communication own lifecycle;
adding a publication admission belongs at the existing staged callback, without
replacing descriptor identity checks. Neither rehearsal demonstrates a need for
a new API or implementation. Repeated durable identity checks protect separate
producer/admission/publication boundaries, and early permissions admission avoids
filesystem effects before invalid input is rejected. No Rust surface deletion
or hard cut is justified. These are source-review conclusions, not speed claims.

**MEDIUM, repaired locally — argument-bearing recursive Make is rejected.**
Extended Host's actual-Makefile fixture with recursive `MAKE` selecting both
`Makefile` and `overrides.mk`, alongside an external unselected root. Under 0.2.7
even recursive `help` fails during admission. The retained failure is
`/tmp/host-make-snapshot.1Q31G1/recursive-help.log`. Canonical adoption of Shared
0.2.8 fixes the case, qualifies the running Make executable and locates admission
beside the selected include. The override file's selected remote reaches the
substitute runner, while unsafe modes still refuse before effects.

Host retains its local-root runtime policy: the new upstream admission binding
does not replace that consumer decision. Only committed upstream files were
exported from a clean private clone; dirty sibling exporter/verifier/report work
was excluded. The 88-file roster is unchanged. Both changelog views select
compatible 0.9.7. Follow-up/delivery remains in
[#42](https://github.com/dragginzgame/ic-host-tooling/issues/42), with generic
upstream ownership in [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).

Focused Linux checks pass: actual Host Make routing/admission and release adapter
fixtures on GNU Make 4.3/Bash 5 and GNU Make 3.81/Bash 3.2; real prepared-formatter
hook adoption on both pairs; 17 selected `ic-host-process` child tests with locked
offline dependencies; selected ShellCheck, snapshot, documentation and whitespace
checks. Child tests include real process cleanup and a deliberately substituted
already-started reap state for exhausted-budget recovery; they do not simulate
an unkillable OS process. The first hook attempt used an obsolete unsorted
manifest fixture and failed; rebuilding that input from current Cargo.toml fixed
the harness. Both attempts remain under `/tmp/ic-host-097-audit/`.

Verdict: **PASS within the selected review after local adoption**, with native
macOS qualification still outstanding. No functions, methods or types were
removed. Package versions, lockfile, pins and real Git index/config remain
unchanged. No sibling edits, full gate, downloads, real release/hook activation,
commit or push occurred. Linux compatibility is not native macOS evidence.
