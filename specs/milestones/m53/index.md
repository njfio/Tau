# M53 — Wave-2 Verifier Alignment for Postgres Live Evidence

Milestone: [GitHub milestone #53](https://github.com/njfio/Tau/milestone/53)

## Objective

Align the wave-2 consolidated verifier with the resolved Postgres evidence path
so claim #6 is validated via deterministic live execution.

## Scope

- Update `scripts/dev/verify-gap-claims-wave2.sh` claim #6 mapping to call
  `scripts/dev/verify-session-postgres-live.sh`.
- Validate the updated wave-2 verifier end-to-end.

## Out of Scope

- New runtime/storage functionality changes.
- CI workflow changes.

## Linked Hierarchy

- Epic: #2327
- Story: #2328
- Task: #2331
- Subtask: #2332
