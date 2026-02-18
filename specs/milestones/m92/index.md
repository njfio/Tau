# M92 - Spacebot G15 Profile Routing Closure (Phase 2)

Milestone objective: close the remaining G15 profile-routing scope in `tasks/spacebot-comparison.md` by adding process-level routing config, task overrides, prompt complexity scoring, and dispatch-time model selection.

## Scope
- Add profile routing schema fields for process models (`channel`, `branch`, `worker`, `compactor`, `cortex`).
- Add task override mapping (`coding`, `summarization`) and deterministic complexity scoring (`light`, `standard`, `heavy`).
- Apply overrides at prompt dispatch with scoped model restoration.
- Add conformance/regression tests and verification evidence package.

## Out of Scope
- New provider integrations.
- Non-model routing architecture changes.
- UI/dashboard work.

## Exit Criteria
- Task `#2536` and subtask `#2537` merged with AC mappings green.
- `tasks/spacebot-comparison.md` G15 checklist updated to reflect completed pathway.
- Verification gates pass: fmt, clippy, scoped tests, full `cargo test`, mutation in diff, live validation.
