# Plan: Issue #3420 - Operator maturity wave implementation

## Approach
1. TUI shell upgrade:
   - Add new CLI mode/flags for operator shell rendering.
   - Build deterministic panel composition utilities in `tau-tui`.
   - Keep no-new-dependency path to avoid introducing terminal stack churn.
2. RL end-to-end harness:
   - Add harness module/binary in training stack (`tau-trainer`) that executes deterministic rollouts via `Trainer`.
   - Compute/emit rollout and RL summary values (including GAE/PPO summary) into artifact JSON.
   - Provide explicit failure-path validation.
3. Auth workflow conformance:
   - Add integration-level auth matrix validation in provider/coding-agent test surfaces.
   - Add deterministic gateway auth/session lifecycle tests covering success + unauthorized paths.
4. Verify:
   - Run scoped tests for touched crates and conformance selectors.
   - Update spec/tasks/milestone status + issue process logs.

## Affected Modules
- `crates/tau-tui/*`
- `crates/tau-trainer/*`
- `crates/tau-training-runner/*` (if harness glue is required)
- `crates/tau-coding-agent/src/tests/auth_provider/*`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs` (or focused auth test module)
- `specs/milestones/m295/index.md`
- `specs/3420/spec.md`
- `specs/3420/plan.md`
- `specs/3420/tasks.md`

## Risks / Mitigations
- Risk: broad multi-module changes increase regression surface.
  - Mitigation: keep each track self-contained with explicit conformance tests.
- Risk: RL e2e can become non-deterministic.
  - Mitigation: fixture-driven inputs, fixed seeds, deterministic artifact assertions.
- Risk: auth coverage drift across provider/gateway surfaces.
  - Mitigation: add a single conformance matrix test grouping with explicit mode/lifecycle cases.

## Interfaces / Contracts
- `tau-tui` CLI interface gains shell-mode options (backward-compatible defaults preserved).
- RL harness exposes deterministic artifact output contract (JSON shape tested).
- Auth conformance tests assert existing provider/gateway contracts; no auth protocol changes.

## ADR
- Not required (no new dependencies or protocol redesign planned in this issue).
