# Plan: Issue #3687 - Group `tau-tui` command palette commands into operator sections

Status: Implemented

## Approach
Extend the command catalog with operator-oriented section metadata, then update
the palette renderer to emit section labels alongside the existing paged result
window. Keep selection and preview tied to the current command entry, while
showing only the section labels that are actually relevant to the visible,
filtered results.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - extend command catalog metadata with section/category labels
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - render grouped palette results and relevant section labels
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN tests for grouped rendering, filtered section visibility, and
    preview stability under paging

## Contracts
- Palette render shows section labels before relevant visible commands
- Filtered results only show section labels for sections with visible matches
- Selection, paging, visible-range feedback, and preview continue to operate on
  the selected command entry rather than on section labels

## Risks
- Section labels can crowd the palette if they are repeated too often
- Group rendering must not break the current page-size assumptions
- Preview text and section labels need to fit together in the existing overlay

## Verification Strategy
- Add failing tests first for grouped rendering and filtered section behavior
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
