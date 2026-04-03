# Spec: Issue #3661 - Recycle stale tau-unified runtime after repo/runtime fingerprint changes

Status: Implemented

## Problem Statement
`just tui` and `tau-unified.sh tui` can silently reuse a long-running
`tau-coding-agent` process that was started from an older checkout state. When
the repo has moved forward, the operator can end up talking to a stale gateway
binary that does not include the currently merged Ralph-loop behavior. The
launcher needs an explicit runtime freshness contract so local testing reflects
the code on the current checkout.

## Scope
In scope:
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `specs/3661/spec.md`
- `specs/3661/plan.md`
- `specs/3661/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing gateway loop semantics or verifier behavior
- TUI command behavior outside launcher/runtime bootstrap
- dashboard/runtime state migration beyond launcher metadata

## Acceptance Criteria
### AC-1 Launcher records a reusable runtime fingerprint
Given `tau-unified.sh up` starts a runtime,
when the launcher persists its runtime bookkeeping,
then it stores a fingerprint describing the current repo/runtime state together
with the existing pid/command files.

### AC-2 Launcher recycles stale runtime for `up`
Given a live runtime process exists and the persisted runtime fingerprint differs
from the current repo/runtime fingerprint,
when the operator runs `tau-unified.sh up`,
then the launcher stops the stale process, replaces it with a fresh runtime, and
prints a clear recycle message instead of silently returning `already running`.

### AC-3 Launcher recycles stale runtime for `tui` bootstrap
Given `tau-unified.sh tui` is bootstrapping a runtime and the existing runtime
fingerprint is stale,
when bootstrap runs,
then the launcher recycles the runtime before launching the TUI so the operator
does not connect to an older gateway build.

### AC-4 Same-fingerprint reuse still stays cheap
Given a live runtime process exists and the persisted runtime fingerprint matches
the current repo/runtime fingerprint,
when the operator runs `tau-unified.sh up` or bootstrapped `tui`,
then the launcher keeps the current process and does not issue an unnecessary
restart.

## Conformance Cases
- C-01 / AC-1 / Regression:
  start the launcher in runner mode and verify a runtime fingerprint file is
  written alongside pid/log/command metadata.
- C-02 / AC-2 / Regression:
  seed a live pid plus mismatched fingerprint, run `up`, and verify the launcher
  logs a recycle event and issues `down` followed by `up`.
- C-03 / AC-3 / Regression:
  seed a live pid plus mismatched fingerprint, run bootstrapped `tui`, and
  verify the launcher recycles the runtime before launching the TUI.
- C-04 / AC-4 / Functional:
  seed a live pid plus matching fingerprint, run `up` and bootstrapped `tui`,
  and verify the launcher reuses the runtime without issuing `down`.

## Success Metrics / Observable Signals
- Local `just tui` sessions no longer reuse a runtime started from an older repo
  state after merged gateway/loop changes
- Launcher output explicitly says when it is reusing versus recycling a runtime
- Runner-mode tests cover both stale and current runtime decisions

## Files To Touch
- `specs/3661/spec.md`
- `specs/3661/plan.md`
- `specs/3661/tasks.md`
- `specs/milestones/m334/index.md`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
