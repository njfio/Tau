Status: Reviewed

# Spec 3806: Operator Recovery, Hands-Off Intake, and Draft PR Loop

## Problem

Tau has durable autonomous coding jobs, provider repair, replay/recover/status,
and guarded PR behavior. The product still has three rough edges:

- `tau-unified` exposes status markers but not an operator flow for listing,
  inspecting, recovering, replaying, or blocking jobs.
- Issue intake can classify missing authority, but it does not generate a
  concrete verifier plan for underspecified or no-authority issues.
- Draft PR publication works, but GitHub-auth success/fallback evidence needs to
  be explicit enough for a hands-off job to stop safely.

## Scope

In scope:
- `tau-unified` autonomous coding job commands for list, inspect, recover,
  replay, and mark-blocked.
- Operator summaries that explain why a job is or is not safe to resume.
- Deterministic issue-intake verifier-plan generation and required-input
  details.
- Draft PR publication evidence that records real `gh pr create --draft`
  attempts when GitHub auth is available and exact manual commands when it is
  not.

Out of scope:
- Bypassing branch protection or using admin merge flags.
- Letting vague issues mutate code without verifier plus edit/provider
  authority.
- Provider-generated verifier plans from live LLM calls.
- A graphical dashboard rewrite.

## Acceptance Criteria

### AC-1: Operators can list autonomous coding jobs

Given an autonomous coding state directory with multiple jobs
When `tau-unified jobs` is run
Then it must list each job with status, operator state, PR state, replay safety,
recoverability, reason code, and next command.

### AC-2: Operators can inspect job evidence from one command

Given a durable autonomous coding job
When `tau-unified job <job-id>` is run
Then it must show status, verifier state, provider repair state, PR state, event
log path, changed files, and an explanation for why replay/recover is safe or
unsafe.

### AC-3: Operators can act from tau-unified

Given a job id
When `tau-unified recover`, `tau-unified replay <job-id>`, or
`tau-unified block <job-id>` is run
Then the command must delegate to `tau-autonomous-coding-job`, return the JSON
evidence, and not require the operator to hand-write state paths.

### AC-4: Intake generates a verifier plan

Given issue intake without verifier/edit/provider authority
When Tau classifies the issue
Then it must persist a deterministic verifier plan, missing inputs, required
authority, and a next-action summary tailored to docs, CLI, Rust/test, broad,
unsafe, or vague issues.

### AC-5: Hands-off intake remains fail-closed

Given an unsafe, too-broad, underspecified, or no-authority issue
When `issue-to-merge` lacks safe verifier/edit/provider authority
Then Tau must block before mutation and explain the exact missing verifier,
scope, credential, edit, or provider inputs.

### AC-6: Draft PR publication has explicit GitHub evidence

Given a job reaches PR-ready in draft mode
When GitHub auth and `gh` are available
Then Tau must attempt `gh pr create --draft`, record stdout/stderr/exit status,
and expose the PR URL when created.

Given GitHub auth or `gh` is missing
When a job reaches PR-ready in draft mode
Then Tau must remain PR-ready, record draft publication as manual-ready or
draft-failed with reason, and expose the exact manual `gh pr create --draft`
command.

## Conformance Cases

- C-01 maps AC-1/AC-2: `tau-unified jobs` and `tau-unified job` summarize a
  stale/replay-safe job and explain the next safe command.
- C-02 maps AC-3: `tau-unified block <job-id>` marks a job blocked through the
  autonomous coding CLI and records operator evidence.
- C-03 maps AC-4/AC-5: docs-only issue intake without authority persists a docs
  verifier plan and blocks without creating a job.
- C-04 maps AC-4/AC-5: vague or broad issue intake persists exact missing scope
  and verifier inputs.
- C-05 maps AC-6: draft PR publication with fake GitHub auth records a
  `draft_created` PR URL.
- C-06 maps AC-6: draft PR publication without GitHub auth records the manual
  draft PR command and does not fail the verified job.

## Success Signals

- Focused `tau-runtime` tests cover issue intake planning and draft PR
  publication fallback/success.
- `scripts/run/test-tau-unified.sh` covers the operator job commands.
- `cargo clippy -p tau-runtime --lib --tests -- -D warnings` passes.
- `cargo clippy -p tau-coding-agent --bin tau_autonomous_coding_job -- -D warnings` passes.
