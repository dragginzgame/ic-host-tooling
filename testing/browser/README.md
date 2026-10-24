# Browser fixture sessions

The private `@dragginzgame/ic-testkit-browser` Node package is consumed from an
immutable Git commit. It has no npm dependencies or installation scripts, and is
not part of the published Rust crate. Node 22 or later is required.

`withFixtureSession(options, async connection => { ... })` spawns one fixture,
removes stale readiness data, waits for complete JSON and passes it to the body.
Options select `command`, `args`, `cwd`, `env`, `readyPath`, `logPath`, and explicit
`startupMs`, `stopMs`, `pollMs` integers in 1..2147483647. The caller creates parent
directories and owns executable/environment admission, readiness schema, source
custody, body deadlines and the browser lifecycle. Paths must be distinct,
caller-owned files. Logs append, including on startup/body failure.

Cleanup sends a newline, then SIGTERM and SIGKILL, waiting at most `stopMs` after
each stage. It reaps the direct child; the fixture must manage its descendants.
It preserves body failures, including falsy exceptions, and aggregates cleanup
and unsuccessful fixture-exit failures after readiness. Signals sent by session
cleanup are expected shutdown, while other signals and nonzero exit codes fail
the session. The callback is awaited before reporting failures; its work is never
abandoned by racing it against child exit. This helper targets Linux/macOS,
not Windows process termination.

Run `node --test testing/browser/session.test.mjs`. Tests use disposable Node
children; installed PocketIC/browser qualification remains with the consumer.
