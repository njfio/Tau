# Spec: Issue #2810 - Command-center control-mode/action SSR markers

Status: Implemented

## Problem Statement
Tau Ops shell command-center currently exposes live health/KPI/alert/timeline markers, but control state visibility and action affordance contracts are missing. Operators cannot validate control mode, rollout gate, allowed actions, or last action metadata directly from SSR shell markers.

## Acceptance Criteria

### AC-1 Control mode and rollout gate markers reflect dashboard snapshot
Given `/ops` shell render with dashboard snapshot state,
When SSR HTML is generated,
Then command-center control markers expose snapshot control mode and rollout gate values.

### AC-2 Action affordance markers reflect allowed actions and active mode
Given `/ops` shell render,
When SSR HTML is generated,
Then pause/resume/refresh action markers reflect allowed action set and active control mode.

### AC-3 Last-action metadata markers reflect snapshot audit state
Given `/ops` shell render,
When SSR HTML is generated,
Then command-center marker attributes include last-action id/action/actor/timestamp when present and deterministic fallback markers when absent.

### AC-4 Existing phase-1A..1G contracts remain stable
Given existing auth, route, control-marker, query-state, and live-data shell contracts,
When control-mode/action markers are integrated,
Then those suites remain green.

## Scope

### In Scope
- SSR shell context extension for control-mode/action marker payload.
- Gateway mapping from `collect_gateway_dashboard_snapshot` control/health data into shell context payload.
- Conformance tests for control mode/action/last-action marker contracts in `/ops` shell output.

### Out of Scope
- Executing dashboard actions from `/ops` shell.
- Client-side polling/hydration behavior.
- New gateway action endpoints.

## Conformance Cases
- C-01 (integration): `/ops` shell includes control-mode + rollout-gate markers matching snapshot.
- C-02 (integration): `/ops` shell includes pause/resume/refresh markers reflecting allowed actions and active mode.
- C-03 (integration): `/ops` shell includes last-action metadata markers with deterministic fallback when absent.
- C-04 (regression): phase-1A..1G suites remain green.

## Success Metrics / Observable Signals
- `cargo test -p tau-gateway functional_spec_2810 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2786 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2794 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2798 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2806 -- --test-threads=1` passes.

## Approval Gate
P1 multi-module slice proceeds with spec marked `Reviewed` per AGENTS.md self-acceptance rule.
