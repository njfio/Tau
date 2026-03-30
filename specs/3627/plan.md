# Plan: Issue #3627 - Align plan_executor documentation with its actual reporting surface

Status: Implemented
Milestone: M329
Parent: #3623

## Compatibility Strategy
```yaml
implementation_strategy:
  task: "3627"
  change_surface:
    - symbol: "plan_executor module/item documentation"
      location: "crates/tau-orchestrator/src/plan_executor.rs"
      change_type: "modification"
      current: "Docs claim plan execution engine with parallel scheduling"
      proposed: "Docs describe report/deadlock analysis helpers only"
      compatibility: "safe"
      reason: "No runtime behavior or public type shape changes."
  overall_compatibility: "safe"
  approach:
    strategy: "Direct documentation correction with regression tests"
    steps:
      - "Add failing tests that assert the corrected truth-in-advertising wording."
      - "Update module and item docs/comments to match the existing helper surface."
      - "Run scoped tau-orchestrator verification."
    migration_guide: |
      No migration required. This is documentation alignment for the existing
      public surface.
    version_impact: "patch/minor pre-1.0 — docs-only correction"
```

## Approach
1. Add RED coverage around the exposed documentation strings/semantics in
   `plan_executor`.
2. Rewrite the module/item docs to reflect report generation and deadlock
   analysis only.
3. Verify with scoped `tau-orchestrator` tests.

## Affected Modules
- `crates/tau-orchestrator/src/plan_executor.rs`

## Risks / Mitigations
- Risk: wording drifts again later as aspirational comments re-enter.
  - Mitigation: keep regression assertions around the corrected semantics.

## Verification Plan
- `CARGO_TARGET_DIR=/tmp/tau-target-3627 cargo test -p tau-orchestrator plan_executor -- --test-threads=1`

## Verification Results
- `CARGO_TARGET_DIR=/tmp/tau-target-3627 cargo test -p tau-orchestrator regression_source_docs_describe_reporting_and_deadlock_helpers_only -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/tau-target-3627 cargo test -p tau-orchestrator plan_executor -- --test-threads=1`

## ADR
No ADR required. This is a documentation correction.
