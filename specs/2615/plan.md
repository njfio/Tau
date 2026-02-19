# Plan: Issue #2615 - Integrate RL loop with live agent decision-making path

## Approach
1. Add RED tests for live event -> rollout collection, scheduled optimizer execution, and failure-gate hold transitions.
2. Implement a `live_rl_runtime` module in `tau-coding-agent` with:
   - env-driven opt-in configuration,
   - event subscription wiring,
   - rollout/span persistence,
   - update scheduling with PPO/GAE math,
   - guarded failure gate controls.
3. Wire the module into local runtime startup so hooks register only when enabled.
4. Run scoped verification gates and map AC/C evidence.

## Affected Modules
- `crates/tau-coding-agent/src/main.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/live_rl_runtime.rs` (new)
- `specs/2615/spec.md`
- `specs/2615/plan.md`
- `specs/2615/tasks.md`

## Risks / Mitigations
- Risk: RL hook overhead impacts runtime prompt latency.
  - Mitigation: keep disabled by default; lightweight event handling; fail-gated hold mode.
- Risk: invalid/empty trajectories create unstable update math.
  - Mitigation: guarded skip paths and explicit status reporting when samples are absent.
- Risk: rollout persistence failures cascade into runtime instability.
  - Mitigation: failure streak guard transitions gate to hold and suppresses further work.

## Interfaces / Contracts
- `LiveRlRuntimeConfig`
- `LiveRlRuntimeGate` (`pass` / `hold`)
- `LiveRlRuntimeSnapshot`
- Event handler bridge attached via `agent.subscribe_async(...)`
- Scheduled optimizer report payload from PPO/GAE update pipeline

## ADR
- Not required (additive module wiring, no new dependency or protocol boundary).
