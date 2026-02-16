# Issue 1634 Plan

Status: Reviewed

## Approach

1. Add combined runner script:
   - `scripts/dev/m21-tool-split-validation.sh`
   - run parity script and performance smoke script
   - synthesize overall decision and report artifacts under `tasks/reports/`
2. Add tests-first harness:
   - `scripts/dev/test-m21-tool-split-validation.sh`
   - pass/warn/fail fixture scenarios to validate deterministic semantics
3. Run RED before combined runner exists.
4. Implement combined runner + report generation.
5. Run GREEN harness and scoped checks:
   - `scripts/dev/roadmap-status-sync.sh --check --quiet`
   - `cargo fmt --check`
   - `cargo clippy -p tau-tools -- -D warnings`

## Affected Areas

- `scripts/dev/m21-tool-split-validation.sh`
- `scripts/dev/test-m21-tool-split-validation.sh`
- `tasks/reports/m21-tool-split-validation.json`
- `tasks/reports/m21-tool-split-validation.md`
- `specs/1634/spec.md`
- `specs/1634/plan.md`
- `specs/1634/tasks.md`

## Risks And Mitigations

- Risk: combined runner masks underlying script failures.
  - Mitigation: fail-fast when sub-script exits non-zero and preserve explicit reason codes.
- Risk: non-deterministic timings create noisy evidence.
  - Mitigation: support fixture mode in harness tests and keep performance thresholds explicit.

## ADR

No dependency/protocol/architecture decision changes; ADR not required.
