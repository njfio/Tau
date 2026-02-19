# M103 - Spacebot G4 Branch-as-Tool (Phase 2)

Status: In Progress
Related roadmap items: `tasks/spacebot-comparison.md` -> G4 (branch runtime execution and parent conclusion wiring)

## Objective
Close the remaining G4 parity gaps after phase-1 by executing an isolated branch reasoning turn from `branch` tool calls and returning structured branch conclusions to the parent session with concurrency guardrails.

## Issue Map
- Epic: #2600
- Story: #2601
- Task: #2602
- Subtask: #2603

## Deliverables
- Branch tool follow-up execution path in `tau-agent-core` that runs an isolated branch turn.
- Memory-only branch tool surface (no user-facing reply tools during branch execution).
- Structured branch conclusion payload recorded as the parent tool result.
- Configurable per-session maximum concurrent branch runs.
- Conformance/regression validation evidence.

## Exit Criteria
- #2600, #2601, #2602, and #2603 closed.
- `specs/2602/spec.md` status set to `Implemented`.
- G4 checklist bullets in `tasks/spacebot-comparison.md` marked complete with validation evidence.
