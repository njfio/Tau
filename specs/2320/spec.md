# Spec #2320

Status: Implemented
Milestone: specs/milestones/m51/index.md
Issue: https://github.com/njfio/Tau/issues/2320

## Problem Statement

Claims 5-15 from the gap list continue to be referenced as if unresolved, but many are already implemented or partially covered. We need a deterministic, executable verification sweep for this claim set to prevent stale planning inputs.

## Scope

In scope:

- Add a wave-2 verification runner that covers claims 5-15 using mapped tests/scripts.
- Update `tasks/resolution-roadmap.md` with a dated wave-2 revalidation table.
- Distinguish statuses as `Resolved`, `Partial`, or `Open` with evidence.

Out of scope:

- Implementing large open items discovered by validation (tracked separately).
- Rewriting the full roadmap document.

## Acceptance Criteria

- AC-1: Given a local checkout, when running the wave-2 verification runner, then mapped checks for claims 5-15 execute in a deterministic order and fail closed on first error.
- AC-2: Given the updated roadmap file, when reviewing the wave-2 section, then each claim (5-15) has status plus evidence command references.
- AC-3: Given current branch state, when running the wave-2 verification runner, then all mapped checks pass and the runner exits `0`.

## Conformance Cases

- C-01 (AC-1, functional): `scripts/dev/verify-gap-claims-wave2.sh` exists, is executable, and uses strict shell execution flags.
- C-02 (AC-2, conformance): `tasks/resolution-roadmap.md` contains `Gap Revalidation Wave 2 (2026-02-17)` with claims 5-15 status/evidence rows.
- C-03 (AC-3, integration): Running `scripts/dev/verify-gap-claims-wave2.sh` exits successfully on this branch.

## Success Metrics / Observable Signals

- Maintainers can reproduce claim status with one command.
- Roadmap statuses for claims 5-15 become evidence-backed.
- Remaining open items are isolated instead of mixed with already delivered features.
