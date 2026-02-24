# Integration Gap Closure Plan

This plan tracks the four capability areas called out from prior README review.
Baseline delivery for each area is now integrated; this document tracks
expansion and hardening work beyond the delivered baseline:
- true RL end-to-end delivery,
- dashboard maturity expansion,
- comprehensive auth workflow verification,
- TUI UX improvements.

## 1) True RL Delivery

Current state:
- Deterministic true-RL end-to-end harness is integrated.
- Live rollout/reward/optimizer and APO significance gating are integrated.
- Operator verification gates exist for RL maturity/readiness waves.

Closure objective:
- Extend from integrated baseline to deeper production policy-operations
  coverage (long-horizon evaluation, promotion controls, and recovery drills).

Execution anchors:
- [`docs/planning/true-rl-roadmap-skeleton.md`](true-rl-roadmap-skeleton.md)
- [`docs/guides/training-ops.md`](../guides/training-ops.md)
- [`scripts/demo/m24-rl-live-benchmark-proof.sh`](../../scripts/demo/m24-rl-live-benchmark-proof.sh)

Initial verification command:

```bash
./scripts/demo/m24-rl-live-benchmark-proof.sh
```

## 2) Dashboard Maturity

Current state:
- Dashboard status/widgets/alerts/stream/timeline conformance is integrated.
- Operator diagnostics surfaces are wired into runtime status and alert feeds.

Closure objective:
- Expand dashboard from integrated diagnostics into richer end-to-end operator
  workflows and higher-fidelity live mutation UX.

Execution anchors:
- [`docs/guides/dashboard-ops.md`](../guides/dashboard-ops.md)
- [`docs/guides/operator-deployment-guide.md`](../guides/operator-deployment-guide.md)
- [`scripts/demo/dashboard.sh`](../../scripts/demo/dashboard.sh)

Initial verification command:

```bash
./scripts/demo/dashboard.sh
```

## 3) Auth Workflow Verification

Current state:
- Provider auth-mode matrix and gateway auth/session lifecycle conformance are integrated.
- Deterministic smoke and conformance suites verify core auth workflows.

Closure objective:
- Expand live-environment permutation coverage and keep README/operator guidance
  synchronized with provider/auth workflow evidence.

Execution anchors:
- [`docs/provider-auth/provider-auth-capability-matrix.md`](../provider-auth/provider-auth-capability-matrix.md)
- [`docs/guides/gateway-auth-session-smoke.md`](../guides/gateway-auth-session-smoke.md)
- [`scripts/demo/gateway-auth-session.sh`](../../scripts/demo/gateway-auth-session.sh)

Initial verification command:

```bash
./scripts/demo/gateway-auth-session.sh
```

## 4) TUI Improvements

Current state:
- `tau-tui` operator-shell mode is integrated with structured panels
  (status/auth/training/alerts/actions) and conformance tests.
- TUI is a terminal operator surface that complements the web dashboard.

Closure objective:
- Continue improving operator-shell usability, interaction depth, and signal
  density while preserving deterministic validation behavior.

Execution anchors:
- [`crates/tau-tui`](../../crates/tau-tui)
- [`docs/guides/demo-index.md`](../guides/demo-index.md)

Initial verification command:

```bash
cargo run -p tau-tui -- shell --width 88 --profile local-dev --no-color
```
