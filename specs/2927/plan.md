# Plan: Issue #2927 - Panic/unsafe audit with guardrail enforcement

1. Create deterministic audit tooling:
   - script to enumerate `panic!` and `unsafe` occurrences
   - path-level aggregation + heuristic test/production classification
   - machine-readable baseline/policy artifact
2. Review inventory for production-facing occurrences and remediate clearly unjustified cases in this slice.
3. Add guardrail command that compares current counts against baseline and fails on unapproved growth.
4. Run verification and regression suites.

## Risks / Mitigations
- Risk: false positives from `#[cfg(test)]` code inside `src/`.
  - Mitigation: classify report with explicit “heuristic/test-only/needs-review” buckets and review production bucket manually.
- Risk: guardrail too strict causing noisy failures.
  - Mitigation: baseline file with explicit thresholds and rationale.

## Interface / Contract Notes
- No external API protocol changes.
- Adds developer tooling under `scripts/dev` + policy artifact under `tasks/policies` + docs/research report.
