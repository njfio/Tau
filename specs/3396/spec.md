# Spec: Issue #3396 - Cover E14-04 MCP inventory scenario in gateway E2E

Status: Accepted

## Problem Statement
`specs/3386/conformance-matrix.md` marks `E14-04` as `N/A`, but the current gateway contract already exposes `/gateway/tools` inventory. We need deterministic coverage proving MCP-style registered tools are surfaced by this endpoint.

## Scope
In scope:
- Add deterministic gateway test coverage for `E14-04` using MCP-prefixed tool registrations.
- Assert `/gateway/tools` inventory includes registered MCP tool entries.
- Update conformance mapping so `E14-04` is no longer `N/A`.

Out of scope:
- Tool builder compile/runtime flows (`E14-01`, `E14-02`, `E14-03`).
- New endpoint additions or protocol changes.

## Acceptance Criteria
### AC-1 MCP-prefixed tool registration is visible in gateway inventory
Given a gateway server configured with a deterministic tool registrar that includes MCP-prefixed tool names,
when `/gateway/tools` is requested,
then the response includes those MCP tool names in inventory.

### AC-2 Existing tools inventory contract remains valid
Given the MCP inventory scenario above,
when inventory metadata is inspected,
then schema and total counts remain consistent with registered tools.

### AC-3 Conformance traceability is updated
Given issue conformance artifacts,
when reviewed,
then `E14-04` is mapped to executable coverage instead of `N/A`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Integration | deterministic fixture registrar with MCP-prefixed tool name | GET `/gateway/tools` | inventory includes MCP-prefixed tool |
| C-02 | AC-2 | Integration | same fixture registrar | inspect inventory schema/count fields | schema/count contract is valid |
| C-03 | AC-3 | Conformance | `E14-04` row in matrix | update mapping | `E14-04` marked Covered with test reference |

## Success Metrics / Observable Signals
- Deterministic gateway test covers `E14-04`.
- `specs/3386/conformance-matrix.md` and `specs/3396/conformance-matrix.md` map `E14-04` to executable coverage.
