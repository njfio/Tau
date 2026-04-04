# Spec: Issue #3710 - Match tau-tui command palette queries against section labels

Status: Implemented

## Problem Statement
The `tau-tui` command palette groups commands into operator-facing sections like
`Clipboard`, `Runtime`, and `Missions`, but the matcher still ignores those
section labels. Queries such as `clipboard save` or `runtime detach` should
surface the obvious commands because the palette itself teaches operators to
think in those grouped categories. Without section-aware filtering, discoverable
group labels remain visually useful but behaviorally inert.

## Scope
In scope:
- section-aware command palette filtering
- combined matching over section labels with existing command metadata
- preserving existing exact/prefix/fuzzy and metadata-aware behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3710/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- ranking redesign beyond section-aware matching
- new command-palette commands
- runtime/gateway changes
- transcript search behavior

## Acceptance Criteria
### AC-1 Section labels contribute to multi-token command discovery
Given a command section contributes one part of the query and command metadata
contributes another,
when the operator searches with those combined terms,
then Tau surfaces the command in command-palette matches.

### AC-2 Section labels work for runtime-oriented command discovery
Given a command lives under the `Runtime` group and its command text supplies
the remaining query term,
when the operator searches using the section label plus command intent,
then Tau surfaces the runtime command even if the summary does not repeat the
section label.

### AC-3 Existing command-palette filtering behavior does not regress
Given the earlier exact/prefix/fuzzy, separator-normalized, metadata-aware, and
multi-token matching behavior,
when section-aware matching lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Querying `clipboard save` surfaces `save-transcript` because
  `clipboard` comes from the section label and `save` comes from the command
  metadata. Maps to AC-1. Tier: Functional.
- C-02 Querying `runtime detach` surfaces `detach` because `runtime` comes from
  the section label and `detach` comes from the command text. Maps to AC-2.
  Tier: Functional.
- C-03 Existing command-palette filter and submission coverage still passes
  after section-aware matching lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can search by the same section concepts the palette presents in the
  UI
- Section labels improve discovery instead of only acting as visual grouping
- Existing palette ranking remains stable

## Key Decisions
- Treat section labels as another metadata source rather than a separate search
  mode
- Preserve existing ranking tiers and broaden only what counts as a match
- Limit the slice to filter behavior instead of UI rendering changes
