# M66 - Spacebot G15 Process-Type Model Routing (Phase 1)

Milestone objective: deliver role-profile model routing for plan-first orchestrator attempts so
planner/delegated/review phases can dispatch to distinct models defined in route-table role
profiles, while preserving legacy behavior when no role model override is configured.

## Scope
- Respect `roles.<name>.model` in multi-agent route tables during routed prompt dispatch.
- Apply role model overrides only for the active routed attempt.
- Restore baseline agent model after each routed attempt (success/failure/cancellation).
- Add conformance tests validating dispatch ordering and inherit behavior.

## Out of Scope
- New profile TOML `routing` schema (`channel_model`, `worker_model`, etc.).
- Prompt complexity scoring or task-classifier heuristics.
- Non-orchestrator runtime model routing (channel/voice/background jobs).

## Exit Criteria
- Issue `#2398` AC/C-case mapping implemented.
- Targeted tests pass for `tau-agent-core`, `tau-orchestrator`, and `tau-coding-agent`.
- Parent hierarchy (`#2396` -> `#2399`) is closed with status labels updated.
