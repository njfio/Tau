# Plan #2450

Status: Reviewed
Spec: specs/2450/spec.md

## Approach

1. Add RED tests for lifecycle defaults, access metadata touch, and forgotten
   exclusion after soft delete.
2. Extend runtime record schema and write path defaults.
3. Implement `read_entry`/`search` lifecycle touch updates and forgotten
   filtering in default flows.
4. Add runtime soft-delete entry method.
5. Add `memory_delete` tool, register it, and add conformance tests.
6. Run verify gates and capture AC->test mapping.

## Affected Modules

- `crates/tau-memory/src/runtime.rs`
- `crates/tau-memory/src/runtime/query.rs`
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/memory_tools.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-tools/src/tools/tests.rs`

## Risks and Mitigations

- Risk: touching records on read/search creates excess append records.
  - Mitigation: touch only returned memory ids; saturating counter increments.
- Risk: forgotten filtering regresses existing behavior unexpectedly.
  - Mitigation: add regression tests confirming active records still returned.
