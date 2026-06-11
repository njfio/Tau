# Spec 3809: Intake Run

Status: Implemented

## Problem Statement

`tau-unified intake <id>` can now print a safe rerun command for blocked intake,
but operators still have to copy and paste that command. Tau should provide a
first-class command that reads persisted intake, accepts the missing mutation or
provider authority, and runs the existing verifier-gated `issue-to-merge` loop.

## Scope

In:

- Add `tau-autonomous-coding-job intake-run` to load a persisted intake record.
- Add `tau-unified intake-run <intake-id>` as the operator wrapper.
- Require concrete verifier commands from intake or explicit
  `--verifier-command` answers.
- Require edit or provider repair authority at command time.
- Refuse unsafe, broad, vague, legacy, or placeholder-verifier intake.
- Keep draft PR as the safe default and preserve guarded auto-merge flags.

Out:

- Live provider calls during intake inspection.
- Auto-answering vague clarifying questions.
- Bypassing branch protection or using admin merge flags.
- New persisted intake schema fields.

## Acceptance Criteria

AC-1: Given a `needs_authority` intake with stored issue body and concrete
verifier commands, when `intake-run` is invoked with provider repair authority,
then it runs `issue-to-merge` with the persisted issue fields and verifier
commands.

AC-2: Given a `needs_authority` intake without concrete verifier commands, when
`intake-run` is invoked with an explicit `--verifier-command`, then it uses that
operator-supplied verifier.

AC-3: Given no edit/provider authority, unsafe/broad/vague intake, missing issue
body, or placeholder verifier commands, when `intake-run` is invoked, then it
fails before mutation with a specific error.

AC-4: Given `tau-unified intake-run <intake-id>`, when state dirs and authority
flags are supplied, then it delegates to `tau-autonomous-coding-job intake-run`
without requiring the operator to hand-write state paths.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Unit/Functional | Needs-authority intake with concrete verifier and provider flag | Builds issue-to-merge request from persisted fields |
| C-02 | AC-2 | Unit | Missing-verifier intake plus explicit verifier flag | Uses explicit verifier command |
| C-03 | AC-3 | Unit | Vague or placeholder intake | Fails before mutation |
| C-04 | AC-4 | Contract | `tau-unified intake-run` with fake job CLI | Delegates state dirs, intake id, and authority flags |

## Success Signals

- Focused `tau-coding-agent` binary tests cover C-01..C-03.
- `bash scripts/run/test-tau-unified.sh status_contract` covers C-04.
- `cargo fmt --check`, shell syntax, clippy, oversized-file guard, roadmap sync,
  and diff hygiene pass.
