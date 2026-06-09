# Spec 3807: Intake Clarifying Contract

Status: Reviewed

## Problem Statement

Tau can classify autonomous coding issue intake as unsafe, broad, vague, missing
authority, missing credentials, or solvable. The output is still too thin for a
hands-off queue because an operator or scheduler must infer what exact question
to ask next from prose fields. Intake should persist a machine-readable decision
and exact clarifying questions so Tau can either proceed safely or block with the
minimum missing information.

## Scope

In:

- Extend autonomous coding intake outcomes with machine-readable decision fields.
- Persist structured clarifying questions for vague, broad, unsafe, or
  authority-blocked issues.
- Keep verifier plans and existing classification names backward-compatible.
- Document the contract for operator and queue consumers.

Out:

- Live provider calls during intake.
- Automatic mutation without verifier and edit/provider authority.
- GitHub issue creation, PR merge, or protected-branch bypass.

## Acceptance Criteria

AC-1: Given an underspecified issue, when Tau records intake, then the persisted
outcome includes `decision=needs_clarification` and clarifying questions for
expected behavior, current behavior, affected surface, and verifier command.

AC-2: Given a broad issue, when Tau records intake, then the persisted outcome
includes `decision=split_required` and clarifying questions that ask for a single
module/surface and one acceptance criterion.

AC-3: Given a bounded issue with verifier, edit/provider authority, and required
credentials in the internal authority context, when Tau records intake, then the
outcome includes `decision=ready_to_run`, no clarifying questions, and preserves
the existing `solvable` classification.

AC-4: Given older consumers reading intake JSON, when new fields are absent,
then deserialization remains backward-compatible through defaults.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | title `Fix it`, body `Broken` | `needs_clarification`, four required question reason codes |
| C-02 | AC-2 | Conformance | broad autonomous-everything issue | `split_required`, scoped-surface and acceptance-criterion questions |
| C-03 | AC-3 | Conformance | bounded issue with full authority context | `ready_to_run`, empty questions, `solvable` |
| C-04 | AC-4 | Regression | legacy JSON without new fields | loads with default decision and empty questions |

## Success Signals

- Focused `tau-runtime` tests cover C-01..C-04.
- `docs/guides/autonomous-coding-jobs.md` documents the new intake fields.
- `cargo fmt --check` and `git diff --check` pass.
