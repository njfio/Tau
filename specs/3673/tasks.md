# Tasks: Force Tool-Required Retry Turns In Ralph-Loop Recovery

- [x] T1. Add failing/targeted assertions for required tool choice on gateway retries and CLI prompt rendering.
- [x] T2. Implement agent-level tool-choice override support.
- [x] T3. Apply required tool choice in gateway action retries.
- [x] T4. Honor required tool choice in CLI provider prompt contracts.
- [x] T5. Run scoped gateway/provider verification.

## Evidence refresh — 2026-04-27

- Added provider prompt-contract regressions proving `ToolChoice::Required` renders a hard textual tool-call requirement for Codex, Claude, and Gemini CLI adapters.
- Added gateway regression proving generic mutation recovery retries fall back to `ToolChoice::Required` when the concrete `write` heuristic does not apply.
- Added gateway regression proving a no-tool response after a `ToolChoice::Required` retry blocks with `required_tool_evidence_missing_exhausted` instead of drifting into another soft retry.
- Added gateway/TUI bridge regressions proving `required_tool_evidence_missing_exhausted` is exposed through additive operator snapshots and rendered once in the TUI instead of being replaced by a generic gateway error.
- Added provider tool-choice matrix coverage proving Auto, Required, and concrete Tool prompt contracts across Codex, Claude, and Gemini; concrete Tool now requires the named tool instead of sharing Auto's soft prompt contract.
- Verified scoped provider/gateway regressions plus the full `gateway_openresponses` gateway test filter before closeout.

## Required-tool learning bulletin replay slice

- [x] T6. Define required-tool exhaustion as verifier-blocked recovery evidence: later Ralph turns must receive a `## Learning Insights` bulletin that names `gateway_verifier` and replays the exact `required_tool_evidence_missing_exhausted` reason code.
- [x] T7. Add a focused replay regression that seeds a required-tool exhausted mission, sends a follow-up request, and asserts the follow-up system prompt contains `gateway_verifier` plus `required_tool_evidence_missing_exhausted`.
- [x] T8. Preserve the reason code in the verifier action-history output summary so generic failure-pattern rendering can replay it without special-casing #3673. Existing `gateway_verifier` output-summary preservation covers this path; no #3673-specific runtime branch is required.
- [ ] T9. Verify scoped gateway/provider regressions, format, clippy, and Cargo manifest stability before posting #3673 evidence.

Boundary: `complete_task` remains reserved for explicit mission completion. Required-tool retry exhaustion is fail-closed verifier evidence, so the replay signal stays under `gateway_verifier` with the operational `reason_code` preserved for follow-up learning prompts.
