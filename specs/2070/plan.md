# Plan #2070

Status: Reviewed
Spec: specs/2070/spec.md

## Approach

1. Add workflow contract tests that fail until cache shared-key snippets and
   parallel helper invocation are present.
2. Update `ci.yml` rust-cache steps with lane-scoped `shared-key` and
   `cache-on-failure` settings.
3. Change helper-suite step to a portable parallel helper runner while
   preserving the same discovery scope/pattern.
4. Add a local timing comparison script + report artifact for serial vs parallel
   helper loops.
5. Add docs and rerun existing CI-workflow contract suites.

## Affected Modules

- `.github/workflows/ci.yml`
- `scripts/dev/ci-cache-parallel-tuning-report.sh`
- `scripts/dev/test-ci-cache-parallel-tuning-report.sh`
- `.github/scripts/test_ci_cache_parallel_contract.py`
- `docs/guides/ci-cache-parallel-tuning.md`
- `tasks/reports/m25-ci-cache-parallel-tuning.json`
- `tasks/reports/m25-ci-cache-parallel-tuning.md`
- `specs/2070/spec.md`
- `specs/2070/plan.md`
- `specs/2070/tasks.md`

## Risks and Mitigations

- Risk: cache key fragmentation lowers hit rate.
  - Mitigation: keep shared-key scoped by lane + OS/target + lockfile hash.
- Risk: parallel helper execution introduces flakiness.
  - Mitigation: retain same test discovery pattern and verify with contract
    suites.
- Risk: workflow changes accidentally remove required guard steps.
  - Mitigation: rerun existing CI contract tests after modifications.

## Interfaces and Contracts

- Timing report script:
  `scripts/dev/ci-cache-parallel-tuning-report.sh`
- Validation suites:
  `scripts/dev/test-ci-cache-parallel-tuning-report.sh`
  `python3 .github/scripts/test_ci_cache_parallel_contract.py`
  existing CI workflow contract tests under `.github/scripts/test_ci_*.py`.

## ADR References

- Not required.
