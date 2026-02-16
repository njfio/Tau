# Issue 1613 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add split harness and capture RED result before refactor.

T2: extract memory tools domain into `tools/memory_tools.rs`.

T3: extract jobs tools domain into `tools/jobs_tools.rs`.

T4: wire root-module imports/re-exports and keep external API stable.

T5: run scoped verification (`cargo test -p tau-tools`, strict clippy, fmt,
split harness, roadmap sync check).

## Tier Mapping

- Unit: existing `tau-tools` unit tests
- Functional: split harness checks for line budget + module boundaries
- Conformance: AC/C-case mapping validated by harness + tests
- Integration: existing tool behavior/runtime tests in `tau-tools`
- Regression: crate tests + strict clippy/fmt
