# Spec: Issue #3756 - Mission harness operator UI and TUI proof view

Status: Implemented

## Problem Statement

Tau can run a deterministic autonomous harness proof, but operators do not have
a first-class UI/TUI surface for mission state, benchmark proof, verification
gates, and conservative self-improvement review. The dashboard should follow the
provided three-window Tau Agent Harness template and expose machine-testable
contract markers for the real harness concepts.

## Scope

In scope:

- `/ops/harness` dashboard route and gateway wiring
- dashboard layout markers for mission dashboard, proof view, self-improvement
  review/apply flow, benchmark runner, policy gates, and TUI companion
- deterministic benchmark POST that writes the latest M334 proof artifact
- proposal approve/reject/dry-run POST endpoints with gateway audit records
- proposal diff route for review flow
- TUI operator shell harness summary panel
- CSS-only first-viewport browser presentation matching the supplied dark
  operator harness template
- regression tests for route visibility, gateway reachability, and TUI output

Out of scope:

- new dependencies or JS frontend runtime
- unattended self-improvement apply
- provider-backed or queue-backed benchmark execution from the web form

## Acceptance Criteria

### AC-1 The dashboard exposes a first-class mission harness route

Given an operator opens `/ops/harness`,
when the Tau Ops shell renders,
then the shell shows Mission Harness navigation, breadcrumb, three template
windows, KPI cards, active missions, and M334 benchmark proof controls.

### AC-2 The proof view exposes mission evidence and verification gates

Given the harness route is rendered,
when an operator inspects the mission detail area,
then they can see run id, goal, plan DAG, acceptance criteria, tool evidence,
memory/learning evidence, verification gates, artifacts, and operator log.

### AC-3 The self-improvement flow remains conservative and approval-gated

Given the self-improvement review area is rendered,
when an operator inspects a proposal,
then allowed targets are limited to skill/config/prompt, source code and safety
policy are marked blocked, proposal actions record audit entries, and apply
remains approval-gated.

### AC-4 The TUI exposes the same benchmark proof summary

Given the operator shell TUI renders its deterministic fixture,
when the harness summary is present,
then it shows benchmark id, mission id, transport, skill, status, pass count,
failed gates, and proof artifact.

### AC-5 Non-harness routes do not show the harness panel as active

Given another ops route is rendered,
when the harness panel is present for DOM stability,
then it is hidden and marked not visible.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | `/ops/harness` render | shell route renders | template panels and navigation markers are visible |
| C-02 | AC-2, AC-3 | Functional | `/ops/harness` render | controls are inspected | benchmark, gate, policy, and approval markers are present |
| C-03 | AC-5 | Regression | `/ops` render | shell route renders | harness panel is hidden and command center remains active |
| C-04 | AC-1..AC-3 | Integration | gateway test server | GET `/ops/harness` | route resolves with benchmark and apply markers |
| C-05 | AC-1..AC-3 | Integration | gateway test server | POST benchmark/proposal actions | proof and audit artifacts are written; direct apply is blocked |
| C-06 | AC-4 | Unit | deterministic TUI fixture | shell renderer runs | harness summary lines are present |

## Success Metrics / Observable Signals

- `/ops/harness` returns `200 OK` from the gateway test server.
- `/ops/harness/run-benchmark` writes `ops-harness/m334/latest.json`.
- Proposal action endpoints append `ops-harness/audit.jsonl`.
- Direct proposal apply returns approval-required rejection instead of mutating.
- Dashboard tests assert the template windows and safe apply controls.
- Browser smoke shows the dark harness workspace in the first viewport.
- TUI tests assert M334 proof summary output.
- No new dependency or production-side mutation path is introduced.
