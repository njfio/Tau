# M323 - Unified TUI bootstrap and auth launch UX

Status: Active

## Context
`scripts/run/tau-unified.sh tui` currently launches an agent shell even when no
dashboard runtime artifacts exist, which makes the top panels look broken
(`state.json`/`auth-status.json` missing). In parallel, OpenAI auth launch flow
uses `codex --login`, which is incompatible with current Codex CLI syntax.

## Issue Hierarchy
- Epic: #3549
- Story: #3550
- Task: #3551

## Scope
- Auto-bootstrap unified runtime when `tui` is launched and runtime is not
  running, with an explicit opt-out.
- Keep runner test mode deterministic and avoid auto-bootstrap side effects in
  script tests.
- Update OpenAI auth launch command to use `codex login`.

## Exit Criteria
- `specs/3551/spec.md` is `Implemented` with AC verification evidence.
- `tau-unified.sh tui` can launch with live runtime artifacts without requiring
  manual `up`.
- `/auth login openai --mode oauth-token --launch` uses `codex login`.
