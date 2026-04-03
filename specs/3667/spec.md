# Spec: Issue #3667 - Align tau-unified launcher with gateway turn timeout budget

Status: Implemented

## Problem Statement
`tau-unified.sh` forwards `--request-timeout-ms` and provider CLI timeout flags
to the runtime, but it does not forward `--turn-timeout-ms`. In interactive
gateway Ralph-loop mode that leaves the gateway attempt timeout disabled (`0`),
so request handling can hang until the TUI client disconnects. When that
happens, no completed attempt trace is persisted and the mission is left
`running`.

## Scope
In scope:
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `specs/3667/spec.md`
- `specs/3667/plan.md`
- `specs/3667/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing Rust-side gateway timeout defaults
- changing TUI timeout derivation logic
- redesigning gateway retry policy

## Acceptance Criteria
### AC-1 Unified launcher forwards gateway turn timeout
Given `tau-unified.sh up` or bootstrap `tui` is invoked with a request timeout,
when it builds the runtime command,
then it also forwards `--turn-timeout-ms` using the same operator-facing budget.

### AC-2 Launcher tests cover default and override propagation
Given the launcher shell tests run,
when they inspect the recorded runtime command,
then they assert both the default timeout and an explicit override are
forwarded to `--turn-timeout-ms`.

### AC-3 Interactive gateway turns fail through structured runtime paths first
Given the runtime has a gateway turn timeout budget,
when an action request stalls,
then the gateway can terminate the attempt through its own timeout/runtime
handling before the TUI client disconnects first.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `tau-unified.sh up` records `--turn-timeout-ms <request_timeout_ms>` in the
  spawned runtime command.
- C-02 / AC-1 / Functional:
  bootstrap `tui` preserves the same runtime `--turn-timeout-ms` budget.
- C-03 / AC-2 / Regression:
  launcher tests assert both default and explicit override propagation for
  `--turn-timeout-ms`.
- C-04 / AC-3 / Regression:
  existing launcher tests remain green after the timeout forwarding change.

## Success Metrics / Observable Signals
- Interactive gateway requests no longer hang with mission state stuck
  `running` solely because launcher omitted the gateway turn timeout.
- New action failures surface through structured gateway/runtime errors with
  attempt traces instead of generic transport disconnects.

## Files To Touch
- `specs/3667/spec.md`
- `specs/3667/plan.md`
- `specs/3667/tasks.md`
- `specs/milestones/m334/index.md`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
