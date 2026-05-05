# Issue 3788: Harness KPI Labels Stay Word-Boundary Readable

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The mission harness dashboard KPI cards inherit global `overflow-wrap: anywhere`
behavior. On the desktop harness viewport this causes the "Pending
Verifications" KPI heading to split mid-word, leaving a dangling fragment and
making the top dashboard summary look broken.

## Scope

In scope:
- Keep the four KPI cards visible in the first dashboard row.
- Preserve full KPI label wording.
- Prevent KPI headings from splitting inside words.
- Preserve zero document-level horizontal overflow in the rendered harness.

Out of scope:
- Renaming KPI metrics.
- Reworking the dashboard grid or mission/benchmark tables.
- Changing live mission data semantics.

## Acceptance Criteria

AC-1: Given the `/ops/harness` dashboard renders at the desktop harness
viewport, when the KPI row is displayed, then KPI headings wrap only at word
boundaries and do not use global `anywhere` wrapping.

AC-2: Given the "Pending Verifications" KPI heading renders, when it wraps,
then it remains a two-word label without splitting `Verifications` into a
dangling third line fragment and keeps an accessible full-label name.

AC-3: Given the KPI card typography is compacted, when the harness is measured
at the desktop preview viewport, then KPI headings fit their cards without
creating document-level horizontal overflow.

## Conformance Cases

C-01 maps to AC-1: HTML exposes a KPI label fit contract and CSS scopes KPI
heading wrapping to word-boundary behavior.

C-02 maps to AC-2: The pending-verifications card keeps the full label text
and receives the same KPI heading fit treatment as the rest of the KPI row.

C-03 maps to AC-3: Browser geometry confirms KPI heading scroll widths stay
within client widths and the document has no horizontal overflow.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3788`
- Browser geometry check against `/tmp/tau-harness-after.html` at 1512x1038.
