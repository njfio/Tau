# Tasks: Issue #3040 - Training proxy coverage hardening

## Ordered Tasks
1. [x] T1 (RED): add failing conformance test for upstream `x-request-id` response header pass-through.
2. [x] T2 (GREEN): implement `x-request-id` response header propagation in proxy runtime.
3. [x] T3 (GREEN): add/validate failure-path and health contract tests.
4. [x] T4 (REGRESSION): rerun targeted `tau-training-proxy` tests.
5. [x] T5 (VERIFY): run `cargo fmt --check` and `cargo check -q`.

## Tier Mapping
- Unit: header parsing/filter helpers
- Property: N/A (no randomized invariants)
- Contract/DbC: N/A (no contracts crate changes)
- Snapshot: N/A (no snapshot tests)
- Functional: proxy forwarding and failure-path behavior tests
- Conformance: C-01..C-04
- Integration: in-crate async endpoint flow tests
- Fuzz: N/A (no new untrusted parser surface)
- Mutation: N/A (non-critical targeted coverage slice)
- Regression: targeted proxy test reruns + baseline checks
- Performance: N/A (no perf-sensitive path changes)
