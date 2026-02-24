# True RL Roadmap Skeleton

This planning artifact defines long-horizon true-RL expansion and hardening
phases tracked under
[Epic #1657](https://github.com/njfio/Tau/issues/1657) and
[Milestone #24](https://github.com/njfio/Tau/milestone/24).

Current-state boundary:
- implemented baseline today:
  - deterministic RL end-to-end harness artifacts
  - live rollout/reward scoring and optimizer integration
  - APO significance gating in runtime optimization
- planned expansion:
  - broader production-scale policy-learning operations and control-plane
    hardening beyond the integrated baseline

## Stage 0: Architecture and Data Contracts

Objective: harden trajectory, advantage, and checkpoint data contracts.

Primary issues:
- [#1658 Story](https://github.com/njfio/Tau/issues/1658)
- [#1665](https://github.com/njfio/Tau/issues/1665)

Entry criteria:
- M22 naming alignment complete for current prompt-optimization docs.

Exit evidence:
- trajectory schema + adapter contracts documented and tested.

## Stage 1: Experience Collection Runtime

Objective: harden concurrent rollout collection with explicit liveness and
backpressure controls.

Primary issues:
- [#1659 Story](https://github.com/njfio/Tau/issues/1659)
- [#1666](https://github.com/njfio/Tau/issues/1666)
- [#1667](https://github.com/njfio/Tau/issues/1667)

Entry criteria:
- Stage 0 contracts implemented and validated.

Exit evidence:
- collector throughput/backpressure behavior documented and reproducible.

## Stage 2: PPO/GAE Core Optimization

Objective: expand and harden policy/value optimization and checkpoint resume
paths.

Primary issues:
- [#1660 Story](https://github.com/njfio/Tau/issues/1660)
- [#1668](https://github.com/njfio/Tau/issues/1668)
- [#1669](https://github.com/njfio/Tau/issues/1669)
- [#1670](https://github.com/njfio/Tau/issues/1670)

Entry criteria:
- Stage 1 collector stability and deterministic fixtures available.

Exit evidence:
- PPO/GAE conformance fixtures pass with deterministic math checks.

## Stage 3: Safety-Constrained Policy Learning

Objective: deepen safety constraints in reward shaping and promotion gates.

Primary issues:
- [#1661 Story](https://github.com/njfio/Tau/issues/1661)
- [#1671](https://github.com/njfio/Tau/issues/1671)
- [#1672](https://github.com/njfio/Tau/issues/1672)

Entry criteria:
- Stage 2 optimization loop complete.

Exit evidence:
- safety regression benchmark and checkpoint gating policy validated.

## Stage 4: Benchmarking and Statistical Proof

Objective: maintain policy-improvement proof with significance,
reproducibility, and live benchmark protocol at higher operational scale.

Primary issues:
- [#1662 Story](https://github.com/njfio/Tau/issues/1662)
- [#1673](https://github.com/njfio/Tau/issues/1673)
- [#1674](https://github.com/njfio/Tau/issues/1674)
- [#1675](https://github.com/njfio/Tau/issues/1675)
- [#1698](https://github.com/njfio/Tau/issues/1698)

Entry criteria:
- Stage 3 safety constraints integrated.

Exit evidence:
- baseline-vs-trained significance report and live-run benchmark artifacts.

## Stage 5: Operations and Control Plane Hardening

Objective: ship production operational controls (pause/resume/rollback/recovery).

Primary issues:
- [#1663 Story](https://github.com/njfio/Tau/issues/1663)
- [#1676](https://github.com/njfio/Tau/issues/1676)
- [#1677](https://github.com/njfio/Tau/issues/1677)
- [#1710](https://github.com/njfio/Tau/issues/1710)
- [#1702 Gate](https://github.com/njfio/Tau/issues/1702)

Entry criteria:
- Stage 4 benchmark proof accepted.

Exit evidence:
- operator controls verified with failure drills and rollback coverage.

## Validation Rhythm

For roadmap drift control:

- maintain stage-to-issue mapping in this document
- run roadmap status sync artifacts:
  - `scripts/dev/roadmap-status-sync.sh`
- publish periodic tracker updates on:
  - [#1657 Epic](https://github.com/njfio/Tau/issues/1657)
  - [#1702 Gate](https://github.com/njfio/Tau/issues/1702)
