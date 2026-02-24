# Spec: Issue #3454 - M299 close E2E scenario gaps M7/X9/D12

Status: Implemented

## Problem Statement
Review artifacts still report 3 pending E2E PRD scenario groups (M7 memory
graph/persistence, X9 cortex bulletin/cross-session, D12 dashboard live data).
Without explicit deterministic conformance matrices for these groups, the
claimed "15/15 scenario-group coverage" is incomplete.

## Scope
In scope:
- Add deterministic E2E conformance tests for M7, X9, and D12 in
  `tau-gateway` integration suites.
- Cover representative lifecycle behavior for each group using scripted
  provider responses and isolated workspace state.
- Wire AC-to-test mapping and verification evidence into issue artifacts.

Out of scope:
- Re-architecting dashboard/cortex/memory runtime production code.
- Live third-party provider/network validation.
- Expanding partial groups outside M7/X9/D12 in this issue.

## Acceptance Criteria
### AC-1 Artifact contract exists for M299 delivery slice
Given issue #3454 is in implementation,
when milestone and issue spec artifacts are inspected,
then `specs/milestones/m299/index.md` and `specs/3454/{spec,plan,tasks}.md`
exist and map ACs to conformance cases.

### AC-2 M7 memory graph/persistence conformance is executable and deterministic
Given an isolated gateway workspace with scripted behavior,
when the M7 conformance test matrix runs,
then memory write/read/update/delete, search scoping/type filters, and memory
graph response contracts pass with deterministic assertions.

### AC-3 X9 cortex bulletin/cross-session conformance is executable and deterministic
Given cortex runtime endpoints and scripted LLM responses,
when the X9 conformance matrix runs,
then cortex chat/status, deterministic fallback, and bulletin injection behavior
are validated with stable assertions.

### AC-4 D12 dashboard live-data conformance is executable and deterministic
Given dashboard health/widget/alert/timeline endpoints and stream endpoint,
when the D12 conformance matrix runs,
then status/read-model payloads and stream-event contracts pass with
deterministic assertions.

### AC-5 Verification evidence is complete and auditable
Given implementation PR verification,
when RED/GREEN/REGRESSION commands are executed,
then evidence is recorded in `specs/3454/tasks.md` with completed tier matrix.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M299 issue hierarchy | inspect artifact paths | required milestone/issue files exist |
| C-02 | AC-2 | Integration | isolated workspace + memory endpoints | run M7 matrix test | CRUD + graph + scoping/type filters pass |
| C-03 | AC-3 | Integration | cortex endpoints + scripted LLM | run X9 matrix test | chat/status/fallback/bulletin assertions pass |
| C-04 | AC-4 | Functional/Integration | dashboard endpoints + stream endpoint | run D12 matrix test | status payloads + stream events pass |
| C-05 | AC-5 | Regression | PR verification commands | execute RED->GREEN->REGRESSION loop | evidence is complete and traceable |

## Success Metrics / Observable Signals
- M7/X9/D12 conformance tests pass in CI deterministic mode.
- Review gap statements for pending M7/X9/D12 groups are no longer true.
- AC->conformance->test traceability is complete in issue artifacts and PR body.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `ls -1 specs/milestones/m299/index.md specs/3454/spec.md specs/3454/plan.md specs/3454/tasks.md` confirms required artifact package exists. |
| AC-2 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3454_c02_m7_memory_graph_and_persistence_matrix -- --nocapture` passed; validates M7 CRUD/search filter/graph contracts. |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3454_c03_x9_cortex_bulletin_and_cross_session_matrix -- --nocapture` passed; validates cortex chat/status/fallback/bulletin injection/context contracts. |
| AC-4 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3454_c04_d12_dashboard_live_data_stream_matrix -- --nocapture` passed; validates dashboard live-data route and stream contracts. |
| AC-5 | ✅ | RED/GREEN/REGRESSION evidence captured in `specs/3454/tasks.md`, including `cargo fmt --check`, `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`, and scoped mutation campaign (`cargo mutants ...`) with `6/6` caught in final run. |
