# Spec 3781: Active Missions Whole-Row Scroll Boundary

Status: Reviewed

## Problem

The Tau Agent Harness dashboard currently shows a clipped partial Active Missions row above the benchmark panel in the first viewport. That makes the left rail look broken and reduces confidence in the harness as an operator-grade autonomy surface.

## Acceptance Criteria

AC-1: Given the `/ops/harness` dashboard is rendered, when the Active Missions panel reaches its first-viewport budget, then the scroll boundary exposes whole mission rows only.

AC-2: Given the Active Missions panel uses a compact first-viewport budget, when more missions exist than fit in the visible budget, then all missions remain in the DOM and the table wrapper remains the scroll region.

AC-3: Given the benchmark panel is the canonical autonomy proof surface, when the Active Missions table is constrained, then the benchmark panel remains directly after Active Missions and still fits into the dashboard first-viewport sequence.

## Scope

In scope:
- Tau Agent Harness dashboard HTML/CSS contract.
- Conformance coverage for the Active Missions scroll boundary.
- Browser geometry validation against the generated static harness preview.

Out of scope:
- New mission data models.
- Backend benchmark execution changes.
- Gateway/channel/dashboard adapter architecture changes.

## Conformance Cases

C-01 maps AC-1: Render `/ops/harness` and assert the Active Missions panel advertises a whole-row scroll boundary with an explicit visible-row budget.

C-02 maps AC-2: Render `/ops/harness` and assert mission rows 0 through 4 remain in the DOM while the table wrapper is the active scroll region.

C-03 maps AC-3: Render `/ops/harness` and assert the benchmark panel remains after Active Missions with its canonical benchmark metadata.

## Success Metrics

- `cargo test -p tau-dashboard-ui functional_spec_3781` passes.
- Browser geometry confirms zero partial Active Missions rows at the scroll boundary.
- Existing Tau Agent Harness dashboard regressions remain green.
