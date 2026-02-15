# Issue 1689 Spec

Status: Implemented

Issue: `#1689`  
Milestone: `#23`  
Parent: `#1623`

## Problem Statement

`tau-coding-agent` is one of the largest runtime crates and still contains many
split modules without top-level `//!` docs. This obscures runtime boundaries,
command contracts, and failure reason semantics during operator debugging.

## Scope

In scope:

- add module-level `//!` docs to undocumented files in `tau-coding-agent/src`
- document runtime-loop/dispatch/startup boundary contracts
- document command/macro/profile and transport mode contract semantics
- document failure/diagnostic expectations for channel/runtime surfaces

Out of scope:

- behavior changes
- CLI/protocol changes
- dependency changes

## Acceptance Criteria

AC-1 (runtime boundaries):
Given runtime/startup modules,
when headers are inspected,
then phase boundaries and orchestration contracts are documented.

AC-2 (command contracts):
Given command/tool/profile modules,
when docs are read,
then command contract and persistence invariants are explicit.

AC-3 (failure semantics):
Given channel/transport/rpc modules,
when docs are read,
then failure/diagnostic semantics are documented.

AC-4 (regression safety):
Given scoped checks,
when tests/docs checks run,
then no regressions are introduced.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given targeted `tau-coding-agent` files, when scanning for `//!`, then no gap files remain. |
| C-02 | AC-2 | Conformance | Given command/tool/profile headers, when inspected, then contract invariants are explicit. |
| C-03 | AC-3 | Conformance | Given channel/transport/rpc headers, when inspected, then failure semantics are explicit. |
| C-04 | AC-4 | Regression | Given `tau-coding-agent` tests/docs checks, when run, then all pass. |

## Success Metrics

- zero missing module headers in targeted `tau-coding-agent` files
- clearer runtime/command/failure contracts at module boundaries
