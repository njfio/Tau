# Tasks: Issue #3603 - Require mutating tool evidence before implementation completion

## Mutating evidence enforcement slice

- [x] T1 Specify: define mutating tool evidence as the required completion proof for implementation/build/create prompts.
- [x] T2 RED: add a gateway regression where read-only tool calls plus an implementation completion claim remain insufficient.
- [x] T3 GREEN: require mutating tool evidence before accepting final implementation completion claims.
- [x] T4 COMPAT: preserve the #3602 no-tool fabricated progress behavior while adding the stricter read-only-only guard.
- [x] T5 CLOSEOUT: verify scoped gateway recovery tests, format/lint, Cargo manifest stability, push, and publish #3603 evidence.

## Semantics

- Mutating tool evidence means a workspace-changing tool action such as write, edit, patch, or another gateway-classified mutation succeeds during the attempt or its accepted recovery flow.
- Read-only evidence is insufficient for final completion of implementation/build/create/edit prompts, even when the read-only call succeeded and informed the model.
- A completion claim backed only by read-only tools should trigger the same recovery posture as no-tool fabricated progress, but the reason code should make the missing mutation evidence explicit.
- The recovery prompt should tell the model that read-only exploration is not enough and that it must perform a mutating workspace action before claiming completion.
- If the model repeats the completion claim after read-only-only evidence and no mutating tool evidence, the gateway should block with structured operator context before legacy compatibility failure frames.
- Timeout, cancellation, Required tool-choice, and existing verifier recovery semantics must remain compatible with the mutation evidence guard.

## Evidence

- Regression: `issue_3603_read_only_completion_without_mutating_tool_evidence_blocks` proves read-only tool calls plus an implementation completion claim remain insufficient for build/create-style prompts.
- Verifier behavior: first read-only-only completion records `mutation_evidence_missing_continue`; the repeated recovery claim records `mutation_evidence_missing_exhausted` and blocks the mission.
- Verification: `cargo test -p tau-gateway mutating_tool_evidence -- --test-threads=1` passed.
- Compatibility: `cargo test -p tau-gateway issue_3673 -- --test-threads=1` remains the closeout guard for existing Required tool-choice recovery semantics.