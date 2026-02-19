# Tasks: Issue #2609 - Under-tested crates coverage wave

## Ordered Tasks
1. T1 (RED): add diagnostics tests for doctor arg parsing/policy usage failure behavior and percentile/render edge outputs.
2. T2 (RED): add training-proxy tests for invalid sequence header rejection and upstream non-2xx propagation + attribution log fields.
3. T3 (RED): add provider client helper tests for Azure endpoint detection and strict fallback gate decisions.
4. T4 (GREEN): apply minimal code adjustments only if RED tests expose defects; otherwise keep changes test-only.
5. T5 (VERIFY): run fmt, scoped clippy, and scoped tests for `tau-diagnostics`, `tau-training-proxy`, and `tau-provider`.
6. T6 (CLOSE): update issue log/status and mark `specs/2609/spec.md` as `Implemented`.

## Tier Mapping
- Unit: C-01, C-05
- Property: N/A (no randomized invariant surface introduced)
- Contract/DbC: N/A (no contracts annotations introduced)
- Snapshot: N/A (no snapshot fixtures required)
- Functional: C-02
- Conformance: C-01..C-05
- Integration: C-04
- Fuzz: N/A (no new parser/fuzzer target)
- Mutation: N/A for this focused coverage wave (no critical-path mutation campaign targeted in-slice)
- Regression: C-02, C-03
- Performance: N/A (no runtime algorithm change)
