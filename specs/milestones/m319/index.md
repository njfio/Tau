# M319 - Unified one-command runtime entrypoint wave

Status: Active

## Context
Operators currently have multiple runbook steps to launch and manage runtime
surfaces. M319 delivers a single command entrypoint that starts/stops/reports
the integrated gateway/dashboard runtime and optionally launches live TUI from
the same command surface.

Primary sources:
- `docs/guides/operator-deployment-guide.md`
- `scripts/demo/index.sh`
- `README.md`

## Issue Hierarchy
- Epic: #3533
- Story: #3534
- Task: #3535

## Scope
- Add unified runtime launcher script with `up`, `status`, `down`, and
  optional `tui` execution path.
- Add deterministic launcher contract test suite.
- Persist PID/log/runtime metadata for lifecycle operations.
- Update README and operator docs with one-command entrypoint guidance.

## Exit Criteria
- `specs/3535/spec.md` is `Implemented` with AC evidence.
- One command launches integrated runtime and exposes lifecycle controls.
- Contract tests validate lifecycle and fail-closed argument handling.
- README/operator docs include the unified command path.
