# Plan: Issue #2996 - panic/unsafe audit script and production-path guardrails

## Approach
1. T1 RED: add fixture-based conformance test script for expected classification counts before implementation.
2. Implement `scripts/dev/audit-panic-unsafe.sh` with deterministic sorting and test-path classification.
3. Run fixture conformance + repo audit script and capture outputs.
4. Run production-target guardrail checks for `clippy::panic` and `unsafe_code`.
5. Update milestone/issue status artifacts after verification.

## Affected Modules
- `scripts/dev/audit-panic-unsafe.sh` (new)
- `scripts/dev/test-audit-panic-unsafe.sh` (new)
- `scripts/dev/fixtures/panic-unsafe-audit/` (new fixture files)
- `specs/milestones/m177/index.md`
- `specs/2996/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: path-based classification may misclassify some `#[cfg(test)]` modules.
  - Mitigation: keep classification explicit as path class (test-path vs non-test-path), not semantic runtime classification.
- Risk: non-deterministic output ordering causes flaky checks.
  - Mitigation: sort all match output before aggregation.
- Risk: script portability across shells.
  - Mitigation: use POSIX-compatible shell + `rg`/`awk`/`sort` only.

## Interfaces / Contracts
- Script entrypoint: `scripts/dev/audit-panic-unsafe.sh [scan_root]`
- Default scan root: `crates`
- Output contract: summary counts + grouped file/line listings for panic and unsafe matches.
