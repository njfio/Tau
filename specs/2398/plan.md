# Plan: Issue #2398 - Apply role model overrides at orchestrator dispatch time

## Approach
1. Add a scoped model override helper to `tau-agent-core::Agent` that temporarily swaps
   the active dispatch model during an async closure and restores baseline afterward.
2. Extend orchestrator runtime adapter contract to accept per-attempt model override hints.
3. Pass `role_profile.model` from routed attempt execution into the adapter dispatch call.
4. Add failing conformance tests first in coding-agent orchestrator routing tests and agent-core
   scoped helper tests, then implement minimal production changes.
5. Run targeted verification (`fmt`, `clippy`, scoped tests) and mutation-in-diff.

## Affected Modules
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-orchestrator/src/orchestrator.rs`
- `crates/tau-coding-agent/src/orchestrator_bridge.rs`
- `crates/tau-coding-agent/src/tests.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`

## Risks and Mitigations
- Risk: model override leaks into subsequent prompts.
  - Mitigation: scoped helper restores baseline model in all outcomes; dedicated regression test.
- Risk: trait signature updates break orchestrator test harnesses.
  - Mitigation: update all runtime adapter implementations in same commit with compile-driven checks.
- Risk: fallback semantics accidentally change while plumbing model hints.
  - Mitigation: keep route selection untouched and preserve existing fallback regression tests.

## Interface/Contract Notes
- Role profile `model` field remains optional and unchanged schema-wise.
- Orchestrator runtime prompt execution contract gains optional model override for routed attempts.
- No wire-format, profile-TOML, or persistence schema changes in this slice.

## ADR
- Not required: no dependency introduction and no protocol/schema redesign.
