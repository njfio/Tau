# Spec: Issue #3390 - Cover B6 branch/undo/redo tool-call scenarios

Status: Reviewed

## Problem Statement
`specs/3386/conformance-matrix.md` still marks `B6-03`, `B6-04`, and `B6-05` as `N/A` because tool-call-driven branch/undo/redo flows were not exercised in gateway end-to-end conformance tests.

## Scope
In scope:
- Add deterministic gateway tests for:
  - `B6-03`: branch via tool call.
  - `B6-04`: undo navigation via tool call.
  - `B6-05`: redo navigation via tool call.
- Extend fixture pipeline tool behavior to model session branch/navigation effects through the gateway runtime loop.
- Update conformance mapping to mark these B6 rows as covered with explicit tests.

Out of scope:
- Inbound channel webhook/polling endpoint implementation.
- Provider fallback architecture changes.
- WASM/MCP extension surface additions.

## Acceptance Criteria
### AC-1 B6-03 branch via tool call is executable end-to-end
Given an active gateway session and scripted tool-call sequence,
when the model invokes a `branch` tool call,
then a branch session artifact is created and verifiable through gateway session surfaces.

### AC-2 B6-04 undo navigation via tool call is executable end-to-end
Given the branch session created by tool flow,
when the model invokes an `undo` tool call,
then navigation head transitions backward and the tool result reports applied undo semantics.

### AC-3 B6-05 redo navigation via tool call is executable end-to-end
Given an immediately preceding undo transition,
when the model invokes a `redo` tool call,
then navigation head transitions forward and the tool result reports applied redo semantics.

### AC-4 Conformance traceability updated for B6-03/04/05
Given issue conformance artifacts,
when reviewed,
then `B6-03`, `B6-04`, and `B6-05` are mapped to executable tests and no longer marked `N/A`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | seeded source session + scripted `branch` tool call | POST `/v1/responses` | branch session exists and includes expected lineage/prompt |
| C-02 | AC-2 | Functional | branch session with navigation history | scripted `undo` tool call | undo result reports changed transition to previous head |
| C-03 | AC-3 | Functional | immediately after undo | scripted `redo` tool call | redo result reports changed transition to branch head |
| C-04 | AC-4 | Conformance | B6 rows in matrix | update mappings | `B6-03/04/05` set to Covered with test reference |

## Success Metrics / Observable Signals
- New/updated nightly-tier gateway test(s) for `B6-03/04/05` are deterministic and green.
- `specs/3386/conformance-matrix.md` and `specs/3390/conformance-matrix.md` explicitly map these rows to executable coverage.
