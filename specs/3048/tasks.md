# Tasks: Issue #3048 - KAMN SDK coverage hardening

## Ordered Tasks
1. [x] T1 (RED): add failing conformance test for malformed-input propagation with SDK error context.
2. [x] T2 (RED/GREEN): add conformance test for deterministic normalized-equivalent identity outputs.
3. [x] T3 (RED/GREEN): add conformance test for nested report persistence + trailing newline contract.
4. [x] T4 (GREEN): implement minimal SDK runtime context hardening required by RED.
5. [x] T5 (REGRESSION): rerun targeted and full `kamn-sdk` tests.
6. [x] T6 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-sdk -- -D warnings`, and `cargo check -q`.

## Tier Mapping
- Unit: initialization and report persistence behavior tests in crate test module
- Property: N/A (no randomized invariant requirement in this slice)
- Contract/DbC: N/A (no contracts crate annotations)
- Snapshot: N/A (no snapshot output surface)
- Functional: browser DID init/report behavior checks
- Conformance: C-01..C-04
- Integration: end-to-end init + persisted report roundtrip behavior
- Fuzz: N/A (no new parser surface)
- Mutation: N/A (non-critical targeted hardening slice)
- Regression: targeted + full crate test reruns plus lint/check gates
- Performance: N/A (no hotspot/perf-path changes)
