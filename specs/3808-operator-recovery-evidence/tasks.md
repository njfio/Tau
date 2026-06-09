# Tasks 3808: Operator Recovery Evidence Surface

- [x] T1 (RED): Add failing `tau-unified status_contract` assertions for
  recovery evidence and intake views.
- [x] T2 (GREEN): Project existing job evidence fields through
  `tau-unified status`, `jobs`, and `job`.
- [x] T3 (GREEN): Add `tau-unified intakes` and `tau-unified intake <id>`.
- [x] T4 (DOCS): Update the autonomous coding job guide.
- [x] T5 (VERIFY): Run focused shell verification, syntax checks, and diff
  hygiene checks.

## Evidence

- RED: `bash scripts/run/test-tau-unified.sh status_contract` failed before
  implementation because
  `control_plane.autonomous_coding.background_job_id=background-status-job`
  was not emitted.
- GREEN: `bash scripts/run/test-tau-unified.sh status_contract` passed.

## Test Tiers

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Shell projection over persisted JSON; no Rust unit changed |
| Property | N/A |  | No parser/invariant algorithm changed |
| Contract/DbC | N/A |  | No Rust public API contract changed |
| Snapshot | N/A |  | Existing shell assertions are line-contract checks |
| Functional | Done | `bash scripts/run/test-tau-unified.sh status_contract` |  |
| Conformance | Done | `status_contract` assertions cover C-01..C-05 |  |
| Integration | Done | `status_contract` exercises launcher command dispatch and fake CLI delegation |  |
| Fuzz | N/A |  | No untrusted parser surface added beyond JSON loading with skip/fail behavior |
| Mutation | N/A |  | Operator shell projection, not a critical algorithmic path |
| Regression | Done | `status_contract` preserves prior job/status behavior while adding fields |  |
| Performance | N/A |  | Small local JSON scans only |
