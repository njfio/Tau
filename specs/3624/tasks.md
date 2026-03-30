# Tasks: Issue #3624 - Restore action-history persistence and telemetry fidelity on primary agent paths

Status: Implemented
Milestone: M329
Parent: #3623

1. [x] T1 (RED): add failing `tau-agent-core` coverage for prompt-path
   persistence and placeholder telemetry values.
2. [x] T2 (GREEN): preserve/finalize prompt-path persistence behavior and thread
   real turn/latency values into action-history tool records.
3. [x] T3 (VERIFY): run scoped `tau-agent-core` tests and any touched targeted
   integration coverage; update issue/process artifacts.

## Test Mapping
- AC-1 -> C-01/C-02 -> prompt-path persistence tests
- AC-2 -> C-03 -> tool-history telemetry tests
- AC-3 -> C-04 -> scoped verification commands
