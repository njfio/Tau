# Spec: Issue #3732 - Immutable safety floor for gateway safety endpoints

Status: Reviewed

## Problem Statement
The gateway safety policy and safety rules PUT endpoints currently accept
mutations that can weaken core protections. An authenticated caller could
disable safety, disable secret-leak detection, disable outbound payload
scanning, or preserve default rule IDs while replacing their matcher/pattern
contents with harmless values. Tau needs an immutable minimum floor so runtime
safety cannot be silently self-disabled through configuration writes.

## Scope
In scope:
- `crates/tau-safety/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/safety_runtime.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3732/spec.md`
- `specs/3732/plan.md`
- `specs/3732/tasks.md`

Out of scope:
- retroactive migration/clamping of already persisted unsafe files on read
- changing non-floor safety tuning such as warn/redact/block modes
- new auth or routing policy for gateway safety endpoints

## Acceptance Criteria
### AC-1 Policy floor rejects disabling core safety toggles
Given a safety policy update request,
when the caller sets `enabled=false`, `apply_to_inbound_messages=false`,
`apply_to_tool_outputs=false`, `secret_leak_detection_enabled=false`, or
`apply_to_outbound_http_payloads=false`,
then the safety floor rejects the mutation.

### AC-2 Rules floor preserves default rule definitions
Given a safety rules update request,
when the caller removes a default rule, disables a default rule, or changes a
default rule's `pattern`, `matcher`, or `reason_code`,
then the safety floor rejects the mutation.

### AC-3 Valid superset updates remain allowed
Given a valid safety policy and a ruleset that preserves all default rules while
adding extra custom rules,
when the caller submits the update,
then the gateway accepts the mutation.

### AC-4 Gateway PUT endpoints fail closed with explicit floor violations
Given a floor-violating gateway PUT request,
when the endpoint validates the request,
then it returns `400` with error code `safety_floor_violation`.

## Conformance Cases
- C-01 / AC-1 / Regression: reject disabling secret-leak detection or outbound
  payload scanning in `enforce_safety_policy_floor`.
- C-02 / AC-2 / Regression: reject modified default rule content in
  `enforce_safety_rules_floor`.
- C-03 / AC-3 / Functional: accept default-valid policy and a superset ruleset.
- C-04 / AC-4 / Integration: gateway safety PUT handlers return `400
  safety_floor_violation` for floor-breaking payloads.

## Success Metrics / Observable Signals
- Gateway config writes can no longer weaken the minimum safety baseline.
- Default rule definitions are immutable, not just their IDs.
- Safety floor violations are surfaced consistently as bad requests.
