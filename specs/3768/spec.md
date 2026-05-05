# Spec: Issue #3768 - Harness proof DAG compact single row

Status: Implemented

## Problem Statement

The mission proof view now shows tool evidence early, but the plan DAG still renders as large pill controls that wrap into two rows at desktop preview width. Operators need the DAG to read as a compact single-row progression so proof evidence and verification context stay higher in the first viewport.

## Scope

In scope:

- Render the five-step proof DAG as a single compact row at desktop preview width.
- Add a durable density marker for the proof DAG.
- Keep node status, IDs, labels, and current-node metadata intact.
- Preserve the existing narrow responsive collapse behavior.

Out of scope:

- Replacing the DAG with SVG, icons, canvas, or animation.
- Changing mission plan states or gateway data.
- Changing acceptance, evidence, memory, artifact, or operator log content.

## Acceptance Criteria

### AC-1 Proof DAG stays single-row at desktop preview width

Given `/ops/harness` renders at the desktop preview width, when the proof DAG is inspected, then it uses five equal compact columns and each node stays on one line.

### AC-2 DAG semantics are preserved

Given the DAG renders compactly, when metadata is inspected, then the five node IDs, statuses, and current-node marker remain intact.

### AC-3 Compact proof layout remains testable

Given the route renders, when style contracts are inspected, then proof DAG density markers are present and existing harness tests remain green.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness proof DAG | inspect rendered CSS | DAG uses five compact columns and nowrap node labels |
| C-02 | AC-2 | Functional | harness proof DAG | inspect rendered HTML | node count, current node, IDs, and statuses remain present |
| C-03 | AC-3 | Functional | harness route | run dashboard tests | existing harness contracts remain green |

## Success Metrics / Observable Signals

- Dashboard UI tests prove compact DAG markers and preserved node semantics.
- Browser geometry shows all DAG nodes share one row at 1371px desktop preview width.
- Existing gateway-backed harness route integration remains green.
- No new dependency is introduced.
