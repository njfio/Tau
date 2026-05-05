# Spec 3786: Proof Header Metadata No-Wrap

Status: Implemented

## Problem

The center Mission Detail proof header splits the Run ID value into two visual lines. That makes the durable run identifier harder to scan and weakens the proof-view header as an operator reference surface.

## Acceptance Criteria

AC-1: Given `/ops/harness` is rendered in the desktop harness preview, when the proof header metadata is visible, then the Run ID value remains on one visual line.

AC-2: Given the proof header metadata is compacted, when all five metadata pairs are rendered, then Run ID, Elapsed, Tool Budget, Cost, and Retry Count remain visible without changing their values.

AC-3: Given metadata no-wrap rules are applied, when the center proof window is rendered, then the proof window remains inside the first viewport with no horizontal document overflow.

## Scope

In scope:
- Tau Agent Harness center proof header metadata HTML/CSS contract.
- Run ID and metadata value no-wrap behavior.
- Conformance test plus browser geometry validation.

Out of scope:
- Changing mission data, tool evidence tables, proposal review UI, or gateway behavior.
- TUI behavior.

## Conformance Cases

C-01 maps AC-1: Render `/ops/harness` and assert the proof header declares single-line Run ID behavior.

C-02 maps AC-2: Render `/ops/harness` and assert all five metadata values remain present under the proof header.

C-03 maps AC-3: Browser geometry checks the Run ID value height against one-line metadata height and verifies the proof window remains in viewport.

## Success Metrics

- `cargo test -p tau-dashboard-ui functional_spec_3786` passes.
- Browser geometry confirms Run ID height is one line, no document horizontal overflow, and the proof window stays within the viewport.
- Existing harness regression tests remain green.
