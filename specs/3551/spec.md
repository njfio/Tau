# Spec: Issue #3551 - Unified TUI runtime bootstrap + codex login command compatibility

Status: Implemented

## Problem Statement
Operators launching `./scripts/run/tau-unified.sh tui` see empty dashboard
panels when runtime artifacts are missing, making the UI feel disconnected.
Also, OpenAI auth launch currently tries `codex --login`, which fails with
current Codex CLI that expects `codex login`.

## Scope
In scope:
- Add runtime bootstrap behavior to `tau-unified.sh tui` when runtime is not
  active (default enabled, explicit opt-out).
- Preserve deterministic behavior for runner-based launcher tests.
- Update OpenAI auth launch command construction from `--login` to `login`.
- Update/extend tests covering launcher and auth launch command behavior.

Out of scope:
- Full TUI UI redesign.
- Dashboard rendering architecture changes.
- Provider auth model redesign beyond launch command compatibility.

## Acceptance Criteria
### AC-1 TUI launches with integrated runtime state by default
Given `tau-unified.sh tui` is invoked and no unified runtime is active,
when launch occurs without opt-out,
then runtime is started automatically before TUI execution.

### AC-2 Operators can disable auto-bootstrap explicitly
Given operators want current manual behavior,
when invoking `tau-unified.sh tui --no-bootstrap-runtime`,
then no runtime bootstrap is attempted.

### AC-3 OpenAI auth launch uses current Codex CLI syntax
Given `/auth login openai --mode oauth-token --launch`,
when command launch spec is rendered/executed,
then command shape is `codex login` (or `<codex-cli> login`).

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | runtime stopped | run `tau-unified.sh tui` | launcher bootstraps runtime first |
| C-02 | AC-2 | Unit | explicit opt-out flag | run `tau-unified.sh tui --no-bootstrap-runtime` | no bootstrap path executed |
| C-03 | AC-3 | Unit/Functional | auth launch command build + mock codex script | execute auth login launch | launch args equal `login` |

## Success Metrics / Observable Signals
- First-run `tui` launch no longer defaults to empty/missing artifact panel path.
- Auth launch no longer fails on Codex CLI argument parsing due `--login`.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `scripts/run/tau-unified.sh` now bootstraps runtime for `tui` via `bootstrap_runtime_for_tui` and waits for initial dashboard artifacts. Verified by `bash scripts/run/test-tau-unified.sh` bootstrap assertions. |
| AC-2 | ✅ | New CLI flags `--bootstrap-runtime` and `--no-bootstrap-runtime` added. Test validates default runner mode does not auto-bootstrap while explicit bootstrap path does. |
| AC-3 | ✅ | OpenAI auth launch spec now emits `login` subcommand (`codex login`). Verified by `cargo test -p tau-coding-agent functional_execute_auth_command_login_openai_launch_executes_codex_login_command` and `cargo test -p tau-provider`. |
