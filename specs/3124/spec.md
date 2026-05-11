# Spec: Issue #3124 - ops job cancel action contracts

Status: Implemented

## Problem Statement
`/ops/tools-jobs` now exposes deterministic jobs list and job detail output contracts (PRD `2100` and `2101`) but does not expose deterministic cancel action contracts required by PRD item `2102`.

## Scope
In scope:
- Add deterministic cancel action markers for jobs rows.
- Add deterministic cancel contract panel for requested cancel actions.
- Add gateway query contract for requested cancel job id.
- Apply deterministic cancel outcomes to fixture jobs rows.
- Add conformance and regression tests.

Out of scope:
- Real runtime cancellation wiring to `/gateway/jobs/{job_id}/cancel`.
- Job queue persistence or orchestration changes.
- Dependency changes.

## Acceptance Criteria
### AC-1 `/ops/tools-jobs` renders deterministic cancel action markers
Given `/ops/tools-jobs` renders with jobs rows,
when shell HTML is produced,
then each jobs row renders deterministic cancel action markers including enabled/disabled contract state.

### AC-1a Disabled cancel actions are not clickable operator controls
Given a job row is terminal (`completed`, `failed`, or `cancelled`),
when `/ops/tools-jobs` renders cancel affordances,
then the row and selected-job cancel controls preserve disabled marker data but
render non-link disabled text instead of a clickable cancel link.

### AC-2 selected jobs expose deterministic cancel request contract panel
Given `/ops/tools-jobs` receives a cancel request contract for a job id,
when shell HTML is produced,
then cancel contract panel markers render requested job id and deterministic cancel status outcome.

### AC-3 gateway route resolves deterministic cancel outcomes for requested jobs
Given gateway renders `/ops/tools-jobs?cancel_job=<id>`,
when requested job id is valid and cancellable or not,
then deterministic cancel outcome contracts are rendered and jobs rows reflect deterministic status transitions.

### AC-4 non-tools routes keep hidden cancel markers and regressions remain green
Given any non-`/ops/tools-jobs` route renders,
when shell HTML is produced,
then cancel markers remain present and hidden, and nearby regressions remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | active route `/ops/tools-jobs` with running/completed jobs | render shell | jobs rows include deterministic cancel action markers with per-row enabled/disabled contracts |
| C-02 | AC-2 | Functional | cancel request state provided in snapshot | render shell | cancel panel markers show requested job id and deterministic cancel status |
| C-03 | AC-3 | Integration | gateway route `/ops/tools-jobs?cancel_job=job-001` | HTTP render | requested running job transitions deterministically to `cancelled`; cancel panel contracts show `cancelled` |
| C-04 | AC-4 | Regression | active route is not `/ops/tools-jobs` | render shell | cancel contract markers remain present/hidden; nearby regression suites pass |
| C-05 | AC-1a | Functional/Regression | terminal jobs render in `/ops/tools-jobs` | render shell and gateway route | disabled cancel markers render as `Cancel unavailable` text with `aria-disabled=true` and no clickable href |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_3124 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3124 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3120 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3120 -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3124_c01_c02_tools_route_renders_job_cancel_action_markers -- --nocapture` covers enabled/disabled markers and terminal non-link controls. |
| AC-1a | ✅ | Same dashboard regression plus `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway 3124 -- --nocapture` assert terminal job cancel controls render as `Cancel unavailable` spans with `aria-disabled=true`. |
| AC-2 | ✅ | Dashboard `functional_spec_3124_c01_c02_*` covers selected cancel panel markers. |
| AC-3 | ✅ | Gateway `integration_spec_3124_c03_*` covers deterministic cancellation and terminal disabled cancel rendering. |
| AC-4 | ✅ | Dashboard `regression_spec_3124_c04_*` keeps non-tools route hidden marker contracts. |
