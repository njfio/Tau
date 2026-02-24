# Tasks: Issue #3448 - M298 wave-1 E2E harness and ops dashboard conformance slice

Status: Reviewed

## Ordered Tasks
1. [ ] T1 (RED, Conformance): add failing artifact/contract checks for wave-1 AC mapping and harness/dashboard expectations.
2. [ ] T2 (GREEN, Spec/Docs): finalize artifact consistency and conformance mapping updates.
3. [ ] T3 (RED, Integration): add failing E2E tests for gateway lifecycle + agent session flow using deterministic scripted LLM.
4. [ ] T4 (GREEN, Implementation): implement wave-1 harness helpers and minimal runtime wiring to satisfy T3.
5. [ ] T5 (RED, Functional/Integration): add failing ops dashboard live control/data conformance regressions.
6. [ ] T6 (GREEN, Implementation): implement/fix dashboard/gateway behavior for live conformance.
7. [ ] T7 (VERIFY, Regression): execute scoped fmt/clippy/tests and record RED/GREEN/REGRESSION evidence.
8. [ ] T8 (VERIFY): update spec status to `Implemented` once all ACs pass.

## Test Tier Mapping
| Tier | Planned Coverage |
| --- | --- |
| Unit | Harness helper utilities and parser/mapper edge cases |
| Property | N/A (unless new parser/invariant-heavy utility is introduced) |
| Contract/DbC | N/A (unless public API contract annotations are added) |
| Snapshot | Optional for stable structured output; not a behavior replacement |
| Functional | Dashboard live control/data behavior |
| Conformance | AC-aligned contract checks (C-01..C-04) |
| Integration | E2E harness scenarios + gateway/dashboard integration paths |
| Fuzz | N/A for wave-1 scope unless untrusted parser surface is added |
| Mutation | Run `cargo mutants --in-diff` for critical changed paths before merge |
| Regression | Targeted regressions for control/data and scenario stability |
| Performance | N/A unless wave-1 introduces hotspot regressions |
