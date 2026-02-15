# Issue 1705 Spec

Status: Accepted

Issue: `#1705`  
Milestone: `#22`  
Parent: `#1700`

## Problem Statement

M22 requires a deterministic terminology scan proving stale RL wording is
eliminated from current-state docs/help while preserving approved forward-looking
references. Current scan output still reports stale findings, blocking milestone
evidence.

## Scope

In scope:

- run and refresh M22 terminology scan artifacts
- classify findings as approved vs stale using allowlist policy
- remediate stale wording in docs/help or policy context rules
- publish resulting scan report artifacts for gate evidence

Out of scope:

- broad rewrite of historical planning docs unrelated to current stale findings
- new runtime feature behavior

## Acceptance Criteria

AC-1 (scan execution):
Given repository source at current head,
when terminology scan runs,
then JSON + Markdown reports are generated under `tasks/reports/`.

AC-2 (stale elimination):
Given scan findings,
when remediation is complete,
then no unapproved stale RL phrasing remains (`stale_count == 0`).

AC-3 (policy/documentation consistency):
Given allowlist policy and terminology guide,
when contract checks run,
then approved contexts and migration wording remain coherent and test-backed.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given scanner script, when executed with repo defaults, then `m22-rl-terminology-scan.json/.md` are emitted. |
| C-02 | AC-2 | Conformance | Given completed remediation, when scanner report is inspected, then `summary.stale_count` equals `0`. |
| C-03 | AC-3 | Integration | Given policy/guide updates, when allowlist contract tests run, then schema and guide alignment checks pass. |
| C-04 | AC-3 | Regression | Given scanner fixture test, when run, then approved/stale classification behavior remains deterministic. |
| C-05 | AC-1 | Regression | Given invalid scanner invocation, when executed, then deterministic non-zero error behavior remains. |

## Success Metrics

- M22 scan report produced and linked with `stale_count=0`
- policy/guide/contracts remain passing after remediation
