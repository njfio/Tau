# Tasks: Operator Recovery, Hands-Off Intake, and Draft PR Loop

- [x] T1: Write spec, plan, and task artifacts before implementation.
- [x] T2: Add operator job list/inspect/recover/replay/block commands to
  `tau-unified`.
- [x] T3: Add deterministic verifier-plan and missing-input metadata to issue
  intake outcomes.
- [x] T4: Harden draft PR publication evidence for GitHub-auth success and
  missing-auth/manual fallback.
- [x] T5: Update docs and README boundaries.
- [x] T6: Add focused tests for runtime and `tau-unified`.
- [x] T7: Run focused tests, clippy, format, shell syntax, and diff checks.

## Test Tiers

| Tier | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-runtime spec_3806 -- --test-threads=1` | Intake planner and PR publication evidence. |
| Property | N/A | Not parser/algorithm-heavy | Deterministic heuristics only. |
| Contract/DbC | ✅ | `scripts/run/test-tau-unified.sh` | `tau-unified` operator command contract. |
| Snapshot | N/A | Dynamic state paths | Stable marker assertions instead. |
| Functional | ✅ | `scripts/run/test-tau-unified.sh`; `cargo test -p tau-runtime spec_3796_c04 -- --test-threads=1` | Operator recovery loop and intake regression. |
| Conformance | ✅ | `cargo test -p tau-runtime spec_3806 -- --test-threads=1`; `scripts/run/test-tau-unified.sh` | Maps to C-01..C-06. |
| Integration | ✅ | Runtime tests with fake `gh`; launcher tests with fake autonomous-job CLI | Draft PR and operator action paths. |
| Fuzz | N/A | No untrusted parser change | JSON parsing remains serde/Python stdlib. |
| Mutation | N/A | Product shell/runtime slice | Not a critical algorithm change. |
| Regression | ✅ | `cargo test -p tau-runtime spec_3801 -- --test-threads=1`; missing-auth PR fallback test | Protects current caveats. |
| Performance | N/A | Not performance-sensitive | No hot path change. |
