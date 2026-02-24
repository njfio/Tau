# Tasks: Issue #3557 - Interactive in-flight progress indicator for TUI agent turns

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED): add failing tests for progress indicator contract and non-tty
   output cleanliness.
2. [x] T2 (GREEN): implement TTY-gated interactive turn progress tracker in
   runtime loop.
3. [x] T3 (GREEN): wire status mapping for completed/cancelled/timed-out/failed.
4. [x] T4 (VERIFY): run targeted runtime-loop + REPL harness tests.
5. [x] T5 (DOC): update operator docs and mark spec artifacts Implemented.

## Test Tier Intent
| Tier | Planned |
| --- | --- |
| Unit | runtime-loop helper/status tests |
| Functional | interactive path progress emission behavior |
| Integration | REPL harness regression for non-tty cleanliness |
| Conformance | C-01..C-04 mapping |
| Regression | non-tty no-noise guarantee |

## TDD Evidence
### RED
- `cargo test -p tau-coding-agent unit_interactive_turn_progress_is_enabled_only_for_tty_streams -- --nocapture`
  - failed initially with unresolved helper imports:
    `no should_emit_interactive_turn_progress in runtime_loop`

### GREEN
- `cargo test -p tau-coding-agent interactive_turn_progress -- --nocapture` passed.
- `cargo test -p tau-coding-agent integration_repl_harness_executes_prompt_flow_with_mock_openai -- --nocapture` passed.
- `TAU_REPL_HARNESS_TIMEOUT_MS=7000 cargo test -p tau-coding-agent regression_repl_harness_turn_timeout_remains_deterministic_in_ci -- --nocapture` passed.

### REGRESSION
- Updated non-tty REPL fixtures to assert stderr omits `interactive.turn=`.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `interactive_turn_progress` runtime-loop unit tests |  |
| Property | N/A |  | No property/invariant logic changed |
| Contract/DbC | N/A |  | No contract macro surfaces changed |
| Snapshot | N/A |  | No snapshot fixtures involved |
| Functional | ✅ | interactive runtime-loop progress contract tests |  |
| Conformance | ✅ | C-01..C-04 mapped via runtime-loop + REPL harness |  |
| Integration | ✅ | REPL harness prompt + timeout fixtures |  |
| Fuzz | N/A |  | No parser/input fuzz scope changed |
| Mutation | N/A |  | UX observability scope; no critical mutation gate required |
| Regression | ✅ | non-tty fixture `stderr_not_contains: interactive.turn=` |  |
| Performance | N/A |  | No benchmarked hotspot changes |
