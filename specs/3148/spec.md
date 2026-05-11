# Spec: Issue #3148 - /ops/training status and control contracts

Status: Implemented

## Problem Statement
The `/ops/training` route currently exposes only endpoint template markers. PRD section `6.11` requires deterministic route contracts for status, rollouts, optimizer report, and control actions.

## Scope
In scope:
- Add deterministic training status markers on `/ops/training`.
- Add deterministic rollout history and optimizer report markers.
- Add deterministic pause/reset/export action markers.
- Add UI conformance + gateway integration tests.

Out of scope:
- Live training orchestration behavior changes.
- New backend endpoints.
- Persisting training configuration mutations.

## Acceptance Criteria
### AC-1 `/ops/training` renders deterministic training status markers
Given active route `/ops/training`,
when shell HTML is rendered,
then status markers include running state, gate, store path, update interval, max rollouts, and failure streak.

### AC-2 `/ops/training` renders deterministic rollout and optimizer markers
Given active route `/ops/training`,
when shell HTML is rendered,
then rollout history and optimizer summary markers are present with deterministic values.

### AC-3 `/ops/training` renders deterministic training control action markers
Given active route `/ops/training`,
when shell HTML is rendered,
then pause/reset/export action markers and endpoint templates are present, but
the controls render as disabled endpoint-marker-only buttons until live
mutation/export wiring exists.

### AC-4 gateway `/ops/training` route includes training contract markers
Given gateway serves `/ops/training`,
when route HTML is rendered,
then status/rollout/optimizer/action markers are present.

### AC-5 non-training routes keep training panel hidden
Given active route is not `/ops/training`,
when shell HTML is rendered,
then training panel stays hidden and route regressions remain stable.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | active route `/ops/training` | render shell | status marker set present |
| C-02 | AC-2 | Functional | active route `/ops/training` | render shell | rollout and optimizer markers present |
| C-03 | AC-3 | Functional | active route `/ops/training` | render shell | training action markers/endpoints present |
| C-04 | AC-4 | Integration | gateway `/ops/training` request | render HTML | training route contract markers present |
| C-05 | AC-5 | Regression | active route `/ops` | render shell | training panel hidden marker stable |
| C-06 | AC-3 | Functional/Regression | training control lacks live action wiring | render shell and gateway route | pause/reset/export render as disabled `button type=button` controls with `data-action-mode=endpoint-marker-only` and no inert GET href |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_3148 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3148 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3144 -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3148 -- --nocapture` covers training status markers. |
| AC-2 | ✅ | `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3148 -- --nocapture` covers rollout and optimizer markers. |
| AC-3 | ✅ | `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3148_c03_training_route_renders_action_markers -- --nocapture` covers disabled endpoint-marker-only controls and absence of inert GET action hrefs. |
| AC-4 | ✅ | `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway integration_spec_3148_c04_ops_training_route_renders_training_contract_markers -- --nocapture` covers server-rendered training contract markers. |
| AC-5 | ✅ | `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3148 -- --nocapture` covers hidden non-training route markers. |
