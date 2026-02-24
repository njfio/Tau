# Spec: Issue #3539 - M320 TUI-driven interactive agent flow

Status: Implemented

## Problem Statement
`tau-unified tui` currently launches `tau-tui shell-live`, which only renders
dashboard artifacts. Operators cannot type prompts and execute agent turns from
this path, so the program is not fully integrated around one actionable TUI
entrypoint.

## Scope
In scope:
- Add a `tau-tui` command mode that launches `tau-coding-agent` interactive
  runtime using inherited terminal IO.
- Keep a dashboard shell render at entry for operator context before handoff to
  interactive runtime.
- Update `scripts/run/tau-unified.sh tui` so default behavior launches the new
  interactive mode.
- Preserve live dashboard watch mode as an explicit launcher option.
- Add deterministic tests for mode parsing, command construction, and launcher
  routing.
- Update README/operator docs to reflect the integrated usage path.

Out of scope:
- New agent model capabilities.
- Dashboard schema redesign.
- Webchat or gateway API contract changes.

## Acceptance Criteria
### AC-1 TUI exposes an interactive agent mode
Given a terminal operator invokes `tau-tui` in agent mode,
when mode arguments are valid,
then the command renders operator context and starts `tau-coding-agent`
interactive runtime with inherited stdin/stdout/stderr.

### AC-2 Unified launcher defaults `tui` to agent-interactive mode
Given `scripts/run/tau-unified.sh tui` is invoked,
when no mode override is supplied,
then launcher executes the new `tau-tui` agent mode.

### AC-3 Launcher still supports explicit dashboard watch mode
Given an operator needs read-only dashboard watch output,
when the launcher receives explicit live-shell mode selection,
then it runs `tau-tui shell-live` with watch controls.

### AC-4 Conformance tests enforce fail-closed parsing/routing
Given malformed flags or incompatible combinations,
when `tau-tui` and launcher parsers run,
then commands fail closed with clear diagnostics and deterministic tests.

### AC-5 Docs describe the unified interactive workflow
Given operator docs are reviewed,
when following the quick-start path,
then they show that `tau-unified tui` is agent-interactive by default and how
to access live dashboard watch mode explicitly.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | `tau-tui` agent args | parse + build command | interactive agent runner command is deterministic |
| C-02 | AC-1 | Integration | test runner hook | run agent mode | command launches with inherited IO contract and expected args |
| C-03 | AC-2 | Conformance | launcher `tui` default | execute `tui` | runner/log shows agent mode path |
| C-04 | AC-3 | Conformance | launcher `tui --live-shell` | execute command | runner/log shows `shell-live` path with watch flags |
| C-05 | AC-4 | Regression | malformed mode/flags | execute parser | exits non-zero with actionable diagnostics |
| C-06 | AC-5 | Functional | README + operator docs | inspect | unified interactive and explicit live-shell guidance present |

## Success Metrics / Observable Signals
- Operators can run one command (`scripts/run/tau-unified.sh tui`) and type
  prompts immediately.
- Launcher/TUI test suites validate default interactive routing and explicit
  live-shell fallback.
- Docs no longer describe `tui` as observability-only.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `cargo test -p tau-tui` (includes `conformance_tui_agent_mode_dry_run_emits_interactive_launch_contract` + new `main.rs` parse/command contract tests); `tau-tui agent --dry-run` now emits deterministic interactive launch command for `tau-coding-agent`. |
| AC-2 | ✅ | `bash scripts/run/test-tau-unified.sh` validates default `tui` route logs agent mode path and launcher output marker `tau-unified: launching tui (agent)`. |
| AC-3 | ✅ | `bash scripts/run/test-tau-unified.sh` validates `tui --live-shell --iterations ... --interval-ms ...` routes to explicit live-shell mode marker `tau-unified: launching tui (live-shell)`. |
| AC-4 | ✅ | `cargo test -p tau-tui` includes regression parsing checks for invalid/missing agent args; launcher enforces fail-closed `--iterations/--interval-ms require --live-shell` and passes contract script assertions. |
| AC-5 | ✅ | `README.md` and `docs/guides/operator-deployment-guide.md` now document `tau-unified.sh tui` interactive default and explicit `--live-shell` fallback. |
