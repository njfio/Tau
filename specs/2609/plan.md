# Plan: Issue #2609 - Under-tested crates coverage wave

## Approach
1. Add RED tests for each AC-mapped behavior seam in diagnostics, training-proxy, and provider client helper layers.
2. Keep implementation changes test-only unless a failing test reveals a genuine defect requiring minimal production adjustment.
3. Run scoped verification gates across the three target crates and confirm no warning/regression drift.
4. Update issue/process artifacts with AC -> test evidence.

## Affected Modules
- `crates/tau-diagnostics/src/lib.rs`
- `crates/tau-training-proxy/src/lib.rs`
- `crates/tau-provider/src/client.rs`
- `specs/2609/spec.md`
- `specs/2609/plan.md`
- `specs/2609/tasks.md`
- `specs/milestones/m104/index.md`

## Risks / Mitigations
- Risk: Test fixtures may become brittle if they overfit formatting details.
  - Mitigation: assert stable behavioral substrings/fields rather than full-string snapshots.
- Risk: Upstream proxy integration tests can flake on networking assumptions.
  - Mitigation: keep tests local with deterministic `httpmock` and explicit request payloads.
- Risk: Env-dependent provider tests may interfere with other tests.
  - Mitigation: reuse existing env-lock patterns for helper tests that mutate env vars.

## Interfaces / Contracts
- `tau-diagnostics`:
  - `parse_doctor_command_args`
  - `execute_policy_command`
  - `percentile_duration_ms` / `render_audit_summary`
- `tau-training-proxy`:
  - `parse_training_proxy_attribution`
  - `handle_chat_completions` behavior via router integration
- `tau-provider::client`:
  - auth helper decisions (`is_azure_openai_endpoint`, `resolved_secret_for_provider`)

## ADR
- Not required: coverage expansion only, no dependency/protocol/architecture decisions.
