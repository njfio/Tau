# Fast-Lane Dev Loop (M25.4.2a)

This guide documents crate-scoped fast-lane wrappers used for high-frequency
local feedback loops.

Wrapper script:

- `scripts/dev/fast-lane-dev-loop.sh`

Test suites:

- `scripts/dev/test-fast-lane-dev-loop.sh`
- `python3 .github/scripts/test_fast_lane_dev_loop_contract.py`

Benchmark report artifacts:

- `tasks/reports/m25-fast-lane-loop-comparison.json`
- `tasks/reports/m25-fast-lane-loop-comparison.md`

## Wrapper Catalog

List wrappers and commands:

```bash
scripts/dev/fast-lane-dev-loop.sh list
```

Run a single wrapper command:

```bash
scripts/dev/fast-lane-dev-loop.sh run tools-check
```

Preview without execution:

```bash
scripts/dev/fast-lane-dev-loop.sh run trainer-smoke --dry-run
```

## Benchmark Comparison

Compare fast-lane median loop timing against baseline from
`tasks/reports/m25-build-test-latency-baseline.json`:

```bash
scripts/dev/fast-lane-dev-loop.sh benchmark \
  --baseline-json tasks/reports/m25-build-test-latency-baseline.json \
  --output-json tasks/reports/m25-fast-lane-loop-comparison.json \
  --output-md tasks/reports/m25-fast-lane-loop-comparison.md
```

Deterministic fixture mode for tests:

```bash
scripts/dev/fast-lane-dev-loop.sh benchmark \
  --fixture-json path/to/fixture.json \
  --baseline-json path/to/baseline.json \
  --output-json /tmp/fast-lane.json \
  --output-md /tmp/fast-lane.md
```
