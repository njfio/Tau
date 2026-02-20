# Spec: Issue #2826 - Command-center control confirmation SSR markers

Status: Implemented

## Problem Statement
Tau Ops command-center controls expose action and enablement markers, but do not yet publish deterministic confirmation contracts in SSR output. Live validation and operators need confirmation metadata for pause/resume/refresh actions to prevent unsafe control mutations and complete PRD command-center control requirements.

## Acceptance Criteria

### AC-1 Pause/resume/refresh controls publish deterministic confirmation markers
Given `/ops` shell render,
When control action buttons are inspected,
Then each control button exposes deterministic confirmation marker attributes.

### AC-2 Confirmation marker payload is action-specific and stable
Given control action confirmation markers,
When operators/tests inspect marker payload,
Then each action includes deterministic confirmation title/body/verb attributes tied to that action.

### AC-3 Gateway `/ops` integration exposes confirmation markers in live shell output
Given gateway `/ops` render against live fixture state,
When shell HTML is requested,
Then control action confirmation markers are present with expected deterministic payload contracts.

### AC-4 Existing command-center contracts remain stable
Given existing command-center contract suites (phase 1A..1K),
When confirmation markers are integrated,
Then prior suites remain green.

## Scope

### In Scope
- `tau-dashboard-ui` command-center control button confirmation marker contracts.
- Gateway `/ops` integration conformance tests for confirmation markers.
- Regression validation for phase 1A..1K command-center suites.

### Out of Scope
- Client-side modal runtime behavior.
- New control action endpoint semantics.
- New dashboard websocket streams.

## Conformance Cases
- C-01 (functional): UI SSR control buttons include deterministic confirmation markers.
- C-02 (functional): confirmation marker payload is action-specific for pause/resume/refresh.
- C-03 (integration): gateway `/ops` response includes confirmation markers.
- C-04 (regression): phase-1A..1K suites remain green.

## Success Metrics / Observable Signals
- `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2786 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2794 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2798 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2806 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2810 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2814 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2818 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2822 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2786 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2794 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2798 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2806 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2810 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2814 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2818 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2822 -- --test-threads=1` passes.

## Approval Gate
P1 multi-module slice proceeds with spec marked `Reviewed` per AGENTS.md self-acceptance rule. Human review required in PR.

## Implementation Notes
- Added deterministic confirmation marker attributes to command-center control actions in `tau-dashboard-ui`:
  - `data-confirm-required`
  - `data-confirm-title`
  - `data-confirm-body`
  - `data-confirm-verb`
- Confirmation payload contracts now render for:
  - `tau-ops-control-action-pause`
  - `tau-ops-control-action-resume`
  - `tau-ops-control-action-refresh`
- Added UI and gateway conformance tests for confirmation marker payloads.
