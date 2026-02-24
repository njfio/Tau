# M313 - E2E core scenario depth verification wave

Status: Active

## Context
M313 deepens E2E coverage by adding one deterministic gate that aggregates
integration-package memory/tool scenarios with gateway lifecycle/session/core
contract tests into a single auditable verification report.

Primary sources:
- `tests/integration/tests/agent_tool_memory_roundtrip.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/tau-e2e-testing-prd.md`

## Issue Hierarchy
- Epic: #3508
- Story: #3509
- Task: #3510

## Scope
- Add deterministic M313 E2E core-depth verification script and report.
- Add script contract test with fail-closed required-step checks.
- Map integration + gateway E2E selectors to executable commands.
- Update README links with M313 verification entrypoint.

## Exit Criteria
- `specs/3510/spec.md` is `Implemented` with AC evidence.
- M313 script report includes all required E2E step IDs.
- Contract test fails closed on missing required-step IDs.
- README includes M313 verification entrypoint.
