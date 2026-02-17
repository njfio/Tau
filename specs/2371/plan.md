# Issue 2371 Plan — Typed Memory + Importance Foundation

## Approach

1. Introduce a `MemoryType` enum and default importance mapping in
   `tau-memory` runtime types.
2. Extend persisted `RuntimeMemoryRecord` with `memory_type` and `importance`
   fields using serde defaults for backward compatibility.
3. Add an explicit write path accepting optional type/importance while keeping
   current write API behavior for existing call sites.
4. Extend `memory_write` argument parsing and validation to accept the new
   fields.
5. Include metadata in tool outputs and apply an importance multiplier in search
   ranking.
6. Add conformance tests first, then implementation, then regression checks.

## Affected Modules

- `crates/tau-memory/src/runtime.rs`
  - `MemoryType` enum, record fields, write methods.
- `crates/tau-memory/src/runtime/query.rs`
  - importance-aware ranking score adjustments.
- `crates/tau-tools/src/tools/memory_tools.rs`
  - argument schema + parsing + output fields.
- `crates/tau-tools/src/tools/tests.rs`
  - conformance tests C-01..C-05.

## Risks and Mitigations

- Risk: ranking behavior regressions.
  - Mitigation: targeted conformance ordering test (C-04) plus existing
    regression search tests.
- Risk: compatibility break for existing serialized records.
  - Mitigation: serde default fields and explicit backward compatibility test
    (C-05).
- Risk: invalid importance values causing silent clamping.
  - Mitigation: strict validation in tool layer with deterministic error output
    (C-02).

## Interfaces / Contracts

- `memory_write` accepts:
  - `memory_type: string` (enum values above)
  - `importance: number` (optional, `0.0..=1.0`)
- `memory_write`, `memory_read`, `memory_search.matches[]` emit:
  - `memory_type`
  - `importance`

## ADR

No ADR required for this scoped enhancement (no new dependency, no protocol
break).
