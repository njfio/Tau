# Plan: Issue #3398 - E14 tool-builder WASM coverage

## Approach
1. Reuse deterministic existing tool-builder tests for `E14-01` and `E14-02`:
   - `functional_tool_builder_tool_builds_wasm_artifacts`
   - `integration_tool_builder_generated_tool_executes_through_extension_runtime`
2. Add/extend deterministic regression coverage for `E14-03` in generated tool runtime tests to assert fail-closed sandbox behavior for a misbehaving WASM module under bounded limits.
3. Update conformance mapping:
   - mark `E14-01..E14-03` as `Covered` in `specs/3386/conformance-matrix.md`.
   - add issue-local traceability in `specs/3398/conformance-matrix.md`.

## Affected Modules
- `crates/tau-runtime/src/generated_tool_builder_runtime.rs` (tests)
- `specs/3386/conformance-matrix.md`
- `specs/3398/conformance-matrix.md`

## Risks / Mitigations
- Risk: sandbox failure reason codes may vary by trap mode (fuel vs timeout).
  - Mitigation: assert stable fail-closed reason contract at generated-tool layer (`generated_tool_sandbox_validation_failed`) and validate diagnostics/reason detail conservatively.
- Risk: accidental broad test runtime increase.
  - Mitigation: keep new test scoped to a single deterministic unit/regression case.

## Interfaces / Contracts
- No new public APIs; uses existing generated-tool builder/runtime contract and existing conformance matrix artifacts.

## ADR
- Not required (no architecture/dependency/protocol decision changes).
