# Tasks: Issue #3474 - M304 tau-tui shell-live watch mode

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance/Regression): add failing parser/output tests for
   watch-mode args and deterministic marker contracts.
2. [x] T2 (GREEN, Implementation): implement `shell-live` watch args and
   execution loop with cycle marker output.
3. [x] T3 (GREEN, Docs): update README live TUI command examples with watch mode.
4. [x] T4 (VERIFY): run scoped tau-tui tests and formatting checks, then update
   spec/task status to `Implemented`.

## Tier Mapping
| Tier | Planned Coverage |
| --- | --- |
| Unit | parser + watch marker helper tests in `tau-tui` |
| Property | N/A (no randomized invariant surface) |
| Contract/DbC | N/A (no DbC annotation changes) |
| Snapshot | N/A |
| Functional | watch marker contract rendering |
| Conformance | C-01/C-02 parser contracts |
| Integration | shell-live one-shot and watch path selection |
| Fuzz | N/A (no parser beyond bounded CLI flags) |
| Mutation | N/A (non-critical CLI UX slice) |
| Regression | explicit invalid `--iterations 0` fail-closed test |
| Performance | N/A (no benchmark/hot-path scope) |

## TDD Evidence
### RED
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui 3474 -- --nocapture`
- Expected failure before implementation:
  - `expected watch-mode parse success: "unknown argument: --watch"`
  - `assertion failed: HELP.contains("--watch")`
  - `assertion failed: err.contains("--iterations must be >= 1")`

### GREEN
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui 3474 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo run -p tau-tui -- shell-live --state-dir .tau/dashboard --profile local-dev --watch --iterations 1 --interval-ms 0 --no-color` emitted deterministic watch marker:
  - `watch.cycle=1/1 watch.interval_ms=0 watch.diff_ops=...`

### REGRESSION
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-tui --tests -- -D warnings` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | marker helper + parser validation tests in `crates/tau-tui/src/main.rs` |  |
| Property | N/A |  | No randomized invariant surface introduced |
| Contract/DbC | N/A |  | No DbC annotations changed |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `functional_spec_3474_c03_live_watch_marker_contract_is_deterministic`; watch-mode run output marker check |  |
| Conformance | ✅ | `integration_spec_3474_c01_*`, `regression_spec_3474_c02_*` |  |
| Integration | ✅ | `cargo test -p tau-tui -- --nocapture` and shell-live one-shot/watch runs |  |
| Fuzz | N/A |  | No new untrusted parser/runtime surface requiring fuzz campaign |
| Mutation | N/A |  | Non-critical CLI UX slice |
| Regression | ✅ | `regression_spec_3474_c02_parse_args_rejects_shell_live_zero_iterations` |  |
| Performance | N/A |  | No hotspot path changes |
