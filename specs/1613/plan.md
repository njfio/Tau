# Issue 1613 Plan

Status: Reviewed

## Approach

1. Add a split harness script for `tools.rs` maintainability budget and module
   boundary checks.
2. Create new modules under `crates/tau-tools/src/tools/`:
   - `memory_tools.rs`
   - `jobs_tools.rs`
3. Move memory-tool structs/impls into `memory_tools.rs`.
4. Move jobs-tool structs/impls into `jobs_tools.rs`.
5. Keep `tools.rs` as composition surface by importing/re-exporting moved tool
   types.
6. Run scoped verification (`tau-tools` tests, strict clippy, fmt, roadmap sync
   check, split harness).

## Affected Areas

- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/memory_tools.rs`
- `crates/tau-tools/src/tools/jobs_tools.rs`
- `scripts/dev/test-tools-rs-maintainability-split.sh`
- `specs/1613/*`

## Risks And Mitigations

- Risk: subtle behavior drift when moving impl blocks.
  - Mitigation: move code verbatim and preserve helper usage via `super::*`.
- Risk: test import breakage for moved tool types.
  - Mitigation: re-export moved types from root module.
- Risk: root line budget not reached after first pass.
  - Mitigation: target two largest domains (`memory` and `jobs`) first.

## ADR

No dependency/protocol architecture change; ADR not required.
