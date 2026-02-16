# Issue 1613 Spec

Status: Implemented

Issue: `#1613`  
Milestone: `#21`  
Parent: `#1606`

## Problem Statement

`crates/tau-tools/src/tools.rs` remains oversized and mixes unrelated tool
domains in one file, making policy/runtime changes difficult to review and
raising maintenance risk.

## Scope

In scope:

- split `tools.rs` into additional cohesive domain modules
- preserve existing tool behavior and registry wiring
- reduce `tools.rs` below the maintainability budget for this story

Out of scope:

- behavior changes to tool contracts
- policy semantics changes
- dependency changes

## Acceptance Criteria

AC-1 (line budget):
Given the refactored `tools.rs`,
when counting lines,
then `crates/tau-tools/src/tools.rs` is under `2,500` lines.

AC-2 (domain modules):
Given memory and jobs tool domains,
when reviewing module layout,
then these domains are moved to dedicated modules under
`crates/tau-tools/src/tools/` with root-level wiring preserved.

AC-3 (registry/runtime parity):
Given existing tool registration/runtime behavior,
when running scoped tests,
then builtin tool availability and core execution behavior remain unchanged.

AC-4 (quality gates):
Given scoped validation checks,
when running tests/lints/format/split harness,
then all pass with no new warnings.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given `tools.rs`, when checked, then line count is `< 2500`. |
| C-02 | AC-2 | Functional | Given source layout, when inspected, then `memory_tools.rs` and `jobs_tools.rs` exist under `crates/tau-tools/src/tools/` and root module imports them. |
| C-03 | AC-3 | Integration | Given `tau-tools` tests, when run, then tool registry names and memory/jobs tool behaviors remain parity. |
| C-04 | AC-4 | Regression | Given scoped checks, when running `cargo test -p tau-tools`, strict clippy, fmt, and split harness, then all pass. |

## Success Metrics

- `tools.rs` reduced below 2,500 lines
- memory/jobs tool domains extracted to dedicated modules
- `tau-tools` test suite remains green with strict linting
