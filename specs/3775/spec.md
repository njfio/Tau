Status: Accepted

# Spec 3775: Keep Canonical Benchmark Visible In Dashboard Viewport

## Problem

The Active Missions table can consume the full left dashboard window, pushing
the M334 autonomy benchmark panel below the first viewport. That hides the
canonical proof harness from the operator even though benchmark execution is a
first-class capability of the Tau Agent Harness lane.

## Scope

In scope:
- Reserve first-viewport dashboard space for the M334 autonomy benchmark panel.
- Keep the Active Missions content available through an internal scroll region.
- Preserve the existing benchmark form, proof artifact metadata, and mission
  state chips.

Out of scope:
- Changing benchmark execution semantics.
- Removing active mission rows.
- Adding new benchmark categories.

## Acceptance Criteria

AC-1: Given the mission dashboard renders in the desktop harness layout, when
the Active Missions table contains all five rows, then the dashboard constrains
that section to an internal scroll budget instead of pushing the benchmark panel
below the first viewport.

AC-2: Given the canonical M334 benchmark panel is first-class proof, when the
dashboard renders, then the panel is marked as a first-viewport benchmark anchor
and the run benchmark form remains present.

AC-3: Given mission state chips are still required for operator scanning, when
the Active Missions section is constrained, then the compact mission status and
gate markers remain present.

## Conformance Cases

C-01 maps to AC-1. Render `/ops/harness` and assert the Active Missions section
declares a benchmark-visible first-viewport budget plus a scroll-region table
wrapper.

C-02 maps to AC-2. Render `/ops/harness` and assert the benchmark panel keeps
the M334 id, proof artifact metadata, and run benchmark form.

C-03 maps to AC-3. Render `/ops/harness` and assert compact mission state and
gate chips still render inside the constrained mission section.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3775` passes.
- Existing mission-state and benchmark contract tests continue to pass.
- Browser geometry confirms the benchmark panel top and run button are inside
  the first viewport.
