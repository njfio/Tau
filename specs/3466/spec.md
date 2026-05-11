# Spec: Issue #3466 - M302 fail-closed ops control-action outcomes

Status: Implemented

## Problem Statement
`POST /ops/control-action` currently prioritizes happy-path action mutation but
does not provide deterministic operator-visible outcome markers for missing or
invalid form submissions. In those paths, operators can receive raw error
responses instead of consistent fail-closed redirect semantics in the ops shell
workflow.

## Scope
In scope:
- Add deterministic redirect outcome markers for ops control-action form posts:
  - `applied`
  - `missing`
  - `failed`
- Add fail-closed redirect handling for missing/invalid form submissions.
- Render normalized outcome markers in the ops shell command-center contracts.
- Add gateway and UI conformance tests for success/failure/missing paths.

Out of scope:
- Changes to `/dashboard/actions` JSON API semantics.
- New control actions beyond `pause`, `resume`, and `refresh`.
- Auth-mode or session-token workflow changes.

## Acceptance Criteria
### AC-1 Control-action form submissions return deterministic redirect markers
Given `POST /ops/control-action` form submissions,
when action is missing, invalid, or valid,
then the response is a `303` redirect to `/ops` with deterministic
`control_action_status`, `control_action`, and `control_action_reason` query
markers.

### AC-2 Missing or invalid form submissions fail closed
Given malformed or unsupported control-action form payloads,
when the form endpoint handles the request,
then it does not surface raw JSON error payloads to the operator route and
instead redirects with a failed/missing marker contract.

### AC-3 Ops shell renders normalized control-action outcome marker contracts
Given `/ops` query markers for control-action outcome,
when the shell renders,
then command-center HTML includes a deterministic marker panel containing
normalized status/action/reason data attributes.

### AC-4 Unsupported marker inputs normalize to safe defaults
Given unsupported `control_action_status`, `control_action`, or
`control_action_reason` query values,
when shell controls are resolved,
then marker values normalize to safe defaults (`idle`, `none`, `none`).

### AC-5 Control-action redirects preserve supplied shell context
Given `POST /ops/control-action` receives valid `session` and `range` form
fields,
when the endpoint redirects back to `/ops`,
then the redirect location preserves the normalized shell context before the
control-action outcome markers.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1/AC-2 | Integration | missing action form payload | POST `/ops/control-action` | redirect includes `control_action_status=missing` with deterministic markers |
| C-02 | AC-1/AC-2 | Integration/Regression | invalid action payload | POST `/ops/control-action` | redirect includes `control_action_status=failed` and reason code |
| C-03 | AC-1 | Integration | valid `resume` action | POST `/ops/control-action` | action applies and redirect includes `control_action_status=applied` |
| C-04 | AC-3 | Functional | `/ops` query with marker params | render ops shell | control-action status panel contains normalized status/action/reason attributes |
| C-05 | AC-4 | Unit/Functional | unsupported marker query values | resolve controls/render shell | marker contract defaults to `idle/none/none` |
| C-06 | AC-5 | Integration/Regression | valid action payload with `session=ops-live-session` and `range=6h` | POST `/ops/control-action` | redirect preserves `session` and `range`, and redirect body renders the same form context |

## Success Metrics / Observable Signals
- Operator-facing control-action form workflow always returns deterministic
  redirect marker contracts.
- Ops shell command-center includes explicit outcome markers for last form
  attempt status and reason.
- Existing control-action mutation behavior remains preserved for valid actions.
- Control-action form redirects do not silently reset session or timeline range.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3466 -- --nocapture` covers `integration_spec_3466_c01_*` (missing marker redirect), `integration_spec_3466_c03_*` (applied marker redirect), and `regression_spec_3466_c02_*` (failed marker redirect). |
| AC-2 | ✅ | `regression_spec_3466_c02_ops_control_action_invalid_action_fails_closed_with_redirect_marker` asserts invalid action path returns `303` redirect instead of raw error response and leaves control state unchanged. |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3466 -- --nocapture` covers `functional_spec_3466_c04_*` marker panel contract and gateway integration tests assert redirect-body marker attributes. |
| AC-4 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway requested_control_action -- --nocapture` and selector `unit_requested_control_action_reason_defaults_and_normalizes_values` validate safe normalization defaults/aliases to `idle|none|none`. |
| AC-5 | ✅ | `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway regression_spec_3466_ops_control_action_redirect_preserves_session_and_range_context -- --nocapture` covers context-preserving redirect and body render. |
