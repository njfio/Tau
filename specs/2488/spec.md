# Spec #2488 - RED/GREEN conformance for G16 phase-2 profile-policy watcher reload

Status: Implemented

## Problem Statement
Task #2487 must provide explicit RED/GREEN proof that watcher-driven TOML hot-reload behavior fails before implementation and passes after implementation.

## Acceptance Criteria
### AC-1 RED evidence captured
Given C-01..C-04 tests are added before implementation, when scoped `spec_2487` tests run, then at least one test fails.

### AC-2 GREEN evidence captured
Given implementation lands, when the same scoped tests rerun, then all C-01..C-04 pass.

## Scope
In scope:
- RED/GREEN command evidence for `spec_2487` test set.

Out of scope:
- Additional behavior beyond #2487.

## Conformance Cases
- C-01 (AC-1): pre-implementation scoped run fails.
- C-02 (AC-2): post-implementation scoped run passes.
