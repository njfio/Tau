# Tasks: Issue #3543 - GPT-5 default model migration for runtime entrypoints

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED): update/add targeted tests expecting GPT-5.2 defaults and record
   pre-implementation failures.
2. [x] T2 (GREEN): migrate CLI/TUI/launcher default model ids to
   `openai/gpt-5.2`.
3. [x] T3 (GREEN): update README/operator docs to remove
   `openai/gpt-4o-mini` recommendations.
4. [x] T4 (VERIFY): run focused tests, format/lint checks, and complete AC
   evidence.

## TDD Evidence
### RED
- `cargo test -p tau-tui regression_spec_c06_agent_mode_defaults_to_gpt5_baseline`
  - pre-implementation failure: expected `openai/gpt-5.2`, got
    `openai/gpt-4o-mini`.

### GREEN
- `cargo test -p tau-tui` passed.
- `bash scripts/run/test-tau-unified.sh` passed.
- `cargo test -p tau-cli --lib` passed.

### REGRESSION
- `cargo fmt --check` passed.
- `cargo clippy -p tau-tui -p tau-cli -- -D warnings` passed.
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-tui`; `cargo test -p tau-cli --lib` |  |
| Property | N/A |  | No randomized invariants introduced |
| Contract/DbC | N/A |  | No new contracts annotations |
| Snapshot | N/A |  | No snapshot surface changed |
| Functional | ✅ | `cargo test -p tau-tui`; launcher behavior via script tests |  |
| Conformance | ✅ | `tau-tui` agent default model test + launcher contract script |  |
| Integration | ✅ | `bash scripts/run/test-tau-unified.sh` runner-hook lifecycle flow |  |
| Fuzz | N/A |  | No untrusted-input parser changes |
| Mutation | N/A |  | Default-config/docs scope; no critical algorithm path |
| Regression | ✅ | default-model parse regression + syntax/lint checks |  |
| Performance | N/A |  | No hotspot behavior changes |
