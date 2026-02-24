# Spec: Issue #3482 - M306 command-center last-action reason visibility

Status: Implemented

## Problem Statement
The command-center Last Action section exposes request id/action/actor/timestamp
but omits action reason. Operators lose contextual intent for control actions,
despite reason being available in dashboard action audit records.

## Scope
In scope:
- Extend command-center snapshot with `last_action_reason`.
- Render Last Action reason row in ops shell (`tau-ops-last-action-reason`).
- Preserve existing marker contracts while adding reason marker contracts.
- Add dashboard-ui + gateway tests for populated/fallback reason values.

Out of scope:
- New action endpoint fields.
- Free-form reason formatting transformations.
- Localization or time formatting changes.

## Acceptance Criteria
### AC-1 Snapshot mapping includes last-action reason
Given gateway dashboard control snapshots with last action records,
when command-center context is built,
then `last_action_reason` is populated from the latest action reason and
defaults to `none` when unavailable.

### AC-2 Ops shell renders Last Action reason row
Given command-center render contexts,
when ops shell HTML is rendered,
then Last Action section contains deterministic reason row and marker:
- `id="tau-ops-last-action-reason"`
- `data-last-action-reason="<value>"`.

### AC-3 Gateway `/ops` render preserves reason row contracts
Given gateway `/ops` route rendering,
when fetched in integration tests,
then rendered HTML includes populated reason row content and marker values.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | action snapshot with reason | map command-center snapshot | `last_action_reason` populated |
| C-02 | AC-2 | Regression | default/no-action snapshot | render ops shell | reason row uses fallback `none` |
| C-03 | AC-3 | Integration | gateway `/ops` render | fetch HTML | reason row + data marker contracts present |

## Success Metrics / Observable Signals
- Last Action reason is visible to operators without DOM inspection.
- Fallback behavior remains deterministic (`none`) for no-action paths.
- Gateway and UI selector contracts guard reason-row stability.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3482 -- --nocapture` includes `functional_spec_3482_c01_collect_command_center_snapshot_maps_last_action_reason`. |
| AC-2 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3482 -- --nocapture` covers populated/fallback reason marker rows (`functional_spec_3482_c01_*`, `regression_spec_3482_c02_*`). |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3482 -- --nocapture` covers `integration_spec_3482_c03_ops_shell_last_action_reason_row_renders_fixture_reason`. |
