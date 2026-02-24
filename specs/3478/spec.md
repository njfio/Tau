# Spec: Issue #3478 - M305 command-center last-action detail rows

Status: Implemented

## Problem Statement
`tau-ops-control-last-action` includes useful metadata through `data-*`
attributes, but operators do not receive explicit human-readable detail rows in
the rendered command center section. This reduces usability during incident
triage and creates a gap between machine markers and operator-facing content.

## Scope
In scope:
- Render readable Last Action detail lines in command center:
  - request id
  - action name
  - actor
  - timestamp
- Add deterministic fallback text for no-action/default cases.
- Add `tau-dashboard-ui` and gateway integration tests for rendered rows.

Out of scope:
- New control-action API fields.
- Changes to `/dashboard/actions` payload schema.
- Timestamp localization/timezone formatting.

## Acceptance Criteria
### AC-1 Last Action section renders readable detail rows
Given command-center context with last-action metadata,
when the ops shell renders,
then `tau-ops-control-last-action` contains deterministic detail elements for
request id, action, actor, and timestamp.

### AC-2 Last Action section fails closed with deterministic fallback
Given command-center context with no action metadata,
when the ops shell renders,
then Last Action detail rows render safe fallback values (`none`/`0`) rather
than empty content.

### AC-3 Gateway-rendered ops shell preserves last-action detail contracts
Given gateway-backed `/ops` rendering,
when command center HTML is fetched,
then Last Action readable row markers are present alongside existing data
attributes.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | command-center context with last action | render dashboard UI | readable detail row elements include populated values |
| C-02 | AC-2 | Regression | default/no-action context | render dashboard UI | detail rows render deterministic fallback values |
| C-03 | AC-3 | Integration | gateway `/ops` route | fetch rendered shell | Last Action row elements and values are present |

## Success Metrics / Observable Signals
- Operators can read last-action details directly in the UI section.
- Fallback behavior is deterministic and test-guarded.
- Gateway integration keeps Last Action readable markers stable.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3478 -- --nocapture` covers `functional_spec_3478_c01_last_action_section_renders_readable_detail_rows` with populated values. |
| AC-2 | ✅ | Same selector run covers `regression_spec_3478_c02_last_action_section_defaults_to_fallback_rows` asserting fallback `none`/`0` rows. |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3478 -- --nocapture` covers `integration_spec_3478_c03_ops_shell_last_action_section_renders_readable_rows` for gateway `/ops` rendering. |
