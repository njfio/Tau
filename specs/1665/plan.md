# Issue 1665 Plan

Status: Reviewed

## Approach

1. Add a new adapter in `tau-algorithm` that groups spans by
   `(rollout_id, attempt_id)` and emits `EpisodeTrajectory` outputs.
2. Build `TrajectoryStep` values from span attributes with deterministic
   extraction order and fallback payloads for missing fields.
3. Compute `done` on the terminal step per trajectory and propagate optional
   `logprob`/`value_estimate` metadata when available.
4. Validate each generated trajectory with `EpisodeTrajectory::validate` and
   return deterministic errors on failure/empty input.
5. Add tests for happy path, partial telemetry fallback, and deterministic
   failure behavior.

## Affected Areas

- `crates/tau-algorithm/src/adapters.rs`
- `crates/tau-algorithm/src/lib.rs`
- `specs/1665/{spec,plan,tasks}.md`

## Risks And Mitigations

- Risk: overfitting to one span schema variant.
  - Mitigation: attribute extraction supports multiple keys and structured
    fallback metadata.
- Risk: non-deterministic ordering across grouped spans.
  - Mitigation: stable sort by `(rollout_id, attempt_id, sequence_id)`.

## ADR

No architecture/dependency/protocol decision change. ADR not required.
