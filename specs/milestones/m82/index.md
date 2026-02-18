# M82 - Spacebot G17 Prompt Templates (Phase 3)

## Context
`tasks/spacebot-comparison.md` identifies G17 gaps around template engine parity and operator-editable prompt semantics. M80/M81 delivered workspace template loading plus builtin/default fallback diagnostics in `tau-onboarding`.

M82 delivers the next bounded slice: migrate startup template rendering to `minijinja` and add Spacebot-style alias variables that are safe for startup context.

## Scope
- Add `minijinja` as workspace dependency and use it in startup prompt composition.
- Replace manual placeholder parser in `crates/tau-onboarding/src/startup_prompt_composition.rs`.
- Support startup-safe aliases: `identity`, `tools`, `memory_bulletin`, `active_workers`.
- Preserve fail-closed fallback semantics and template source diagnostics.
- Conformance + regression coverage and RED/GREEN evidence.

## Out of Scope
- Runtime prompt hot-reload watchers (G16 coupling).
- Full `tau-agent-core` process prompt template migration.
- New runtime bulletin generation or worker orchestration features.

## Issue Hierarchy
- Epic: #2480
- Story: #2481
- Task: #2482
- Subtask: #2483

## Verification Targets
- `cargo test -p tau-onboarding -- spec_2482`
- `cargo test -p tau-onboarding -- startup_prompt_composition`
- `cargo clippy -p tau-onboarding -- -D warnings`
- `cargo mutants -p tau-onboarding --in-diff`
- `./scripts/demo/local.sh --skip-build --binary target/debug/tau-coding-agent --timeout-seconds 120`
