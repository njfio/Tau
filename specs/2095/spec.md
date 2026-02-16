# Spec #2095

Status: Implemented
Milestone: specs/milestones/m27/index.md
Issue: https://github.com/njfio/Tau/issues/2095

## Problem Statement

Story M27.1 tracks extraction of CLI execution-domain flag groups out of
`crates/tau-cli/src/cli_args.rs` to improve maintainability while preserving
flag/runtime behavior. Task `#2096` completed the scoped delivery via subtask
`#2097` and merged PRs `#2100` / `#2099`.

## Acceptance Criteria

- AC-1: Execution-domain flag group is extracted into dedicated module(s).
- AC-2: CLI parsing/runtime compatibility is preserved for migrated fields.
- AC-3: Story-level validation links ACs to task/subtask conformance evidence.

## Scope

In:

- consume merged task/subtask outputs from `#2096` and `#2097`
- capture story-level AC -> conformance mapping
- publish closure artifacts and evidence for hierarchy roll-up

Out:

- additional domain extractions beyond M27.1 scope
- semantic changes to CLI event/startup command behavior

## Conformance Cases

- C-01 (AC-1, integration):
  `crates/tau-cli/src/cli_args/execution_domain_flags.rs` exists and is wired
  from `cli_args.rs`.
- C-02 (AC-2, functional):
  `cargo check -p tau-cli --lib --target-dir target-fast` passes.
- C-03 (AC-2, regression):
  `bash scripts/dev/test-cli-args-domain-split.sh` passes.
- C-04 (AC-2, integration):
  `cargo test -p tau-coding-agent startup_preflight_and_policy --target-dir target-fast`
  passes.

## Success Metrics

- Story issue `#2095` closes with linked task/subtask evidence.
- `specs/2095/{spec,plan,tasks}.md` lifecycle is complete.
- Epic `#2094` roll-up is unblocked.
