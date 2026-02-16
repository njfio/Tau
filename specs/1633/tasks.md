# Issue 1633 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add `scripts/dev/test-tools-rs-size-split.sh` and capture RED while `tools.rs` exceeds threshold.

T2: extract helper-domain block into `crates/tau-tools/src/tools/runtime_helpers.rs` and wire module imports.

T3: run GREEN split harness and targeted tau-tools parity tests.

T4: run scoped roadmap/fmt/clippy checks and prepare PR evidence.

## Tier Mapping

- Functional: file-size threshold + module wiring harness
- Regression: targeted tau-tools parity tests
- Integration: split harness + scoped quality checks
