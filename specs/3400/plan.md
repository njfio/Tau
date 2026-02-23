# Plan: Issue #3400 - F10 provider fallback/circuit-breaker coverage

## Approach
1. Reuse existing deterministic fallback/circuit tests in `tau-provider` for:
   - `F10-02` fallback success on retryable primary failure.
   - `F10-04` circuit opens and skips unhealthy route.
   - `F10-05` half-open cooldown retry of primary route.
   - `F10-08` fallback telemetry payload assertions.
2. Add explicit deterministic tests for currently unmapped scenarios:
   - `F10-01` primary success returns directly without fallback invocation.
   - `F10-03` all routes fail and return deterministic exhaustion/failure behavior.
3. Update conformance artifacts:
   - mark `F10-01/02/03/04/05/08` as `Covered` in `specs/3386/conformance-matrix.md`.
   - add issue-local traceability in `specs/3400/conformance-matrix.md`.

## Affected Modules
- `crates/tau-provider/src/fallback.rs` (tests)
- `specs/3386/conformance-matrix.md`
- `specs/3400/conformance-matrix.md`

## Risks / Mitigations
- Risk: `F10-03` expected error variant may differ by route count and retryability.
  - Mitigation: assert deterministic failure contract at fallback-router boundary and include stable payload checks.
- Risk: introducing flaky timing assertions in circuit logic.
  - Mitigation: continue using deterministic injected clock patterns already present in fallback tests.

## Interfaces / Contracts
- No public API changes.
- Coverage uses existing `FallbackRoutingClient` behavior and existing telemetry event payload schema.

## ADR
- Not required (no new dependencies, architecture, or protocol changes).
