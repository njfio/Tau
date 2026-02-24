# Tasks: Issue #3551 - Unified TUI runtime bootstrap + codex login command compatibility

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED): add/update tests to express bootstrap + auth launch command
   expectations and capture failing evidence.
2. [x] T2 (GREEN): implement `tau-unified.sh tui` bootstrap controls and
   runtime bootstrap behavior.
3. [x] T3 (GREEN): switch OpenAI auth launch command to `codex login`.
4. [x] T4 (VERIFY): run launcher/provider/auth test matrix and lints.
5. [x] T5 (DOC): update spec/plan/tasks to `Implemented` with evidence.

## TDD Evidence
### RED
- `bash scripts/run/test-tau-unified.sh`
  - initial failure after introducing bootstrap assertions:
    `assertion failed (runner status logged): expected output to contain 'runner_mode=status'`.
  - cause: over-aggressive log truncation in test harness.

### GREEN
- `bash scripts/run/test-tau-unified.sh` passed after log-count assertion fix.
- `cargo test -p tau-provider` passed (`76 + 2` tests).
- `cargo test -p tau-coding-agent functional_execute_auth_command_login_openai_launch_executes_codex_login_command` passed.

### REGRESSION
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-provider`; targeted coding-agent auth launch test |  |
| Property | N/A |  | No randomized invariant logic |
| Contract/DbC | N/A |  | No contract macro surfaces changed |
| Snapshot | N/A |  | No snapshot suites affected |
| Functional | ✅ | `bash scripts/run/test-tau-unified.sh` |  |
| Conformance | ✅ | launcher bootstrap assertions + codex login launch assertion |  |
| Integration | ✅ | `cargo test -p tau-provider` auth workflow conformance subset |  |
| Fuzz | N/A |  | No parser/untrusted input changes |
| Mutation | N/A |  | Launcher/auth UX compatibility scope |
| Regression | ✅ | RED->GREEN launcher harness evidence + fmt/shell checks |  |
| Performance | N/A |  | No hotspot changes |
