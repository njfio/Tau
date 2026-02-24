# Tasks: Issue #3555 - Wire interactive request timeout/retry policy into local runtime + unified TUI

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED): add regression tests for local runtime timeout wiring,
   `tau-tui` timeout/retry command forwarding, and unified launcher default
   timeout/retry forwarding.
2. [x] T2 (GREEN): update local runtime timeout wiring to use CLI timeout.
3. [x] T3 (GREEN): add `tau-tui agent` timeout/retry args and launch passthrough.
4. [x] T4 (GREEN): add unified launcher timeout/retry defaults, overrides, and
   pass-through into `tau-tui agent`.
5. [x] T5 (VERIFY): run targeted shell + Rust test matrix and collect evidence.
6. [x] T6 (DOC): update README/run docs and mark spec artifacts Implemented.

## Test Tier Intent
| Tier | Planned |
| --- | --- |
| Unit | `tau-tui` parser + command builder tests |
| Functional | launcher runner-mode script tests |
| Conformance | spec case-mapped tests (`C-01`..`C-05`) |
| Integration | local runtime timeout behavior test |
| Regression | explicit timeout wiring regression case in runtime startup tests |

## TDD Evidence
### RED
- `cargo test -p tau-tui spec_c07_parse_args_accepts_agent_timeout_and_retry_overrides -- --nocapture`
  - failed with:
    `"unknown argument: --request-timeout-ms"`
- `bash scripts/run/test-tau-unified.sh`
  - failed with:
    `"expected output to contain '--request-timeout-ms 45000'"`
- `cargo test -p tau-coding-agent regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent -- --nocapture`
  - failed with:
    `"run_local_runtime should complete quickly under low request timeout: Elapsed(())"`

### GREEN
- `bash scripts/run/test-tau-unified.sh` passed.
- `cargo test -p tau-tui -- --nocapture` passed.
- `cargo test -p tau-coding-agent regression_spec_2542_c03_run_local_runtime_prompt_executes_model_call -- --nocapture` passed.
- `cargo test -p tau-coding-agent regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent -- --nocapture` passed.

### REGRESSION
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-tui -- --nocapture` |  |
| Property | N/A |  | No property/invariant logic changed |
| Contract/DbC | N/A |  | No contract macro surfaces changed |
| Snapshot | N/A |  | No snapshot fixtures involved |
| Functional | ✅ | `bash scripts/run/test-tau-unified.sh` |  |
| Conformance | ✅ | `C-01..C-05` mapped across runtime/tui/launcher/docs checks |  |
| Integration | ✅ | `cargo test -p tau-coding-agent regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent -- --nocapture` |  |
| Fuzz | N/A |  | No untrusted-parser mutation scope |
| Mutation | N/A |  | Runtime/launcher wiring task; no critical-path mutation gate requested |
| Regression | ✅ | RED->GREEN timeout wiring guard (`regression_spec_3555_c01...`) |  |
| Performance | N/A |  | No performance benchmarked hotspot in scope |
