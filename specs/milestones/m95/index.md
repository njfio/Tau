# M95 - Spacebot G8 Local Embeddings (Phase 1)

Milestone: GitHub milestone `M95 - Spacebot G8 Local Embeddings (Phase 1)`

## Objective
Deliver the next bounded `G8` slice from `tasks/spacebot-comparison.md` by enabling local memory embeddings as the default runtime policy while preserving explicit remote-provider overrides.

## Scope
- Provide local embedding provider as default tool policy/profile behavior.
- Preserve explicit remote provider (`openai` / `openai-compatible`) override paths.
- Verify behavior through conformance/regression tests and live validation evidence.

## Out of Scope
- FastEmbed runtime/model-loading implementation details beyond existing local provider wiring.
- New provider families or remote protocol changes.
- Broader memory lifecycle and retrieval enhancements outside G8.

## Issue Hierarchy
- Epic: #2551
- Story: #2552
- Task: #2553
- Task: #2556
- Subtask: #2554

## Exit Criteria
- G8 default-local profile behavior is implemented and verified.
- ACs for milestone tasks are covered by conformance tests with RED/GREEN evidence.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, scoped tests, mutation in diff, and live validation pass.
