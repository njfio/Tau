# Plan: Issue #2929 - Refactor gateway_openresponses into maintainable submodules

1. Baseline current state:
   - capture `gateway_openresponses.rs` line count,
   - inspect current route wiring,
   - inspect oversized-file exemption artifact.
2. Extract dashboard runtime handlers into `gateway_openresponses/dashboard_runtime.rs`:
   - move authorization helper + dashboard route handlers,
   - keep signatures and response payloads stable,
   - import handlers into parent module for route wiring.
3. Extract memory runtime handlers/helpers into `gateway_openresponses/memory_runtime.rs`:
   - move memory CRUD/graph endpoint handlers,
   - move memory graph/storage helper functions,
   - preserve shared helper access for `ops_dashboard_shell.rs`.
4. Update oversized-file policy artifact to reflect resolved split debt.
5. Verify with targeted gateway tests and quality gates.

## Risks / Mitigations
- Risk: visibility/import breakages across submodules.
  - Mitigation: keep moved functions `pub(super)` and preserve parent `use` bindings.
- Risk: subtle JSON payload drift.
  - Mitigation: no behavior edits; run existing gateway endpoint regression tests.
- Risk: policy artifact mismatch with line-count reality.
  - Mitigation: explicitly validate `wc -l` + policy content in verification step.

## Interface / Contract Notes
- No public API/wire-format changes intended.
- This is an internal module-boundary refactor with policy hygiene updates.
