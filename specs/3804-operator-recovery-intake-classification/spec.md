Status: Reviewed

# Spec 3804: Operator Recovery and Issue Intake Classification

## Problem

Autonomous coding jobs persist heartbeat, lease, checkpoints, events, repair
evidence, and status. Operators still need that state translated into product
behavior: running, stale lease, recoverable, blocked, needs authority, safe to
replay, and the exact command to act. Issue intake also needs specific
classification rather than a generic "authority required" block.

## Scope

In scope:
- Add operator classification fields to autonomous coding job status JSON.
- Add a safe `mark-blocked` command for jobs an operator has inspected.
- Surface operator classification fields through `tau-unified status`.
- Add issue intake classifications for unsafe, too broad, underspecified,
  missing verifier, missing edit/provider authority, missing credentials, and
  solvable.

Out of scope:
- Automatic replay execution from `tau-unified`.
- Admin or protected-branch bypass.
- Live provider credential probing during static issue intake.

## Acceptance Criteria

### AC-1: Job status classifies recovery state

Given a durable autonomous coding job
When status is refreshed
Then the status JSON must include `operator_state`, `operator_next_command`,
`replay_safe`, `recoverable`, `needs_authority`, `stale_lease`, and
`mark_blocked_command`.

### AC-2: Stale leases are actionable

Given a running/recovering/queued job with an expired lease
When status is refreshed
Then `operator_state` is `stale_lease`
And `recoverable` is true
And `operator_next_command` points at `tau-autonomous-coding-job recover`.

### AC-3: Operators can mark a job blocked

Given an inspected job
When `tau-autonomous-coding-job mark-blocked` is run
Then the job status is `blocked`, the lease is cleared, an event is appended,
and the status includes the supplied reason/detail.

### AC-4: Issue intake classifies blockers

Given issue intake lacks required authority or contains unsafe/broad/vague input
When Tau persists the intake plan
Then it must store a specific classification and required inputs rather than a
generic blocker.

### AC-5: Unified status exposes the operator contract

Given a status JSON with operator fields
When `tau-unified status` runs
Then the control-plane output must include those fields as stable markers.

## Conformance Cases

- C-01 maps AC-1/AC-2/AC-3: a stale running job is classified as
  `stale_lease`, then `mark-blocked` turns it into a blocked job with operator
  evidence.
- C-02 maps AC-4: no-authority issue intake is classified as
  `missing_verifier` and lists verifier/edit authority requirements.
- C-03 maps AC-4: issue-to-merge with verifier but no edit/provider authority is
  classified as `missing_edit_or_provider_authority`.
- C-04 maps AC-4: unsafe merge/bypass text is classified as `unsafe`.
- C-05 maps AC-5: `scripts/run/test-tau-unified.sh` asserts all new operator
  markers.

## Success Signals

- Focused `tau-runtime` tests for C-01..C-04 pass.
- `scripts/run/test-tau-unified.sh` passes.
- `cargo clippy -p tau-runtime --lib --tests -- -D warnings` passes.
