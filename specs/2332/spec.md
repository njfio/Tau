# Spec #2332

Status: Implemented
Milestone: specs/milestones/m53/index.md
Issue: https://github.com/njfio/Tau/issues/2332

## Problem Statement

Claim #6 was moved to resolved evidence via
`scripts/dev/verify-session-postgres-live.sh`, but the consolidated wave-2
verification script still uses an environment-optional direct test call. The
verification runner should align with the resolved evidence path.

## Scope

In scope:

- Update `scripts/dev/verify-gap-claims-wave2.sh` claim #6 section to invoke
  `scripts/dev/verify-session-postgres-live.sh`.
- Execute wave-2 verifier and confirm successful completion.

Out of scope:

- Changes to `tau-session` storage behavior.
- Additional roadmap restructuring.

## Acceptance Criteria

- AC-1: Given the updated wave-2 verifier, when reviewing claim #6 mapping, then
  it invokes `scripts/dev/verify-session-postgres-live.sh`.
- AC-2: Given current branch state, when running
  `scripts/dev/verify-gap-claims-wave2.sh`, then the command exits `0`.
- AC-3: Given fail-closed script behavior, when the delegated Postgres verifier
  fails, then wave-2 verifier exits non-zero.

## Conformance Cases

- C-01 (AC-1, conformance): `scripts/dev/verify-gap-claims-wave2.sh` contains
  claim #6 invocation of `scripts/dev/verify-session-postgres-live.sh`.
- C-02 (AC-2, integration): `scripts/dev/verify-gap-claims-wave2.sh` completes
  successfully on this branch.
- C-03 (AC-3, functional): wave-2 verifier uses strict shell flags and direct
  command execution for delegated verifier command.

## Success Metrics / Observable Signals

- Wave-2 claim #6 evidence is executable through one consolidated verifier.
- No mismatch remains between roadmap evidence and verification runner behavior.
