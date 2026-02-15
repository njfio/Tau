# Issue 1654 Plan

Status: Reviewed

## Approach

1. Produce baseline hotspot artifacts for undocumented public APIs in
   `tau-multi-channel`, `tau-gateway`, `tau-provider`, and `tau-ops`.
2. Add rustdoc comment blocks to undocumented public items in those crates.
3. Re-run hotspot scan to prove post-change undocumented counts in scope.
4. Re-run marker count, threshold verify, and ratchet check artifacts.
5. Run crate-level tests for touched crates to validate regressions.

## Affected Areas

- `crates/tau-multi-channel/src/**`
- `crates/tau-gateway/src/**`
- `crates/tau-provider/src/**`
- `crates/tau-ops/src/**`
- `tasks/reports/m23-wave2-undocumented-hotspots-*.{json,md}`
- `tasks/reports/m23-rustdoc-marker-count.{json,md}`
- `tasks/reports/m23-rustdoc-marker-threshold-verify.{json,md}`
- `tasks/reports/m23-rustdoc-marker-ratchet-check.{json,md}`

## Risks And Mitigations

- Risk: low-signal “comment spam”.
  - Mitigation: use consistent contract-focused templates and preserve symbol
    context in comments.
- Risk: merge conflicts across high-churn crates.
  - Mitigation: keep edits scoped to doc comments only and avoid logic changes.
- Risk: long test cycle.
  - Mitigation: run crate-scoped tests in parallel first, then rerun failed-only
    as needed.

## ADR

No architecture/dependency/protocol decision change. ADR not required.
