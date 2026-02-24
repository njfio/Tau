# Tasks: Issue #3539 - M320 TUI-driven interactive agent flow

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance): add/adjust tests to fail until `tau-tui agent`
   mode and launcher default `tui` agent routing exist.
2. [x] T2 (GREEN, Implementation): implement `tau-tui agent` parse/dispatch +
   interactive runner command construction.
3. [x] T3 (GREEN, Contract): update `tau-unified.sh` `tui` routing to default
   agent mode while preserving explicit `--live-shell` mode.
4. [x] T4 (GREEN, Docs): update README and operator deployment guide with
   interactive default + explicit live-shell workflow.
5. [x] T5 (VERIFY): run scoped tests and formatting checks; update AC evidence
   and mark spec implemented.

## TDD Evidence
### RED
- `cargo test -p tau-tui --test tui_demo_smoke conformance_tui_agent_mode_dry_run_emits_interactive_launch_contract`
  - failed pre-implementation with `unknown argument: agent`.
- `bash scripts/run/test-tau-unified.sh`
  - failed pre-implementation with missing marker
    `tau-unified: launching tui (agent)`.

### GREEN
- `cargo test -p tau-tui` passed.
- `bash scripts/run/test-tau-unified.sh` passed.

### REGRESSION
- `cargo fmt --check` passed.
- `cargo clippy -p tau-tui -- -D warnings` passed.
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-tui` (`main.rs` parser/command tests) |  |
| Property | N/A |  | No randomized invariants introduced |
| Contract/DbC | N/A |  | No new `contracts` annotations |
| Snapshot | N/A |  | No snapshot fixtures needed |
| Functional | ✅ | `cargo test -p tau-tui`; `bash scripts/run/test-tau-unified.sh` |  |
| Conformance | ✅ | agent-mode dry-run contract + launcher mode-routing assertions |  |
| Integration | ✅ | binary-level `tau-tui` smoke/conformance + launcher runner-hook flow |  |
| Fuzz | N/A |  | No untrusted parser/input surface introduced |
| Mutation | N/A |  | CLI/TUI orchestration scope; no critical algorithm path |
| Regression | ✅ | parse fail-closed checks + launcher watch-flag mode guard coverage |  |
| Performance | N/A |  | No hotspot/perf-sensitive path changed |
