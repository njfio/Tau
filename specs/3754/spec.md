# Spec: Issue #3754 - Close Tau autonomous harness integration gaps

Status: Implemented

## Problem Statement

Tau has a shared mission contract and conservative improvement records, but the
product still lacks an operator-runnable end-to-end harness from `master`.
`tau-coding-agent` self-modification dry-runs are not projected into mission
improvement proposals, the M334 autonomy benchmark is only a fixture/schema
contract, and approved skill/config/prompt apply is not executable.

## Scope

In scope:

- deterministic autonomous harness runner that emits mission proof snapshots
- canonical M334 benchmark fixture integration as the proof corpus
- mission proof requirements for planning, tool execution, memory write,
  verification, learning, and artifacts
- `tau-coding-agent` adapter from self-modification dry-run output into
  `MissionImprovementProposal` evidence
- approval-gated apply mechanics for skill/config/prompt targets only
- operator-runnable CLI surface for local proof without provider credentials

Out of scope:

- autonomous source-code or safety-policy mutation
- unattended production self-modification
- live provider benchmark execution
- broad gateway/dashboard/channel integration

## Acceptance Criteria

### AC-1 A harness mission can run end-to-end without provider credentials

Given a benchmark task definition,
when the deterministic harness runner executes it,
then it creates a `MissionSnapshot` that reaches `Completed` with a valid plan
DAG, tool evidence, memory recall evidence, verification gates, artifact
references, and final learning output.

### AC-2 The canonical autonomy benchmark is executable proof input

Given `tasks/fixtures/m334/tranche-one-autonomy-benchmark.json`,
when the benchmark harness runs,
then every task in the fixture produces a completed mission proof and the suite
summary reports required categories, terminal state, and intervention policy.

### AC-3 Self-modification dry-runs project into mission improvement proposals

Given a failure learning record and a safe skill/config/prompt target,
when `tau-coding-agent` runs the self-modification dry-run adapter,
then the mission records a proposal plus dry-run, test, and safety evidence
without applying changes.

### AC-4 Approved improvements can be applied through an executable safe path

Given a mission improvement proposal with dry-run, test, safety, and operator
approval evidence,
when the apply helper runs for a skill/config/prompt target,
then it writes the requested file content under the workspace, records rollback
metadata, marks the proposal applied, and updates curator memory linkage.

### AC-5 Unsafe apply targets remain blocked

Given a source-code, safety-policy, path-traversal, absolute-path, or
outside-workspace target,
when the dry-run adapter or apply helper evaluates it,
then it fails closed before writing files or mutating mission proposal state.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Unit | one deterministic harness task | mission runner executes | mission completes with plan/tool/memory/verification/artifact/learning proof |
| C-02 | AC-2 | Functional | M334 benchmark fixture | suite runner executes | every task completes and suite categories/interventions are preserved |
| C-03 | AC-3 | Integration | failure learning record and safe skill target | dry-run adapter runs | mission proposal and dry-run/test/safety evidence are recorded |
| C-04 | AC-4 | Integration | approved skill proposal | apply helper runs | file content changes and mission/curator metadata are applied |
| C-05 | AC-5 | Regression | source/outside target | adapter/apply helper runs | deterministic error occurs before write or mission mutation |

## Success Metrics / Observable Signals

- Operator command emits a JSON benchmark proof artifact with completed missions.
- Tests prove the full fixture can run without network/provider credentials.
- Mission snapshots contain non-empty proof for plan, tools, memory,
  verification, artifacts, and learning.
- Self-improvement remains conservative: skill/config/prompt only, approval
  required before apply, rollback metadata recorded.
