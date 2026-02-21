# Plan: Issue #3264 - move stream_openresponses handler to dedicated module

## Approach
1. RED: tighten root guard and assert `stream_openresponses` is no longer declared in root.
2. Add `stream_response_handler.rs` and move `stream_openresponses` implementation.
3. Import moved helper into root so openresponses route flow remains unchanged.
4. Verify with openresponses stream/non-stream tests, guard script, fmt, clippy.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/stream_response_handler.rs` (new)
- `scripts/dev/test-gateway-openresponses-size.sh`
- `specs/milestones/m243/index.md`
- `specs/3264/spec.md`
- `specs/3264/plan.md`
- `specs/3264/tasks.md`

## Risks & Mitigations
- Risk: stream event framing could drift during extraction.
  - Mitigation: run existing stream functional test and malformed-json regression.
- Risk: import visibility regression.
  - Mitigation: explicit root import and compile/test verification.

## Interfaces / Contracts
- Public endpoint paths unchanged.
- Stream response event names/payloads unchanged.

## ADR
No ADR required (internal handler extraction only).
