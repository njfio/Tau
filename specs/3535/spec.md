# Spec: Issue #3535 - M319 unified one-command runtime entrypoint

Status: Implemented

## Problem Statement
The runtime feels fragmented because operators must run multiple independent
commands to launch/manage core surfaces. There is no single lifecycle command
to bring up the integrated runtime and manage status/shutdown deterministically.

## Scope
In scope:
- Add `scripts/run/tau-unified.sh` lifecycle entrypoint.
- Add `scripts/run/test-tau-unified.sh` contract suite.
- Support `up`, `status`, `down`, and optional `tui` mode from one command.
- Persist PID/log/runtime files under `.tau/unified`.
- Update README and operator deployment docs with one-command usage.

Out of scope:
- New gateway/TUI feature development.
- New daemon/system-service integrations.
- Cross-host orchestration automation.

## Acceptance Criteria
### AC-1 One-command launcher starts unified runtime with deterministic artifacts
Given local execution,
when `scripts/run/tau-unified.sh up` runs,
then it starts the runtime via configured launch command, persists pid/log/cmd
artifacts, and prints operator endpoints.

### AC-2 Lifecycle controls expose status and deterministic shutdown
Given an existing launcher state,
when `status` and `down` are executed,
then the script reports runtime status consistently and performs shutdown with
pid-file cleanup.

### AC-3 Contract test enforces fail-closed lifecycle argument behavior
Given `scripts/run/test-tau-unified.sh`,
when pass/fail paths run,
then invalid commands/flag misuse and lifecycle edge paths fail closed with
clear diagnostics.

### AC-4 Docs provide one-command unified program path
Given README and operator deployment documentation,
when reviewed,
then both include explicit one-command launcher usage and lifecycle examples.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | launcher script | `up` | pid/log/cmd artifacts emitted + endpoints printed |
| C-02 | AC-2 | Functional/Regression | launcher state | `status`/`down` | deterministic status and cleanup behavior |
| C-03 | AC-3 | Conformance | launcher contract test | run test script | fail-closed invalid arg/edge-path coverage |
| C-04 | AC-4 | Functional | docs | inspect | one-command usage documented |

## Success Metrics / Observable Signals
- Operators can run one command to bring up unified runtime surfaces.
- Launcher lifecycle behavior is deterministic and test-covered.
- Docs reference the one-command operational path.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `bash scripts/run/test-tau-unified.sh` validated `up` emits deterministic artifacts (`.tau/unified/tau-unified.pid`, `tau-unified.log`, `tau-unified.last-cmd`) and endpoint markers. |
| AC-2 | ✅ | `bash scripts/run/test-tau-unified.sh` validated `status` reports running state and `down` performs shutdown + pid-file cleanup with deterministic output. |
| AC-3 | ✅ | `bash scripts/run/test-tau-unified.sh` validated fail-closed unknown-command and lifecycle edge paths (invalid command, down when stopped). |
| AC-4 | ✅ | `README.md` and `docs/guides/operator-deployment-guide.md` include explicit one-command launcher flow via `scripts/run/tau-unified.sh`. |
