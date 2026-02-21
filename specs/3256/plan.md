# Plan: Issue #3256 - move openresponses preflight helpers to dedicated module

## Approach
1. RED: tighten root guard to `600` and assert moved preflight helper functions are not declared in root.
2. Add `request_preflight.rs` and move preflight helper functions.
3. Import moved helpers into root so `handle_openresponses` and related flow continue unchanged.
4. Verify with preflight conformance tests, guard script, fmt, clippy.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/request_preflight.rs` (new)
- `scripts/dev/test-gateway-openresponses-size.sh`
- `specs/milestones/m241/index.md`
- `specs/3256/spec.md`
- `specs/3256/plan.md`
- `specs/3256/tasks.md`

## Risks & Mitigations
- Risk: helper visibility/import regressions break openresponses compile path.
  - Mitigation: explicit root imports + targeted preflight conformance tests.
- Risk: subtle preflight semantics drift.
  - Mitigation: existing conformance + regression test set in AC-1.

## Interfaces / Contracts
- Public API paths unchanged.
- Preflight policy/validation behavior unchanged.

## ADR
No ADR required (internal helper extraction only).
