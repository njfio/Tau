# Spec: Issue #3398 - Cover E14-01..E14-03 tool-builder WASM scenarios

Status: Implemented

## Problem Statement
`specs/3386/conformance-matrix.md` still marks `E14-01`, `E14-02`, and `E14-03` as `N/A`, even though the repository has deterministic tool-builder/WASM runtime coverage surfaces. We need explicit conformance mapping and deterministic sandbox-limit failure coverage for misbehaving WASM modules.

## Scope
In scope:
- Validate and map executable coverage for:
  - `E14-01` tool builder generates WASM artifacts.
  - `E14-02` generated tool executes through extension runtime.
  - `E14-03` misbehaving WASM fails closed under sandbox limits.
- Add/extend deterministic test coverage for `E14-03` if needed.
- Update conformance mapping so `E14-01..E14-03` are no longer `N/A`.

Out of scope:
- New gateway API endpoints.
- MCP inventory scenario (`E14-04`, already delivered via #3396).

## Acceptance Criteria
### AC-1 E14-01 artifact generation is executable and traceable
Given deterministic tool-builder tests,
when build flows execute,
then generated WASM module/manifest artifacts are produced and asserted.

### AC-2 E14-02 generated tool execution is executable and traceable
Given generated extension registrations,
when the generated tool executes through extension runtime,
then execution succeeds with deterministic result assertions.

### AC-3 E14-03 sandbox-limit failure is executable and traceable
Given a misbehaving WASM payload under bounded sandbox limits,
when validation/execution runs,
then the runtime fails closed with deterministic sandbox-failure semantics.

### AC-4 Conformance traceability is updated
Given issue conformance artifacts,
when reviewed,
then `E14-01`, `E14-02`, and `E14-03` are mapped to executable coverage instead of `N/A`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | tool-builder runtime enabled in deterministic test policy | build generated tool | module/manifest artifacts exist |
| C-02 | AC-2 | Integration | generated extension registration from tool-builder output | execute registered generated tool | deterministic success payload |
| C-03 | AC-3 | Regression | misbehaving/infinite-loop WASM and bounded limits | run generated-tool sandbox validation | fail-closed sandbox reason surfaced |
| C-04 | AC-4 | Conformance | `E14-01..E14-03` rows in matrix | update mappings | all three marked Covered |

## Success Metrics / Observable Signals
- Deterministic executable tests cover `E14-01..E14-03`.
- `specs/3386/conformance-matrix.md` and `specs/3398/conformance-matrix.md` map `E14-01..E14-03` to executable tests.
