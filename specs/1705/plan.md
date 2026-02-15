# Issue 1705 Plan

Status: Reviewed

## Approach

1. Add issue specs/tasks artifacts for M22 terminology scan gate.
2. Run `scripts/dev/rl-terminology-scan.sh` to capture baseline stale findings.
3. Remediate stale items by:
   - updating docs wording where stale terminology is unnecessary
   - refining allowlist policy for explicitly approved guide contexts
4. Re-run scanner and persist updated reports in `tasks/reports/`.
5. Run scanner/allowlist contract tests to confirm behavior stability.

## Affected Areas

- `tasks/policies/rl-terms-allowlist.json`
- `docs/guides/rl-terminology-allowlist.md`
- `docs/guides/training-ops.md`
- `tasks/todo.md`
- `tasks/reports/m22-rl-terminology-scan.{json,md}`

## Risks And Mitigations

- Risk: allowlist broadened too far
  - Mitigation: constrain approved contexts with explicit path + context patterns.
- Risk: scanner contract regressions
  - Mitigation: keep `test-rl-terminology-scan.sh` and allowlist contract tests green.

## ADR

No dependency/protocol changes. ADR not required.
