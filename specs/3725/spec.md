# Spec: Issue #3725 - Define tranche-one autonomy benchmark contract and task set

Status: Implemented

## Problem Statement
Tau has multiple real runtime loops, but it still lacks a durable benchmark
contract that says what tranche-one autonomy must actually accomplish. Without a
versioned benchmark task set, later mission-mode, autopilot, and operator UX
work can claim autonomy progress without proving the user outcome the product
now needs to hit.

## Scope
In scope:
- a machine-readable tranche-one autonomy benchmark fixture under `tasks/fixtures/m334/`
- a benchmark suite of `3-5` complex tasks with explicit categories
- allowed and disallowed operator intervention rules for tranche one
- task-level deliverables and pass requirements that later runtime/result
  classification work can execute against
- milestone update in `specs/milestones/m334/index.md`
- spec/plan/tasks updates under `specs/3725/`

Out of scope:
- runtime benchmark execution harnesses
- mission result classification adapters
- TUI/dashboard/operator UX changes
- self-improvement / RL productionization
- multi-channel expansion work

## Acceptance Criteria
### AC-1 The tranche-one benchmark fixture defines a bounded, representative task set
Given the tranche-one autonomy benchmark contract lands,
when maintainers inspect the benchmark fixture,
then it defines `3-5` complex tasks and spans the tranche-one categories Tau
must prove:
- one governed repo/spec-to-PR build task
- one greenfield build task
- one research/design task
- one data-to-deliverable task

### AC-2 The benchmark fixture encodes the tranche-one intervention model
Given the benchmark fixture,
when maintainers inspect suite policy,
then it allows only provider-auth friction and operator checkpoints for
materially different solution directions, and it explicitly rejects routine
human steering as part of success.

### AC-3 Each benchmark task encodes durable pass expectations for later runtime proof work
Given the benchmark fixture,
when maintainers inspect any benchmark task,
then it includes a mission goal, required deliverables, allowed checkpoint
classes, and pass requirements that later mission/runtime classification work
can measure.

### AC-4 The Ralph-loop milestone index links this benchmark-definition slice
Given M334's issue hierarchy,
when maintainers inspect `specs/milestones/m334/index.md`,
then it includes Issue `#3725` as the benchmark-contract task under the Ralph
loop architecture milestone.

## Conformance Cases
- C-01 The benchmark fixture parses successfully and declares exactly four tasks
  with categories `repo_build`, `greenfield_build`, `research_design`, and
  `data_to_deliverable`. Maps to AC-1. Tier: Conformance.
- C-02 The suite policy allows only `provider_auth` and
  `major_direction_choice`, and rejects routine guidance as a success path.
  Maps to AC-2. Tier: Conformance.
- C-03 Every benchmark task includes `goal`, `required_deliverables`,
  `allowed_checkpoints`, and `pass_requirements`. Maps to AC-3.
  Tier: Conformance.
- C-04 The M334 milestone index includes Issue `#3725` in the issue hierarchy.
  Maps to AC-4. Tier: Functional.

## Success Metrics / Observable Signals
- Tau now has a versioned, repo-local benchmark contract for tranche-one
  autonomy instead of a prose-only success bar
- The benchmark suite reflects the user's priority order: autonomy first,
  usefulness second, self-improvement later
- Later mission-mode and autopilot work can target a stable benchmark fixture
  instead of re-deciding success every time

## Key Decisions
- Choose four tasks rather than the minimum three so the suite covers both the
  bounded repo usefulness domain and the broader complex-task examples from the
  interview.
- Keep the first slice declarative: benchmark definition first, execution and
  classification later.
