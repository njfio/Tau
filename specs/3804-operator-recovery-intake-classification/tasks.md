# Tasks 3804: Operator Recovery and Issue Intake Classification

- [x] T1 - Add failing/locking tests for stale lease status, mark-blocked, and
  intake classification.
- [x] T2 - Add operator fields to autonomous coding job status snapshots.
- [x] T3 - Add `tau-autonomous-coding-job mark-blocked`.
- [x] T4 - Add issue intake classification and targeted required-authority
  output.
- [x] T5 - Surface operator fields in `tau-unified status`.
- [x] T6 - Run focused runtime tests, `tau-unified` contract test, fmt, and
  clippy.

## Test Tier Matrix

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | Focused `tau-runtime` status/intake tests |  |
| Property | N/A |  | No randomized invariant in this slice |
| Contract/DbC | N/A |  | No contract macro surface |
| Snapshot | N/A |  | Status JSON asserted directly |
| Functional | ✅ | `scripts/run/test-tau-unified.sh` |  |
| Conformance | ✅ | C-01..C-05 via Rust and shell tests |  |
| Integration | ✅ | `tau-unified status` reads persisted status JSON |  |
| Fuzz | N/A |  | No untrusted parser added |
| Mutation | N/A |  | Not required for this narrow operator/status slice |
| Regression | ✅ | Existing intake/job tests extended |  |
| Performance | N/A |  | No hot path |
