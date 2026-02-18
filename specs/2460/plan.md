# Plan #2460

## Approach

1. Add RED tests for duplicate lifecycle behavior and heartbeat integration.
2. Implement duplicate detection in `run_lifecycle_maintenance` using cosine
   similarity over active record embeddings with deterministic tie-breakers.
3. Add heartbeat lifecycle config + execution path, reporting counters and
   reason codes.
4. Run scoped verify gates and mutation in diff.

## Affected Modules

- `crates/tau-memory/src/runtime.rs`
- `crates/tau-memory/src/runtime/query.rs`
- `crates/tau-runtime/src/heartbeat_runtime.rs`
- `crates/tau-onboarding/src/startup_transport_modes.rs`
- tests in touched modules

## Risks

- False-positive duplicate detection.
  - Mitigation: configurable threshold + deterministic canonical rules.
- Runtime heartbeat regression.
  - Mitigation: focused integration/regression tests.
