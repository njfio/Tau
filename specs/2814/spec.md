# Spec: Issue #2814 - Command-center timeline chart and range SSR markers

Status: Implemented

## Problem Statement
Tau Ops shell currently exposes timeline aggregate counts but does not provide a timeline chart marker contract or deterministic 1h/6h/24h selector state markers. Operators cannot validate chart-data freshness or selected range state directly in SSR output.

## Acceptance Criteria

### AC-1 Timeline chart markers reflect live queue timeline snapshot metadata
Given `/ops` shell render with dashboard queue timeline snapshot,
When SSR HTML is generated,
Then timeline chart markers expose live point count and last timestamp metadata.

### AC-2 Range selector markers reflect query-selected timeline range
Given `/ops` shell render with a `range` query parameter,
When SSR HTML is generated,
Then 1h/6h/24h selector markers reflect selected range state.

### AC-3 Invalid or missing range query values safely default to 1h markers
Given `/ops` shell render with no or invalid `range` query,
When SSR HTML is generated,
Then selected range markers default to `1h` without breaking existing control-state markers.

### AC-4 Existing phase-1A..1H shell contracts remain stable
Given existing auth, route, shell-control, command-center live-data, and control-state contracts,
When timeline range markers are integrated,
Then those suites remain green.

## Scope

### In Scope
- Query parsing extension for timeline range controls on `/ops*` shell routes.
- Timeline chart + range selector marker rendering in `tau-dashboard-ui` shell.
- Gateway mapping of live queue timeline point metadata into shell context.
- Conformance and regression tests for timeline marker behavior.

### Out of Scope
- Client-side chart rendering implementation.
- Real-time polling stream hydration for timeline charts.
- New API endpoints.

## Conformance Cases
- C-01 (integration): `/ops` shell includes timeline chart markers with point count and last timestamp from snapshot.
- C-02 (integration): `/ops?range=6h` shell includes selected-range markers for 6h and non-selected markers for 1h/24h.
- C-03 (integration): `/ops?range=<invalid>` shell defaults selected-range markers to 1h.
- C-04 (regression): phase-1A..1H suites remain green.

## Success Metrics / Observable Signals
- `cargo test -p tau-gateway functional_spec_2814 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2814 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2786 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2794 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2798 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2806 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2810 -- --test-threads=1` passes.

## Approval Gate
P1 multi-module slice proceeds with spec marked `Reviewed` per AGENTS.md self-acceptance rule.
