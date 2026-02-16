# CI Cache + Parallel Tuning (M25.4.3a)

This guide captures the cache-key and helper-suite scheduling tuning used by
issue `#2070`.

Primary script:

- `scripts/dev/ci-cache-parallel-tuning-report.sh`
- `.github/scripts/ci_helper_parallel_runner.py`

Validation suites:

- `scripts/dev/test-ci-cache-parallel-tuning-report.sh`
- `python3 .github/scripts/test_ci_cache_parallel_contract.py`

Generated report artifacts:

- `tasks/reports/m25-ci-cache-parallel-tuning.json`
- `tasks/reports/m25-ci-cache-parallel-tuning.md`

## Run report generation

```bash
scripts/dev/ci-cache-parallel-tuning-report.sh \
  --output-json tasks/reports/m25-ci-cache-parallel-tuning.json \
  --output-md tasks/reports/m25-ci-cache-parallel-tuning.md
```

Fixture mode for deterministic validation:

```bash
scripts/dev/ci-cache-parallel-tuning-report.sh \
  --fixture-json path/to/fixture.json \
  --output-json /tmp/m25-ci-cache-parallel-tuning.json \
  --output-md /tmp/m25-ci-cache-parallel-tuning.md
```

## What it validates

- CI helper test command median duration in serial mode.
- CI helper test command median duration using
  `ci_helper_parallel_runner.py --workers 4`.
- Improvement delta (`serial_median_ms - parallel_median_ms`) and improvement
  percentage.
- Deterministic status classification: `improved`, `unchanged`, or `regressed`.
