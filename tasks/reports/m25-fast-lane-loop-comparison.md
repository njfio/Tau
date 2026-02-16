# M25 Fast-Lane Loop Comparison

Generated: `2026-02-16T14:20:00Z`
Repository: `njfio/Tau`
Baseline report: `tasks/reports/m25-build-test-latency-baseline.json`

## Summary

| Status | Baseline median ms | Fast-lane median ms | Improvement ms | Improvement % |
|---|---:|---:|---:|---:|
| improved | 1006 | 995 | 11 | 1.09 |

## Wrapper Measurements

| Wrapper | Duration ms | Exit code | Use case | Command |
|---|---:|---:|---|---|
| tools-check | 1003 | 0 | Compile tau-tools quickly for tool-layer edits | `cargo check -p tau-tools --lib --target-dir target-fast` |
| trainer-check | 923 | 0 | Compile tau-trainer library surfaces for trainer/runtime edits | `cargo check -p tau-trainer --lib --target-dir target-fast` |
| trainer-smoke | 995 | 0 | Run targeted trainer regression smoke check | `cargo test -p tau-trainer --target-dir target-fast benchmark_artifact::tests::regression_summary_gate_report_manifest_ignores_non_json_files -- --nocapture` |
