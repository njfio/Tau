# Integration Gap Closure Plan

This plan tracks the four capability gaps called out from README review:
- true RL end-to-end delivery,
- dashboard maturity expansion,
- comprehensive auth workflow verification,
- TUI UX improvements.

## 1) True RL Delivery

Current state:
- Prompt optimization is the canonical training path today.
- True-RL primitives and benchmark/safety proof scripts exist.

Closure objective:
- Promote true RL from staged primitives to an end-to-end, operator-run workflow with acceptance gates.

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
- Dashboard ops routes and API-backed diagnostics exist.
- Not all desired product UX surfaces are complete live-mutation experiences.

Closure objective:
- Expand dashboard from route/ops coverage toward full operator workflows.

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
- Provider auth modes and gateway auth/session flows are implemented.
- Cross-mode verification is not yet represented as a single consolidated acceptance matrix in README-level guidance.

Closure objective:
- Verify and document all supported auth modes/flows using deterministic smoke steps and provider matrix references.

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
- `tau-tui` is a lightweight demo runner and smoke surface.
- It is not a full operator dashboard replacement.

Closure objective:
- Improve usability and operator signal density in terminal UX while preserving deterministic smoke behavior.

Execution anchors:
- [`crates/tau-tui`](../../crates/tau-tui)
- [`docs/guides/demo-index.md`](../guides/demo-index.md)

Initial verification command:

```bash
cargo run -p tau-tui -- --frames 3 --sleep-ms 0 --width 72 --no-color
```

