# Issue #3798: Provider Verifier Repair Loop

Status: Implemented
GitHub Issue: #3798
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture

## Problem

Tau's provider-backed coding loop can request a provider edit, apply it, run
verifiers, and produce PR-ready output when the first edit succeeds. When the
post-edit verifier fails, the loop currently stops in an executing state without
feeding the verifier failure back into the provider.

That makes Tau a one-shot code generator rather than a functional autonomous
coding harness. The product loop needs a bounded repair cycle: inspect failed
verifier evidence and the current diff, ask the provider for a targeted patch,
rerun verifiers, and only commit/prepare PR-ready output after verification is
green.

## Scope

In scope:
- Add a bounded provider repair loop to `tau_live_coding_loop_harness`.
- Include failed verifier stdout/stderr, current changed files, and current diff
  context in repair prompts.
- Persist per-attempt provider evidence in the harness report.
- Add deterministic coverage where the first provider edit fails a verifier and
  a second provider repair reaches PR-ready.
- Preserve fail-closed behavior for malformed provider output, invalid edits,
  missing provider client/auth, and exhausted repair attempts.

Out of scope:
- Changing provider transport semantics or adding new model providers.
- Launching a live browser/game inspection loop.
- Automatically opening or merging GitHub PRs from this harness.
- Adding unbounded retries.

## Acceptance Criteria

### AC-1: Failed Verifier Triggers Provider Repair

Given a provider edit is parsed and applied but a verifier fails, when repair
attempts remain, then Tau builds a repair prompt containing failed verifier
evidence and current diff context and asks the provider for a targeted patch.

### AC-2: Repair Can Reach PR-Ready

Given the second provider response fixes the verifier failure, when Tau applies
the repair and reruns verifiers, then the mission commits the verified changes
and prepares PR-ready output.

### AC-3: Attempts Are Durable And Observable

Given a provider-backed run uses multiple provider attempts, when the report is
written, then it includes the final provider report plus an ordered attempt list
with parse status, reason code, edit paths, response hash, and repair context
flags.

### AC-4: Exhausted Repairs Fail Closed

Given verifier failures persist until the configured repair attempt limit is
exhausted, when the loop exits, then Tau does not commit, does not prepare
PR-ready output, records the exhausted repair reason, and keeps verifier
evidence available.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | first provider edit fails verifier | provider loop continues | second prompt includes verifier stderr and git diff context |
| C-02 | AC-2 | Integration | second provider edit fixes verifier | provider loop reruns | report passes, commit exists, PR-ready bundle exists |
| C-03 | AC-3 | Functional | two provider attempts run | report is written | `provider_attempts` contains two sanitized attempts |
| C-04 | AC-4 | Regression | provider attempts do not satisfy verifier | attempt limit reached | report blocks without commit or PR-ready bundle |

## Success Signals

- Focused harness tests cover C-01, C-03, and C-04.
- A shell integration script covers C-02 against a disposable repo.
- Existing provider-backed proof scripts continue to pass.
