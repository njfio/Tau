# Plan 3776: Keep Proof Memory And Artifacts Visible

## Approach

Add a compact local scroll budget to the Acceptance Criteria and Verification
Gates panels. Keep all items rendered, keep proof ordering unchanged, and add
explicit first-viewport footer markers to Memory / Learning and Artifacts so the
contract is deterministic in tests.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks and Mitigations

- Risk: Compressing acceptance and gate panels could hide some secondary items.
  Mitigation: Use local scrolling and keep all list items in the DOM.
- Risk: Existing proof-priority tests could become stale.
  Mitigation: Keep ordering unchanged and run the 3767-3776 proof-focused test
  range.

## Interfaces

No route, endpoint, schema, proof artifact, or verification semantic changes.
This is a dashboard layout and marker contract only.
