# Plan #2449

Status: Reviewed
Spec: specs/2449/spec.md

## Approach

1. Add RED tests for lifecycle defaults, forgotten exclusion, and delete tool.
2. Extend `RuntimeMemoryRecord` with lifecycle metadata.
3. Add runtime soft-delete method and touch-on-read/search behavior.
4. Add `memory_delete` tool and register in built-in catalog.
5. Run verify gates for touched crates.

## Affected Modules

- `crates/tau-memory/src/runtime.rs`
- `crates/tau-memory/src/runtime/query.rs`
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/memory_tools.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-tools/src/tools/tests.rs`

## Risks and Mitigations

- Risk: lifecycle writes on read/search introduce extra storage churn.
  - Mitigation: append-only updates only for returned records; deterministic
    tests for idempotent defaults.
- Risk: forgotten filtering might hide expected data.
  - Mitigation: explicit tests proving active records remain visible while
    forgotten ones are filtered.
