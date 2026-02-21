# Spec: Issue #3200 - reconcile stale dashboard gap claim in whats-missing report

Status: Implemented

## Problem Statement
The current `tasks/whats-missing.md` report says dashboard consolidation remains incomplete, but repository conformance verification indicates the dashboard runtime consolidation is complete and governed by ADR-backed checks.

## Scope
In scope:
- Update report language to reflect verified dashboard consolidation status.
- Update conformance script markers to reject stale dashboard-gap wording.
- Keep the report's verification contract executable and deterministic.

Out of scope:
- New dashboard runtime feature implementation.
- Architectural redesign.

## Acceptance Criteria
### AC-1 stale dashboard remaining-gap wording is removed
Given `tasks/whats-missing.md`,
when reviewing remaining high-impact gaps,
then stale dashboard consolidation gap wording is absent.

### AC-2 resolved dashboard status is explicitly marked with evidence
Given the refreshed report,
when reviewing resolved capability entries,
then dashboard consolidation appears in resolved status with evidence references.

### AC-3 conformance script enforces the updated dashboard markers
Given `scripts/dev/test-whats-missing.sh`,
when it runs,
then it requires the new dashboard-resolved marker and rejects the stale dashboard-gap marker.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | refreshed report | run whats-missing conformance script | stale dashboard gap marker is rejected |
| C-02 | AC-2 | Functional/Conformance | refreshed report | run whats-missing conformance script | dashboard resolved marker is required |
| C-03 | AC-3 | Functional/Conformance | refreshed report + script | execute script | exits 0 only with updated marker contract |

## Success Metrics / Observable Signals
- `scripts/dev/verify-dashboard-consolidation.sh`
- `scripts/dev/test-whats-missing.sh`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
