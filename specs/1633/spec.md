# Issue 1633 Spec

Status: Implemented

Issue: `#1633`  
Milestone: `#21`  
Parent: `#1613`

## Problem Statement

`crates/tau-tools/src/tools.rs` remains oversized (4,531 lines), exceeding maintainability budget and obscuring domain boundaries despite prior partial splits.

## Scope

In scope:

- split a large helper-domain block from `tools.rs` into a focused submodule under `crates/tau-tools/src/tools/`
- keep tool behavior and public API stable
- verify tools parity tests remain green after extraction
- reduce `tools.rs` below the 4,000-line target

Out of scope:

- redesigning tool behavior/policies
- changing CLI flags/protocols
- adding dependencies

## Acceptance Criteria

AC-1 (size target):
Given `crates/tau-tools/src/tools.rs`,
when line count is measured,
then it is below 4,000 lines.

AC-2 (domain extraction):
Given extracted helper logic,
when reviewing module layout,
then helper functions are moved to a dedicated submodule and imported by `tools.rs`.

AC-3 (behavior parity):
Given post-split implementation,
when running tool parity checks,
then no functional regressions occur.

AC-4 (verification):
Given issue-scope checks,
when run,
then split harness, targeted tau-tools tests, fmt, and clippy pass.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given `tools.rs`, when measured in split harness, then line count < 4000. |
| C-02 | AC-2 | Functional | Given module tree, when inspected by split harness, then `tools/runtime_helpers.rs` exists and is wired by `mod runtime_helpers`. |
| C-03 | AC-3 | Regression | Given targeted tool parity tests, when executed, then all selected parity tests pass. |
| C-04 | AC-4 | Integration | Given issue-scope commands, when run, then split harness + scoped quality checks pass. |

## Success Metrics

- `tools.rs` drops below 4k lines without parity regressions
- extracted helper-domain is isolated in a named submodule
