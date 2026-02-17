# Tasks: Issue #2410 - Fly.io deployment baseline for Tau gateway

## Ordered Tasks
1. T1 (RED): add C-01/C-02/C-03 tests for Fly manifest + docs contract and capture failing output.
2. T2 (GREEN): implement Fly manifest contract loader/validator in `tau-deployment`.
3. T3 (GREEN): add repository-root `fly.toml` satisfying contract.
4. T4 (GREEN): update deployment runbook with Fly launch/deploy/verify commands.
5. T5 (VERIFY): run `cargo fmt --check`, `cargo clippy -p tau-deployment -- -D warnings`, and targeted tests.
6. T6 (CLOSE): PR with AC mapping, RED/GREEN evidence, and completed tier matrix.

## Tier Mapping
- Unit: C-02 validator failure-path checks
- Functional: C-01 repository manifest load contract
- Integration: C-02 service + health-check contract checks
- Regression: C-03 docs/runbook drift guard
- Property/Contract/Snapshot/Fuzz/Performance/Mutation: N/A for this scoped deployment packaging slice
