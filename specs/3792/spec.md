# Issue 3792: Harness Proof Pane Fits The In-App Browser Width

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The in-app browser preview can attach at a narrower desktop viewport than the
wide static preview. At that width, the proof pane keeps the three-column
harness layout but still clips some compact proof labels and slightly clips the
verification gate section vertically.

## Scope

In scope:
- Preserve the three-window desktop harness layout above the existing stacked
  breakpoint.
- Keep all acceptance criteria and verification gates visible at the in-app
  browser viewport width.
- Avoid document-level horizontal overflow.

Out of scope:
- Changing mission, proposal, or benchmark proof data.
- Reordering proof sections.
- Changing gateway/channel/dashboard adapter semantics.

## Acceptance Criteria

AC-1: Given `/ops/harness` renders at the in-app browser desktop width, when
the proof pane displays acceptance criteria, then the labels fit without hidden
horizontal clipping.

AC-2: Given `/ops/harness` renders at the in-app browser desktop width, when
the proof pane displays verification gates, then the gate section fits without
hidden vertical clipping.

AC-3: Given the narrower desktop preview keeps the three-window layout, when
the page is measured, then document-level horizontal overflow remains zero.

## Conformance Cases

C-01 maps to AC-1: Acceptance criteria expose a narrow-width full-label fit
contract and the compact label CSS keeps the longest labels inside their cards.

C-02 maps to AC-2: Verification gates expose a narrow-width height budget and
the section height accommodates all five gate chips.

C-03 maps to AC-3: Browser geometry at 1370x968 reports zero document-level
horizontal overflow and no hidden overflow for the acceptance/gate proof
surfaces.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3792`
- Browser Use screenshot of `/tmp/tau-harness-after.html`
- Playwright geometry check against `/tmp/tau-harness-after.html` at 1370x968.
