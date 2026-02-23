# Plan: Issue #3396 - E14-04 MCP inventory coverage

## Approach
1. Introduce a deterministic gateway test fixture registrar that registers at least one MCP-prefixed tool (for example `mcp.demo.echo`) plus baseline tools.
2. Add an integration test in `crates/tau-gateway/src/gateway_openresponses/tests.rs` that boots the gateway with this registrar and asserts:
   - `/gateway/tools` returns `200`.
   - response schema metadata remains valid.
   - tool inventory contains the MCP-prefixed tool.
3. Update conformance mapping:
   - mark `E14-04` as `Covered` in `specs/3386/conformance-matrix.md`.
   - add issue-local traceability in `specs/3396/conformance-matrix.md`.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3386/conformance-matrix.md`
- `specs/3396/conformance-matrix.md`

## Risks / Mitigations
- Risk: Existing fixture-based tools tests may become brittle if shared registrar behavior changes.
  - Mitigation: use a dedicated registrar/helper for `E14-04` coverage to avoid impacting existing tests.
- Risk: Non-deterministic tool ordering in inventory output.
  - Mitigation: assert set membership and counts, not positional ordering.

## Interfaces / Contracts
- Existing `GET /gateway/tools` endpoint contract (`schema_version`, `total_tools`, `tools[]`) is reused; no API surface changes.

## ADR
- Not required (no architecture/dependency/protocol decision change).
