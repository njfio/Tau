# Issue 3789: Harness Benchmark Categories Render As Operator Labels

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The M334 autonomy benchmark table renders raw category identifiers such as
`greenfield_build`, `research_design`, and `data_to_deliverable`. These machine
IDs overflow the left benchmark column and make the panel look unfinished in
the first viewport.

## Scope

In scope:
- Preserve the machine category identifier in `data-category`.
- Render a compact operator-readable label in the benchmark table.
- Prevent benchmark category cells from horizontal overflow in the desktop
  preview viewport.

Out of scope:
- Changing benchmark result semantics.
- Reworking benchmark row ordering or pass-rate calculations.
- Changing canonical benchmark proof artifacts.

## Acceptance Criteria

AC-1: Given the harness benchmark rows render, when a row has a machine
category identifier, then the row preserves that identifier in `data-category`.

AC-2: Given a category identifier contains separators, when the first table
cell is displayed, then it renders a compact human label instead of the raw
machine ID.

AC-3: Given the benchmark table is measured in the desktop preview viewport,
then category cells do not report scroll-width overflow.

## Conformance Cases

C-01 maps to AC-1: `data-category` remains unchanged for fallback and
state-backed benchmark rows.

C-02 maps to AC-2: Known fallback rows render labels such as `Greenfield build`,
`Research design`, and `Data to deliverable`.

C-03 maps to AC-3: Browser geometry confirms benchmark table cells fit their
client widths with zero document-level horizontal overflow.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3789`
- Browser geometry check against `/tmp/tau-harness-after.html` at 1512x1038.
