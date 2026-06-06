# Plan: Issue #3768 - Runtime reality gate for agentic-runtime claims

## Approach

Add a fast shell gate under `scripts/dev/` that turns the current reality check
into a repeatable artifact. The gate should be useful for humans and CI:

1. Run fast deterministic proof checks by default.
2. Emit structured JSON and markdown.
3. Keep heavyweight/live checks opt-in and visible.
4. Fail closed when a surface that is intentionally not claimable is marked as
   complete.
5. Keep hotspot line counts visible without making this slice a refactor.

The implementation is intentionally a claim-boundary gate, not a replacement for
production RL, live-provider, or headed-browser validation.

## Affected Modules

- `scripts/dev/runtime-reality-gate.sh`
- `scripts/dev/test-runtime-reality-gate.sh`
- `README.md`
- `specs/3768-runtime-reality-gate/*`

## Risks and Mitigations

- Risk: The gate becomes another aspirational report.
  Mitigation: run fast proof commands by default and fail if any fail.
- Risk: Heavy checks make default validation unusably slow.
  Mitigation: record heavyweight/live commands as opt-in by default.
- Risk: Hotspot over-threshold status causes false failure.
  Mitigation: classify hotspots as partial evidence, not a failing condition,
  until a separate refactor slice changes budgets.
- Risk: Shell JSON becomes invalid.
  Mitigation: validate outputs with `jq` in the regression test.

## Verification

- RED/GREEN: `scripts/dev/test-runtime-reality-gate.sh`
- Default gate: `scripts/dev/runtime-reality-gate.sh --output-json /tmp/tau-runtime-reality.json --output-md /tmp/tau-runtime-reality.md`
- Existing fast proofs:
  - `scripts/dev/prove-tau-product.sh --check`
  - `scripts/run/test-tau-unified.sh`
  - `scripts/dev/test-ops-chat-canvas-proof.sh`
  - `scripts/dev/roadmap-status-sync.sh --check --quiet`
- Static: `bash -n scripts/dev/runtime-reality-gate.sh scripts/dev/test-runtime-reality-gate.sh`
- Static: `git diff --check`
