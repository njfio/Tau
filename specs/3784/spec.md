# Spec 3784: Center Proof Evidence Containment

Status: Implemented

## Problem

The center proof window still contains clipped proof evidence. The Tool Execution Evidence table extends past its wrapper at the artifact column, and the Acceptance Criteria card hides the later proof chips below its visible boundary.

## Acceptance Criteria

AC-1: Given `/ops/harness` is rendered in the desktop harness preview, when Tool Execution Evidence is visible, then the evidence table fits inside its wrapper without horizontal clipping.

AC-2: Given `/ops/harness` is rendered in the desktop harness preview, when Acceptance Criteria is visible, then all five acceptance criteria chips fit inside the card.

AC-3: Given compact proof evidence is applied, when columns are compressed or hidden to fit the proof pane, then core execution state remains visible: tool name, plan node, runtime, status, and artifact path.

## Scope

In scope:
- Tau Agent Harness center proof pane HTML/CSS contract.
- Tool evidence table fit and acceptance criteria chip layout.
- Conformance test plus browser geometry validation.

Out of scope:
- Changing mission state or tool execution data.
- Gateway behavior.
- TUI behavior.

## Conformance Cases

C-01 maps AC-1: Render `/ops/harness` and assert Tool Execution Evidence declares a compact no-overflow fit contract with scoped table layout.

C-02 maps AC-2: Render `/ops/harness` and assert Acceptance Criteria declares all-criteria-visible density while retaining all five criteria.

C-03 maps AC-3: Render `/ops/harness` and assert tool, plan node, runtime, status, and artifact cells remain in the DOM while call ID is marked as the hidden compact column.

## Success Metrics

- `cargo test -p tau-dashboard-ui functional_spec_3784` passes.
- Browser geometry confirms zero Tool Execution Evidence horizontal overflow and zero Acceptance Criteria child overflow.
- Existing harness regression tests remain green.
