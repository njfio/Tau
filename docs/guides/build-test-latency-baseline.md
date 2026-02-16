# Build/Test Latency Baseline (M25.4.1a)

This guide defines how to generate reproducible build/test timing matrix artifacts
for M25 velocity work.

Generator script:

- `scripts/dev/build-test-latency-baseline.sh`

Schema contract:

- `tasks/schemas/m25-build-test-latency-baseline.schema.json`

Test suites:

- `scripts/dev/test-build-test-latency-baseline.sh`
- `python3 .github/scripts/test_build_test_latency_baseline_contract.py`

## Fixture Mode (Deterministic)

Use fixture mode for contract validation and reproducible CI checks.

```bash
scripts/dev/build-test-latency-baseline.sh \
  --fixture-json path/to/fixture.json \
  --generated-at 2026-02-16T12:00:00Z \
  --output-json tasks/reports/m25-build-test-latency-baseline.json \
  --output-md tasks/reports/m25-build-test-latency-baseline.md
```

## Live Mode (Real Command Timings)

Live mode executes command rows and captures elapsed milliseconds + exit status.

```bash
scripts/dev/build-test-latency-baseline.sh \
  --iterations 1 \
  --command "check-tools::cargo check -p tau-tools --lib --target-dir target-fast" \
  --command "test-runtime-no-run::cargo test -p tau-github-issues-runtime --target-dir target-fast --no-run" \
  --command "test-trainer-regression::cargo test -p tau-trainer --target-dir target-fast benchmark_artifact::tests::regression_summary_gate_report_manifest_ignores_non_json_files -- --nocapture"
```

If `--command` entries are omitted in live mode, the script runs the same default
three command rows shown above.

## Artifact Expectations

The JSON artifact includes:

- `source_mode` (`fixture` or `live`)
- environment metadata (`os`, `arch`, `shell`, `rustc_version`, `cargo_version`)
- per-command timing stats (`avg_ms`, `p50_ms`, `min_ms`, `max_ms`)
- hotspot ranking sorted by descending `avg_ms`

The Markdown artifact includes a command timing matrix and hotspot table suitable
for issue comments and PR evidence.
