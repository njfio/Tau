# Spec: Issue #3474 - M304 tau-tui shell-live watch mode

Status: Implemented

## Problem Statement
`tau-tui shell-live` currently renders one frame and exits. Operators monitoring
runtime artifacts need deterministic refresh cycles without re-running the
command repeatedly, and they need explicit watch-cycle markers in output so logs
can be interpreted reliably.

## Scope
In scope:
- Add watch-mode CLI support for `shell-live`:
  - `--watch` toggle
  - `--iterations <N>`
  - `--interval-ms <N>`
- Validate watch-mode arguments fail closed on invalid values.
- Emit deterministic watch-cycle markers in shell-live output.
- Add parser/output tests for watch-mode contracts.
- Update README with watch-mode operator command.

Out of scope:
- Full-screen curses UI/event loop.
- Keyboard input controls beyond existing command flags.
- Dashboard protocol/schema changes.

## Acceptance Criteria
### AC-1 shell-live watch flags parse and validate deterministically
Given CLI invocation of `tau-tui shell-live`,
when watch flags are provided,
then parser accepts valid combinations and rejects invalid values (for example
`--iterations 0`) with deterministic errors.

### AC-2 watch mode renders deterministic cycle markers
Given `shell-live --watch`,
when render cycles execute,
then output includes deterministic marker lines with cycle index/total and
refresh interval metadata.

### AC-3 one-shot shell-live behavior remains preserved
Given `shell-live` without `--watch`,
when command executes,
then it renders one live frame as before without requiring watch flags.

### AC-4 README documents watch mode workflow
Given operator command examples in README,
when live TUI commands are reviewed,
then watch-mode invocation is explicitly documented.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | `shell-live` args with watch flags | parse args | valid watch args are accepted |
| C-02 | AC-1 | Regression | `--iterations 0` | parse args | parser rejects with deterministic error |
| C-03 | AC-2 | Functional | watch marker helper | build marker line | cycle/total/interval markers render deterministically |
| C-04 | AC-3 | Integration | `shell-live` without watch | run path selection | single frame render path remains default |
| C-05 | AC-4 | Functional | README live TUI section | inspect docs | watch command is present |

## Success Metrics / Observable Signals
- Operators can run deterministic multi-cycle live shell monitoring.
- Watch output markers are stable for triage and CI logs.
- README reflects one-shot and watch-mode TUI workflows.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui 3474 -- --nocapture` passes parser contracts for watch flags and fail-closed `--iterations 0` validation. |
| AC-2 | ✅ | `CARGO_TARGET_DIR=target-fast cargo run -p tau-tui -- shell-live --state-dir .tau/dashboard --profile local-dev --watch --iterations 1 --interval-ms 0 --no-color` emits deterministic marker line `watch.cycle=1/1 watch.interval_ms=0 watch.diff_ops=...`. |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo run -p tau-tui -- shell-live --state-dir .tau/dashboard --profile local-dev --no-color` preserves one-shot path (`Tau Operator Shell (live) ...` header), and `spec_c03_parse_args_accepts_shell_live_mode_and_state_dir` remains green. |
| AC-4 | ✅ | `README.md` now includes explicit live watch-mode commands in Quickstart and Common Workflows sections. |
