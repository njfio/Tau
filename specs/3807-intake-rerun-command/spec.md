# Spec 3807: Intake Rerun Command

Status: Implemented

## Problem Statement

Tau can persist verifier plans and structured clarifying questions for blocked
issue intake, but `tau-unified intake` still leaves the operator to manually
reconstruct the next `issue-to-merge` command. That is brittle because the
command needs the original issue body, concrete verifier command, repo path,
state directory, and safe provider/edit authority.

## Scope

In:

- Persist the original issue body in intake records, with legacy JSON loading
  defaulting to an empty body.
- Have `tau-unified intake <id>` print a shell-quoted rerun command when the
  intake is `needs_authority`, has the original issue body, and has concrete
  verifier commands.
- Use the existing OpenRouter provider-repair flag in the suggested rerun
  command; do not auto-merge.
- Keep vague, unsafe, broad, missing-verifier, or placeholder-verifier intakes
  blocked without a rerun command.

Out:

- Live provider calls during intake inspection.
- New persisted command fields.
- Auto-merge or branch-protection behavior.
- Inferring verifier commands from prose placeholders.

## Acceptance Criteria

AC-1: Given an intake record with `decision=needs_authority`, an original issue
body, and concrete verifier commands, when `tau-unified intake <id>` runs, then
it prints `rerun_command` with a shell-quoted
`tau-autonomous-coding-job issue-to-merge` command containing state dir, intake
id, repo path, issue URL/title/body, base branch, verifier command, allowed root,
OpenRouter provider repair, and draft PR mode.

AC-2: Given an underspecified intake or an intake without a concrete verifier
command, when `tau-unified intake <id>` runs, then it prints
`rerun_command=none`.

AC-3: Given a newly recorded intake, when it is persisted, then the original
issue body is stored in the intake JSON.

AC-4: Given legacy intake JSON without `issue_body`, when Tau loads it, then it
defaults to an empty body without breaking old records.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Needs-authority intake with concrete Rust verifier | `tau-unified intake` emits shell-quoted rerun command |
| C-02 | AC-2 | Functional | Underspecified intake with questions | `rerun_command=none` |
| C-03 | AC-3 | Conformance | Runtime intake request with issue body | Persisted status contains full `issue_body` |
| C-04 | AC-4 | Regression | Legacy intake JSON without `issue_body` | Deserializes with empty issue body |

## Success Signals

- `bash scripts/run/test-tau-unified.sh status_contract` passes.
- Focused `tau-runtime` tests cover persisted `issue_body` and legacy default.
- `cargo fmt --check`, shell syntax checks, clippy, roadmap sync, oversized-file
  guard, and diff hygiene pass.
