# Spec 3808: Operator Recovery Evidence Surface

Status: Reviewed

## Problem Statement

`tau-unified` can already list, inspect, recover, replay, and block autonomous
coding jobs, but the operator surface drops some evidence that the durable
runtime already persists. Operators should not have to open raw JSON to see
background job ids, PR publication stdout/stderr, auto-merge commands, provider
repair context, or issue-intake clarifying questions.

## Scope

In:

- Extend `tau-unified status`, `jobs`, and `job` output with existing recovery
  and PR evidence fields.
- Add `tau-unified intakes` and `tau-unified intake <id>` views over persisted
  `issue-intake/*.json` records.
- Keep output line-oriented and grep-friendly like the existing launcher.
- Update operator docs.

Out:

- New runtime state fields or schema changes.
- Live provider calls.
- Graphical dashboard work.
- Auto-merge or branch-protection behavior changes.

## Acceptance Criteria

AC-1: Given a job status snapshot with background job id, PR publication
evidence, auto-merge command, and last background reason, when
`tau-unified status` runs, then the control-plane output includes those fields.

AC-2: Given multiple job status snapshots, when `tau-unified jobs` runs, then
each job includes the next safe command plus event-log, provider-repair context,
and mark-blocked command evidence.

AC-3: Given a job id, when `tau-unified job <job-id>` runs, then the detailed
view includes background id, PR publication stdout/stderr/exit status,
auto-merge command, provider-repair model/provider metadata, heartbeat/lease,
and last background reason.

AC-4: Given persisted issue-intake JSON, when `tau-unified intakes` runs, then
the list includes intake id, status, classification, decision, reason code,
question count, missing-input count, and next action.

AC-5: Given an intake id, when `tau-unified intake <intake-id>` runs, then the
detailed view includes the verifier plan, suggested verifier commands, required
authority, missing inputs, and clarifying questions.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Status fixture with PR/background evidence | `tau-unified status` emits the new control-plane fields |
| C-02 | AC-2 | Functional | Stale and PR-ready job status snapshots | `tau-unified jobs` emits evidence paths and safe commands |
| C-03 | AC-3 | Functional | PR-ready job id | `tau-unified job` emits PR/provider/background detail |
| C-04 | AC-4 | Functional | Persisted underspecified intake | `tau-unified intakes` emits decision and counts |
| C-05 | AC-5 | Functional | Persisted underspecified intake | `tau-unified intake` emits verifier plan and questions |

## Success Signals

- `bash scripts/run/test-tau-unified.sh status_contract` passes.
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passes.
- `git diff --check` passes.
