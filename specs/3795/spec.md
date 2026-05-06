# Issue 3795: Harness Fits In App Browser Without Review Rail Clipping

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The live `/ops/harness` route loaded in the Codex in-app browser, but the
three-window desktop grid kept the self-improvement review rail partially
offscreen at the available viewport width. That made the page technically
rendered but not operator-usable.

## Scope

In scope:
- Add a responsive harness layout contract for in-app browser widths.
- Keep desktop three-window layout for wider screens.
- Verify the live browser route exposes the no-clipping contract.

Out of scope:
- Redesigning the entire harness visual system.
- Changing benchmark, proposal, approval, or apply behavior.

## Acceptance Criteria

AC-1: Given the harness renders below the wide-desktop threshold, when the
viewport cannot fit three columns, then the review rail is not clipped offscreen.

AC-2: Given the live harness route is loaded in Browser Use, when the DOM is
inspected, then the harness exposes the no-clipping in-app browser fit marker
and console errors are zero.

## Conformance Cases

C-01 maps to AC-1: `tau-dashboard-ui` render tests assert the `1450px`
responsive contract and two-column grid areas.

C-02 maps to AC-2: Browser Use inspects the live localhost route and verifies
`data-in-app-browser-fit="no-right-rail-clipping"`.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3795_c01_c02_harness_fits_in_app_browser_without_review_rail_clipping`
- Browser Use DOM inspection of the live `/ops/harness` route.
