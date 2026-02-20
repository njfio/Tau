# Plan: Issue #2858 - chat/sessions panel visibility-state contracts

## Approach
1. Add explicit `data-panel-visible` attributes derived from existing route visibility booleans in `tau-dashboard-ui`.
2. Add UI and gateway tests for deterministic visibility-state combinations on `/ops`, `/ops/chat`, and `/ops/sessions`.
3. Re-run chat/sessions/command-center regression suites to verify no behavior drift.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: marker string ordering drift may cause brittle assertions.
  - Mitigation: keep deterministic attribute ordering and assert stable marker snippets.
- Risk: accidental visibility behavior regressions.
  - Mitigation: targeted regression reruns for `spec_2830`, `spec_2838`, `spec_2854`.

## Interface / Contract Notes
- Additive SSR marker attribute change only.
- No endpoint/schema/transport changes.
