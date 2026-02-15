# Issue 1756 Spec

Status: Accepted

Issue: `#1756`  
Milestone: `#22`  
Parent: `#1706`

## Problem Statement

Legacy training flag aliases from pre-M22 naming remain in operator muscle
memory. Current behavior rejects these flags outright, producing generic clap
errors and no migration guidance. We need deterministic compatibility warnings
with stable text so operators can continue workflows while migrating to
canonical prompt-optimization flags.

## Scope

In scope:

- legacy alias normalization for renamed prompt-optimization CLI flags
- deterministic deprecation warning text output for each legacy alias used
- test coverage that locks warning wording and command-level alias behavior

Out of scope:

- adding new CLI feature flags unrelated to prompt-optimization rename
- removing canonical prompt-optimization flags
- changing non-training alias behavior

## Acceptance Criteria

AC-1 (compatibility):
Given a supported legacy flag alias,
when CLI arguments are normalized,
then the alias maps to canonical prompt-optimization flags and parsing succeeds.

AC-2 (warning text stability):
Given one or more legacy aliases in input,
when normalization runs,
then deterministic deprecation warning strings are emitted with canonical
replacement guidance.

AC-3 (regression safety):
Given unknown/unsupported flags,
when parsing runs,
then failure behavior remains fail-closed and deterministic.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given `--train-config`, when args normalize + parse, then CLI has `prompt_optimization_config` populated. |
| C-02 | AC-1 | Functional | Given `--training-proxy-server --training-proxy-bind`, when args normalize + parse, then proxy canonical fields are set. |
| C-03 | AC-2 | Conformance | Given legacy aliases, when normalization runs, then warning messages match stable expected strings. |
| C-04 | AC-2 | Regression | Given `--legacy=value` form, when normalization runs, then replacement keeps value and warning text remains stable. |
| C-05 | AC-3 | Regression | Given unknown flag, when parse runs, then it still errors with deterministic clap failure message. |

## Success Metrics

- legacy rename aliases are compatible for M22 migration workflows
- warning text is deterministic and locked by tests
- unsupported flags remain fail-closed
