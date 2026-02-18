# Plan #2459

## Approach

1. Extend `tau-memory` lifecycle policy/result to include duplicate-handling
   controls and counters.
2. Add deterministic duplicate canonicalization in maintenance pass.
3. Extend runtime heartbeat config/execution to run lifecycle maintenance
   against configured memory store root(s).
4. Add conformance/regression tests for duplicate + heartbeat behavior.

## Risks

- Duplicate canonicalization instability across runs.
  - Mitigation: deterministic ordering and explicit tie-break rules.
- Heartbeat maintenance invocation may fail on missing/corrupt stores.
  - Mitigation: convert failures into diagnostics/reason codes, continue cycle.

## Interfaces / Contracts

- `MemoryLifecycleMaintenancePolicy` (extended duplicate controls)
- `MemoryLifecycleMaintenanceResult` (extended duplicate counters)
- `RuntimeHeartbeatSchedulerConfig` (lifecycle maintenance config fields)

## ADR

No ADR required for this scoped runtime integration and policy extension.
