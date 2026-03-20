# 3603 Require Mutating Tool Evidence For Build Completion

## Objective
Prevent Tau from claiming build/create/edit completion for workspace-changing requests unless the current prompt loop contains fresh successful mutating tool evidence.

## Inputs/Outputs
- Input: a build/create/edit/scaffold user prompt, one or more successful non-mutating tool results, and an assistant text reply claiming completion.
- Output: either
  - a replan prompt instructing the model to perform required mutating tool calls first, or
  - a hard structured runtime error after replan budget is exhausted.
- Output must never be a successful build-completion assistant message when only non-mutating tool evidence exists.

## Boundaries/Non-goals
- Do not change TUI rendering.
- Do not require mutation for planning-only, brainstorming, or research-only prompts.
- Do not attempt a universal ontology for every tool; start with a deterministic allowlist of mutating tool names.
- Do not weaken `#3602` zero-tool false-progress guard.

## Failure modes
- A build prompt performs only `read` and then claims the feature/app/game was built.
- The runtime treats any successful tool message as sufficient evidence and accepts the completion.
- Overly broad mutating-tool matching blocks harmless final summaries after real writes/edits.
- Overly broad prompt matching blocks research/planning tasks that do not imply filesystem changes.

## Acceptance criteria
- [ ] Boolean: build/create/edit/scaffold prompts are classified as requiring mutating evidence for completion claims.
- [ ] Boolean: successful `read`-only evidence is insufficient for a build-completion claim.
- [ ] Boolean: a successful `write` or `edit` tool result is sufficient mutating evidence to allow the final completion reply.
- [ ] Boolean: the runtime triggers one replan prompt when the assistant claims completion with only non-mutating evidence.
- [ ] Boolean: repeating the same completion claim after the replan budget is exhausted returns a hard `AgentError`.
- [ ] Boolean: existing `#3602` implementation-progress guard tests remain green.

## Files to touch
- `crates/tau-agent-core/src/runtime_safety_progress.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/tests/structured_output_and_parallel.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`

## Error semantics
- Interior classifier/helper code returns booleans only; no logging.
- Runtime emits one replan on first violation.
- After replan budget is exhausted, runtime returns a structured `AgentError` containing an assistant excerpt.
- No silent fallback to accepting read-only evidence as build completion.

## Test plan
- Add unit tests for mutating-vs-non-mutating tool evidence classification.
- Add regression test proving read-only evidence still fails closed for build completion.
- Add regression test proving write/edit evidence allows final completion.
- Add provider-client/runtime integration test that performs a `read` tool call followed by a fake completion claim and verify hard failure.
