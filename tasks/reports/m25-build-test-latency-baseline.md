# M25 Build/Test Latency Baseline

Generated: `2026-02-16T13:10:00Z`
Repository: `njfio/Tau`
Source mode: `live`

## Environment

| Field | Value |
|---|---|
| os | darwin |
| arch | arm64 |
| shell | zsh |
| python_version | 3.14.2 |
| rustc_version | rustc 1.90.0 (1159e78c4 2025-09-14) (Homebrew) |
| cargo_version | cargo 1.90.0 (Homebrew) |
| cpu_count | 10 |

## Command Timing Matrix

| Command ID | Runs | Avg ms | P50 ms | Min ms | Max ms | Failing runs |
|---|---:|---:|---:|---:|---:|---:|
| check-tools | 1 | 1006 | 1006 | 1006 | 1006 | 0 |
| test-runtime-no-run | 1 | 1073 | 1073 | 1073 | 1073 | 0 |
| test-trainer-regression | 1 | 933 | 933 | 933 | 933 | 0 |

## Hotspot Ranking

| Rank | Command ID | Avg ms | Command |
|---:|---|---:|---|
| 1 | test-runtime-no-run | 1073 | `cargo test -p tau-github-issues-runtime --target-dir target-fast --no-run` |
| 2 | check-tools | 1006 | `cargo check -p tau-tools --lib --target-dir target-fast` |
| 3 | test-trainer-regression | 933 | `cargo test -p tau-trainer --target-dir target-fast benchmark_artifact::tests::regression_summary_gate_report_manifest_ignores_non_json_files -- --nocapture` |

## Summary

- commands: 3
- runs: 3
- failing runs: 0
- slowest command id: test-runtime-no-run
