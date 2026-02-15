# Issue 1690 Spec

Status: Implemented

Issue: `#1690`  
Milestone: `#23`  
Parent: `#1623`

## Problem Statement

`tau-tools` and `tau-runtime` were split into focused modules, but key files
still lack top-level `//!` boundary docs and explicit notes on safety/policy
interactions and reason-code/error contracts. This increases onboarding and
review friction in high-churn runtime paths.

## Scope

In scope:

- add module-level `//!` docs to selected split modules in `tau-tools` and
  `tau-runtime`
- document policy/safety interaction points
- document reason-code/error contract expectations for operators

Out of scope:

- behavioral/runtime logic changes
- broad rewrite of all crate documentation

## Acceptance Criteria

AC-1 (module boundary docs):
Given selected split runtime/tool modules,
when files are opened,
then top-level `//!` docs describe responsibility and boundaries.

AC-2 (policy/safety interactions):
Given tool and runtime policy modules,
when docs are reviewed,
then safety/policy integration points are explicit.

AC-3 (error/reason-code semantics):
Given modules that emit diagnostics/outcomes,
when docs are reviewed,
then reason-code/error contract expectations are documented.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given patched modules, when inspecting file headers, then `//!` boundary docs are present. |
| C-02 | AC-2 | Functional | Given `tool_policy*` and runtime policy-related files, when reviewing docs, then policy/safety interaction points are documented. |
| C-03 | AC-3 | Conformance | Given runtime output/health/transport modules, when reviewing docs, then error/reason-code contract notes are present. |
| C-04 | AC-1, AC-2, AC-3 | Regression | Given docs and helper checks, when tests run, then docs-link and quality-remediation contracts remain green. |

## Success Metrics

- selected high-churn runtime/tool modules have clear boundary docs
- policy/safety and reason-code semantics are easier to trace for operators
