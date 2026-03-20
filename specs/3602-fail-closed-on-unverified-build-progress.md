# 3602 Fail Closed On Unverified Build Progress

## Objective
Prevent Tau from surfacing fabricated implementation progress for build/create/edit/scaffold requests when no fresh successful tool evidence exists in the current turn.

## Inputs/Outputs
- Input: a user prompt requesting implementation work that implies workspace changes, followed by an assistant text response with no tool calls in the turn.
- Output: either
  - a replan prompt that forces the next turn to request tools, or
  - a hard structured runtime error after replan budget is exhausted.
- Output must never be a successful assistant progress/completion message without fresh tool evidence.

## Boundaries/Non-goals
- Do not redesign the TUI or add progress spinners.
- Do not guarantee a specific tool will be chosen.
- Do not add new tools or relax existing guards.
- Do not classify pure planning/advice requests as implementation progress failures.

## Failure modes
- A natural-language build/create prompt without explicit file/path terms bypasses the existing side-effect guard.
- The assistant claims progress such as "core systems are in place" or "I implemented" without any tool calls or successful tool results.
- The runtime accepts that assistant text and emits it as a normal assistant message instead of replanning or failing.
- Overly broad matching blocks harmless planning-only responses for requests that do not actually imply workspace changes.

## Acceptance criteria
- [ ] Boolean: prompts like "create a snake and tetris mashup game using phaserjs" are treated as implementation requests even without explicit file/path mentions.
- [ ] Boolean: assistant implementation-progress text with no tool evidence triggers a single replan prompt.
- [ ] Boolean: repeating the same implementation-progress text after the replan budget is exhausted returns a hard `AgentError` rather than a successful assistant turn.
- [ ] Boolean: existing unverified side-effect completion tests remain green.
- [ ] Boolean: existing unverified external research tests remain green.
- [ ] Boolean: planning-only responses for non-implementation prompts are not misclassified.

## Files to touch
- `crates/tau-agent-core/src/runtime_safety_memory.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/tests/structured_output_and_parallel.rs`

## Error semantics
- Interior classifier/helper code returns booleans only; no logging.
- Runtime loop emits one replan on first violation.
- After replan budget is exhausted, runtime returns a structured `AgentError` with an excerpt of the offending assistant text.
- No silent fallback to accepting fabricated progress.

## Test plan
- Add unit tests for implementation-request and implementation-progress classifiers.
- Add regression test proving a natural-language build prompt replans before accepting completion.
- Add regression test proving the same prompt fails hard after replan budget is exhausted.
- Re-run nearby side-effect and external-research guard tests to ensure no regressions.
