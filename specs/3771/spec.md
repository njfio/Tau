# Spec: Issue #3771 - Harness verification gates first-screen priority

Status: Implemented

## Problem Statement

The proof pane now shows tool evidence and the operator log in the first viewport, but verification gates still sit below lower-signal memory summary content. Operators need failed and running gate state immediately after evidence/log context because gates decide whether an autonomous run is actually acceptable.

## Scope

In scope:

- Promote verification gates directly after acceptance criteria in the proof grid.
- Keep tool evidence and operator log ahead of secondary proof detail.
- Add durable first-screen priority markers for verification gates.
- Compact acceptance/gate chips enough to improve first-screen fit.

Out of scope:

- Changing gate IDs, statuses, labels, or counts.
- Changing memory summary content.
- Changing right-side self-improvement review ordering.

## Acceptance Criteria

### AC-1 Verification gates are prioritized before memory/artifacts

Given `/ops/harness` renders, when proof DOM order is inspected, then verification gates appear after acceptance and before memory summary and artifacts.

### AC-2 Evidence and log remain first

Given `/ops/harness` renders, when proof DOM order is inspected, then tool evidence and operator log remain ahead of acceptance, gates, memory, and artifacts.

### AC-3 Secondary proof chips are compact

Given `/ops/harness` renders, when style markers are inspected, then acceptance and verification-gate chips use compact padding/gap markers.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | proof grid | inspect DOM order | gates appear before memory and artifacts |
| C-02 | AC-2 | Functional | proof grid | inspect DOM order | evidence and log remain first proof context |
| C-03 | AC-3 | Functional | proof secondary chips | inspect style markers | acceptance/gate chip compact markers are present |

## Success Metrics / Observable Signals

- Dashboard UI tests prove verification gates are promoted and compact markers exist.
- Browser geometry shows verification gates begin in the first 967px desktop viewport.
- Existing evidence/log priority tests remain green.
- No new dependency is introduced.
