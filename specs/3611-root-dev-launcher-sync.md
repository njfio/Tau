# 3611-root-dev-launcher-sync

## Objective
Align the root repo developer entrypoints with the current Codex-backed interactive runtime so the root checkout is the authoritative path for starting, restarting, and testing the Tau TUI stack.

## Inputs/Outputs
- Inputs:
  - `scripts/run/tau-unified.sh` commands: `up`, `down`, `status`, `tui`
  - `cargo run -p tau-tui -- agent --dry-run --no-color`
  - root-level `just` recipes
- Outputs:
  - root launcher defaults to the current Codex model and current timeout/retry policy
  - root dry-run emits the updated runtime command
  - root `justfile` exposes stack control and fresh-session commands

## Boundaries/Non-goals
- Non-goals:
  - TUI visual redesign
  - provider/runtime algorithm changes
  - deleting or rewriting stale side worktrees
- Boundary:
  - only root developer entrypoints, defaults, and their tests are in scope

## Failure modes
- Launcher still emits `openai/gpt-5.2`
- Launcher keeps stale timeout/retry defaults
- `just` recipes call missing files or use invalid Just syntax
- Fresh-session commands fail to reset `.tau/gateway/openresponses/sessions/default.jsonl`
- Dry-run/help output diverges from actual runtime command construction

## Acceptance criteria
- [ ] `scripts/run/tau-unified.sh` defaults `up`/`tui` to `gpt-5.3-codex`
- [ ] `scripts/run/tau-unified.sh` defaults `tui` request timeout to `180000` and agent retries to `0`
- [ ] `cargo run -p tau-tui -- agent --dry-run --no-color` emits `--model gpt-5.3-codex`
- [ ] root `justfile` exposes `stack-up`, `stack-up-fast`, `stack-down`, `restart-stack`, `rebuild`, `tui`, `session-reset`, `stack-up-fresh`, `tui-fresh`
- [ ] at least one real integration test covers the launcher/just entrypoint contract
- [ ] failures in copy/start/reset paths fail loudly with non-zero exit and visible output

## Files to touch
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `crates/tau-tui/src/main.rs`
- `justfile`
- `scripts/dev/test-root-just-launcher.sh`

## Error semantics
- Invalid `just` command composition or missing root files must hard-fail
- Invalid launcher flag values must keep existing `die` behavior
- Fresh-session reset must fail loudly if the target session directory cannot be reset

## Test plan
- Red:
  - failing root launcher expectations for model/timeouts in `scripts/run/test-tau-unified.sh`
  - failing dry-run/unit expectations in `crates/tau-tui/src/main.rs`
  - failing root `justfile` contract test in `scripts/dev/test-root-just-launcher.sh`
- Green:
  - update launcher defaults and dry-run defaults
  - add root `justfile` and fresh-session wiring
- Refactor:
  - centralize shared default values where practical
- Integration:
  - run `scripts/run/test-tau-unified.sh`
  - run `scripts/dev/test-root-just-launcher.sh`
  - run `cargo test -p tau-tui`
  - smoke `just stack-up-fast` / `just stack-down` from root
