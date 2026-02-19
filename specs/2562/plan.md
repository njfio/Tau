# Plan #2562

1. Re-run #2561 conformance/regression tests on touched crates and capture pass results.
2. Run mutation-in-diff for impacted crates; if mutants escape, add/strengthen tests and rerun until zero missed.
3. Run sanitized live validation smoke (`provider-live-smoke`) and record summary.
4. Update comparison checklist and issue process-log comments with command outcomes.
5. Prepare PR evidence matrix linking AC -> tests/gates.

## Risks
- Mutation baseline can fail due unrelated flaky tests in large crates.
- Stale diff file can invalidate mutation-in-diff runs.

## Mitigations
- Scope mutation to impacted crates (`tau-onboarding`, `tau-agent-core`) and regenerate diff before reruns.
- Use deterministic sanitized env for live validation to avoid key leakage/cost drift.
