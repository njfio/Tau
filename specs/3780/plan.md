# Plan 3780: Keep All Verification Gates Visible

## Approach

Add a focused conformance test for the gate-list visibility contract. Mark the
verification-gate section with all-gates-first-viewport metadata, then switch
only that list to a compact two-column layout with tighter chips. Keep the
existing gate order, statuses, memory footer, and artifact footer unchanged.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks and Mitigations

- Risk: Two-column gate chips may make labels harder to scan.
  Mitigation: Use this only for the compact proof card and keep IDs/status
  markers intact for unambiguous proof state.
- Risk: Proof footer layout may regress.
  Mitigation: Preserve the existing DOM order and rerun the 3770-3780 harness
  test set plus browser geometry checks.

## Interfaces

No route, endpoint, schema, gate status, proof source, memory, or artifact
semantics change. This is a dashboard layout and marker contract only.
