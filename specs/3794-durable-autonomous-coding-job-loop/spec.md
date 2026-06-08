# Issue #3794: Durable Autonomous Coding Job Loop

Status: Implemented
GitHub Issue: #3794
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture

## Problem

Tau can run coding missions and durable background jobs, but the product path
does not yet combine them into one recoverable issue-to-PR loop. A coding task
should be submitted as a durable job, preserve mission checkpoints, recover
from stale `running` background-job manifests, replay the mission from the last
checkpoint, and expose operator-readable status through stable files/JSON.

## Scope

In scope:
- Add a reusable autonomous coding job runtime that owns the durable job record
  and delegates execution to `CodingMissionRunner`.
- Submit coding jobs with repo, issue, verifier commands, controlled edits, and
  PR mode metadata.
- Queue the coding job through the existing background-job runtime.
- Replay/resume mission checkpoints after recovery.
- Persist status snapshots with mission phase, verifier state, changed files,
  resume command, and PR-ready handoff evidence.
- Add a dedicated CLI binary for submit/run/status/recover/replay validation.

Out of scope:
- Full arbitrary model planning over any issue without bounded inputs.
- Automatic merge to protected branches.
- Replacing `tau-unified` with a new command center in this slice.
- New scheduler crates or new third-party dependencies.

## Acceptance Criteria

### AC-1: Durable Submit

Given a repo path, issue URL, verifier commands, and controlled edits, when an
autonomous coding job is submitted, then Tau persists a coding mission state, an
autonomous job record, and a background-job record that points back to the
mission.

### AC-2: Multi-File Coding Execution

Given a submitted job whose verifier fails until two files are changed, when
the autonomous job runner executes, then it applies the controlled multi-file
edit, reruns verifiers, commits the change, prepares PR-ready evidence, and
marks the job `pr_ready`.

### AC-3: Recovery Replays Mission Checkpoints

Given a mission has a replay checkpoint and the background-job manifest is
recovered from stale `running`, when the recovered job runs, then it resumes
from the mission checkpoint and preserves earlier verifier evidence instead of
starting over silently.

### AC-4: Operator Status Is Complete Enough To Act

Given an autonomous coding job has run or recovered, when status is inspected,
then JSON/status files expose the background job id, mission id, phase, verifier
summary, changed files, resume command, PR-ready command, PR URL if present,
recovery count, replay count, and last reason code.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | temp repo, verifier commands, controlled edits | submit runs | mission/job/background records exist and are linked |
| C-02 | AC-2 | Functional | verifier requires `status.txt` and `docs/notes.txt` edits | run/replay executes | mission reaches `pr_ready`, commit exists, changed files include both files |
| C-03 | AC-3 | Regression | mission checkpoint plus stale recovered background manifest | recover and replay run | recovery count increments and mission resumes to `pr_ready` |
| C-04 | AC-4 | Conformance | completed or blocked autonomous coding job | status runs | JSON includes actionable operator fields with no missing ids |

## Success Signals

- Focused runtime tests pass for the autonomous coding job runtime.
- Dedicated CLI integration script passes over a disposable git repo.
- Existing background-job recovery tests remain green.
- `tau-unified status` can continue reading mission snapshots without schema
  breakage.
