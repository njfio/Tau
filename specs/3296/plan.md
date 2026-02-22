# Plan: Issue #3296

## Approach
1. Extend live RL runtime config with APO controls and significance thresholds.
2. Inject runtime APO dependencies (LLM client/model + seed prompt) through startup wiring.
3. Build span-derived APO samples from succeeded live rollouts.
4. Run APO with a reward-aware evaluator and fixed low-cost config.
5. Compare baseline vs candidate quality vectors with benchmark significance helpers.
6. Persist adopted prompt resources only when gate passes.
7. Add conformance tests for adopted/non-adopted behavior and runtime report fields.

## Affected Modules
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`

## Risks and Mitigations
- Risk: additional LLM latency during updates.
  Mitigation: bounded APO config (single-round beam) and sample caps.
- Risk: false-positive prompt adoption.
  Mitigation: significance gate + deterministic fallback scoring + explicit reason codes.
- Risk: runtime regressions in existing PPO flow.
  Mitigation: preserve current update path and add regression coverage on existing optimizer behavior.

## Interfaces / Contracts
- Internal runtime-only config/env extension for APO controls.
- Internal `LiveRlOptimizerReport` extension with APO diagnostics.
- Training resources updates include adopted prompt metadata keys.

## ADR
Not required.
