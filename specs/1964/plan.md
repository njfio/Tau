# Issue 1964 Plan

Status: Reviewed

## Approach

1. Add `TrajectoryCollectionBatch` and `TrajectoryCollectionSkip` types in
   `tau-algorithm`.
2. Add `collect_trajectory_batch(store, rollout_ids, window_policy)` helper:
   - normalize rollout ids (sort/dedup)
   - verify rollout ids exist via `query_rollouts`
   - query spans per rollout and adapt via `SpansToTrajectories`
   - accumulate trajectories, span totals, and skip reasons
3. Keep error surface deterministic for unknown rollout ids.
4. Add tests C-01..C-04 using in-memory store + synthetic spans.

## Affected Areas

- `crates/tau-algorithm/src/lib.rs`
- `crates/tau-algorithm/src/collector.rs` (new)
- `specs/1964/spec.md`
- `specs/1964/plan.md`
- `specs/1964/tasks.md`

## Risks And Mitigations

- Risk: non-deterministic rollout id ordering.
  - Mitigation: sort/dedup ids before processing.
- Risk: ambiguous skip conditions.
  - Mitigation: deterministic reason strings (`no spans`, `no trajectories`).

## ADR

No dependency/protocol changes; ADR not required.
