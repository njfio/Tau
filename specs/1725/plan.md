# Issue 1725 Plan

Status: Reviewed

## Approach

1. Add a runner integration load test that enqueues a burst of rollouts and
   starts multiple worker runners.
2. Measure elapsed runtime and compute throughput from completed rollout count.
3. Add a developer harness script for repeatable local execution and metric
   capture.

## Affected Areas

- `crates/tau-training-runner/src/lib.rs`
- `scripts/dev/`
- `specs/1725/{spec,plan,tasks}.md`

## Risks And Mitigations

- Risk: flaky timing assertions under variable CI load.
  - Mitigation: assert positivity and completion invariants, avoid brittle hard
    performance thresholds.
- Risk: harness runtime length increases iteration cost.
  - Mitigation: keep rollout/worker counts bounded for quick deterministic runs.

## ADR

No architecture/dependency/protocol change. ADR not required.
