# Plan: Issue #3732 - Immutable safety floor for gateway safety endpoints

## Approach
1. Strengthen `enforce_safety_policy_floor` so it also requires
   `secret_leak_detection_enabled=true` and
   `apply_to_outbound_http_payloads=true`.
2. Strengthen `enforce_safety_rules_floor` so default rules must preserve
   `pattern`, `matcher`, and `reason_code`, not just `rule_id` and `enabled`.
3. Keep gateway endpoint wiring unchanged except for consuming the stronger floor
   checks.
4. Add focused regression tests for the newly blocked bypasses.

## Risks / Mitigations
- Risk: stricter floor rejects legitimate tuning updates.
  Mitigation: keep warn/redact/block mode changes and custom-rule supersets
  allowed; only immutable baseline protections are enforced.
- Risk: rule comparison logic becomes brittle.
  Mitigation: compare only the immutable fields that determine effective
  protection: `matcher`, `pattern`, `reason_code`, and enabled state.

## Verification
- `cargo test -p tau-safety security_enforce_safety_policy_floor_rejects_secret_leak_detection_disabled -- --nocapture`
- `cargo test -p tau-safety security_enforce_safety_policy_floor_rejects_outbound_payload_scanning_disabled -- --nocapture`
- `cargo test -p tau-safety security_enforce_safety_rules_floor_rejects_modified_default_prompt_rule_pattern -- --nocapture`
- `cargo test -p tau-safety security_enforce_safety_rules_floor_rejects_modified_default_secret_rule_reason_code -- --nocapture`
- `cargo check -p tau-gateway`
- `cargo fmt --check`
