# Plan #2572

## Approach
1. Identify stable insertion point in `request_messages` compaction flow to persist compaction entries.
2. Add deterministic summary-to-memory extraction helper and route outputs through current memory-save path.
3. Gate extraction to warn/aggressive tiers only; keep emergency path unchanged.
4. Add RED conformance/regression tests before implementation changes.
5. Implement minimal behavior needed for ACs, then refactor for readability and deterministic failure handling.

## Affected Modules
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/runtime_turn_loop.rs`
- `crates/tau-agent-core/src/tests/structured_output_and_parallel.rs`
- Potentially existing memory runtime adapters invoked by `tau-agent-core`

## Risks & Mitigations
- Risk: memory-save wiring introduces side effects in hot request path.
  - Mitigation: keep fail-safe error handling and deterministic no-panic behavior.
- Risk: compaction entry persistence duplicates summaries excessively.
  - Mitigation: add deterministic insertion guards and regression tests.

## Interfaces / Contracts
- Internal request-shaping and memory-save wiring contracts only.
- No external API or wire-format changes expected.
