# Spec: Issue #3767 - Harness proof evidence first-screen priority

Status: Implemented

## Problem Statement

The harness preview now fits the desktop viewport better, but the proof evidence table still sits below the initial proof grid. Operators need tool execution evidence to appear as a first-screen proof artifact, near the mission DAG and before secondary acceptance, memory, gate, and artifact detail.

## Scope

In scope:

- Move `Tool Execution Evidence` into the proof window primary grid.
- Place tool evidence before acceptance criteria, verification gates, and artifacts in the proof DOM.
- Span tool evidence across the proof grid so the compact table remains readable.
- Preserve all existing tool evidence rows, data attributes, and compact table behavior.

Out of scope:

- Changing proof data, route state, or gateway behavior.
- Adding client-side expansion controls.
- Removing acceptance, memory, verification, artifact, or operator log sections.

## Acceptance Criteria

### AC-1 Tool evidence is first-screen proof content

Given `/ops/harness` renders, when the proof window is inspected, then tool evidence appears inside the primary proof grid before acceptance, verification gates, and artifacts.

### AC-2 Tool evidence keeps readable compact layout

Given tool evidence renders in the primary grid, when compact desktop styles apply, then the table keeps its no-overflow compact column behavior.

### AC-3 Proof layout remains non-invasive

Given the route renders, when harness contracts are inspected, then existing IDs, data attributes, and gateway-backed proof sections remain present.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness proof window | inspect rendered DOM order | tool evidence appears before acceptance, gates, and artifacts |
| C-02 | AC-2 | Functional | compact proof table | inspect CSS markers | evidence spans the proof grid and keeps compact no-overflow styles |
| C-03 | AC-3 | Functional | harness route | run dashboard and gateway tests | existing proof sections and route integration remain green |

## Success Metrics / Observable Signals

- Dashboard UI tests prove proof evidence placement and existing compact evidence markers.
- Browser geometry shows tool evidence starts inside the visible proof viewport at 1371px desktop preview width.
- Existing gateway-backed harness route integration remains green.
- No new dependency is introduced.
