# Spec: Issue #3414 - Workspace gate stabilization (README examples marker regression)

Status: Implemented

## Problem Statement
Clean `cargo test` runs on `master` fail on two deterministic blockers:
1. `tau-coding-agent` integration suite fails because `README.md` no longer includes required example path markers validated by `examples_assets`.
2. `tau-tools` memory search tests intermittently fail under full-workspace load when shared test policy uses non-deterministic embedding path defaults.

## Scope
In scope:
- Restore required README example path references so `examples_assets` regression suite passes.
- Harden `tau-tools` memory test helper policy for deterministic offline embedding behavior under workspace-load conditions.
- Keep fix documentation-only with no runtime behavior changes.
- Capture deterministic RED/GREEN evidence for the failing test.

Out of scope:
- Unrelated runtime/test/lint regressions.
- Changes to example assets themselves.

## Acceptance Criteria
### AC-1 README includes required example markers
Given repository root README,  
when the examples marker regression test inspects expected paths,  
then all required markers (including `./examples/starter/package.json`) are present.

### AC-2 Regression test passes on clean run
Given a clean workspace checkout,  
when `cargo test -p tau-coding-agent --test examples_assets` runs,  
then it passes with no failing assertions.

### AC-3 `tau-tools` memory regression tests are deterministic under full workspace load
Given `tau-tools` memory regression tests,  
when they run in repeated and full-workspace contexts,  
then expected memory search/read counts are stable and non-zero where contract requires.

### AC-4 Workspace gate can proceed past the former blockers
Given post-fix workspace validation,  
when `cargo test` runs,  
then README marker and `tau-tools` memory regression blockers no longer fail.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | README content | inspect references | includes required `./examples/...` markers |
| C-02 | AC-2 | Regression | clean checkout | run `cargo test -p tau-coding-agent --test examples_assets` | suite passes |
| C-03 | AC-3 | Regression | `tau-tools` memory helper policy | run failing memory tests repeatedly | tests remain stable and pass |
| C-04 | AC-4 | Conformance | full `cargo test` run | execute workspace tests | no failure from README-marker or listed `tau-tools` memory regressions |

## Success Metrics / Observable Signals
- `examples_assets` suite is green on clean checkout.
- `tau-tools` memory regression tests are stable in repeated runs.
- Workspace `cargo test` is no longer blocked by README marker and `tau-tools` memory regressions.

## Implementation Evidence
### C-01/C-02 (README marker regression)
- RED:
  - `cargo test -p tau-coding-agent --test examples_assets`
  - failure: `README should reference ./examples/starter/package.json`
- GREEN:
  - `cargo test -p tau-coding-agent --test examples_assets`
  - result: `4 passed; 0 failed`

### C-03 (tau-tools memory determinism)
- `cargo test -p tau-tools --lib spec_2444_c05_legacy_records_without_relations_return_stable_defaults`
- `cargo test -p tau-tools --lib integration_memory_search_tool_honors_scope_filter`
- `cargo test -p tau-tools --lib integration_memory_tools_fixture_roundtrip_is_deterministic`
- repeated stability check:
  - five consecutive passes of `cargo test -p tau-integration-tests --test agent_tool_memory_roundtrip`

### C-04 (workspace conformance)
- `cargo test`
- `cargo fmt --all -- --check`
- `cargo clippy -- -D warnings`
