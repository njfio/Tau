# Issue 3793: Unified Launcher Starts The Real Gateway Binary

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

`scripts/run/tau-unified.sh up` records and executes `cargo run -p
tau-coding-agent -- ...`. The `tau-coding-agent` package now exposes multiple
binaries, so Cargo exits with `could not determine which binary to run`. The
launcher still briefly writes a pid file, which can make the operator believe a
real gateway is available while no HTTP server is listening.

## Scope

In scope:
- Make the unified launcher execute the `tau-coding-agent` binary explicitly.
- Make the non-runner launcher detach the long-running runtime so it survives
  after `up` exits.
- Lock the generated command contract with a shell regression test.
- Validate the real `/ops/harness` route through a loopback HTTP server and
  Browser Use.

Out of scope:
- Changing gateway auth semantics.
- Changing harness route data or UI layout.
- Adding new runtime dependencies.

## Acceptance Criteria

AC-1: Given `tau-unified.sh up` builds its runtime command, when the command is
recorded, then it includes `cargo run -p tau-coding-agent --bin
tau-coding-agent --`.

AC-2: Given the unified launcher starts with localhost-dev auth, when `up`
returns, then the tracked runtime process remains alive until `down` is called.

AC-3: Given the launched gateway is ready, when `/gateway/status` and
`/ops/harness` are requested, then both return HTTP 200 on the loopback bind.

AC-4: Given the real `/ops/harness` route is open in Browser Use, when the
harness is inspected, then the browser URL is localhost, the harness panel is
present, and console errors are zero.

## Conformance Cases

C-01 maps to AC-1: `scripts/run/test-tau-unified.sh` asserts the command file
contains the explicit binary selector.

C-02 maps to AC-2: `tau-unified.sh status` reports the tracked pid after the
`up` process has returned.

C-03 maps to AC-3: Local curl readiness checks against the launched gateway
return `200`.

C-04 maps to AC-4: Browser Use navigates to a loopback `/ops/harness` URL and
captures DOM evidence.

## Success Signals

- `bash scripts/run/test-tau-unified.sh`
- `scripts/run/tau-unified.sh up --auth-mode localhost-dev --bind 127.0.0.1:8791`
- `scripts/run/tau-unified.sh status`
- `curl http://127.0.0.1:8791/gateway/status`
- Browser Use inspection of the real `/ops/harness` URL.
