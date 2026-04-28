# Tasks: Issue #3602 - Fail closed on fabricated implementation progress

## Fail-closed completion evidence slice

- [x] T1 Specify: define the no-tool fabricated progress contract for implementation/build/create prompts.
- [x] T2 RED: add a gateway regression where the model claims completion for a workspace-changing request without any tool execution evidence.
- [x] T3 GREEN: trigger one recovery prompt for claimed completion without tool evidence, then block with structured operator context if the model repeats the claim.
- [x] T4 COMPAT: keep legacy `response.failed` compatibility frames after the richer blocked operator state.
- [x] T5 CLOSEOUT: verify scoped gateway recovery tests, format/lint, Cargo manifest stability, push, and publish #3602 evidence.

## Semantics

- Fabricated progress means the assistant claims implementation/build/create/edit work was completed while the gateway observed no tool execution evidence for the attempt.
- Workspace-changing prompts include natural-language build/create/edit requests even when the user does not name an explicit file path.
- A claimed completion with no tool evidence must not be treated as a successful OpenResponses turn.
- The gateway may issue one recovery prompt that names the no tool evidence problem and instructs the model to use workspace tools before claiming progress.
- If the model again claims completion without tool evidence after the recovery prompt, the turn must fail closed with structured blocked operator context instead of a generic success.
- The blocked outcome should remain additive for newer clients while preserving legacy compatibility failure frames.

## Evidence

- Regression: `issue_3602_fabricated_progress_without_tool_evidence_blocks_with_structured_reason` proves a natural-language implementation prompt cannot pass by assistant text alone.
- Verifier behavior: first assistant-only completion claim records `claimed_completion_without_tool_evidence_continue`; the repeated recovery claim records `claimed_completion_without_tool_evidence_exhausted` and returns a blocked gateway failure.
- Verification: `cargo test -p tau-gateway fabricated_progress -- --test-threads=1` passed.
- Compatibility: the blocked gateway outcome still returns a legacy failure payload containing `claimed completion without tool evidence` for existing clients.