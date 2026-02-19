# Plan: Issue #2602 - G4 phase-2 branch tool runtime orchestration + limits

## Approach
1. Add RED conformance/regression tests in `tau-agent-core` for branch follow-up execution, structured conclusions, and concurrency limits.
2. Extend `AgentConfig` with `max_concurrent_branches_per_session` and safe default.
3. Implement branch follow-up pipeline in tool-result handling:
   - Parse branch tool arguments/result metadata.
   - Run isolated branch execution from `Agent::fork()`.
   - Restrict branch execution tools to memory-only set.
   - Merge branch conclusion into final tool result payload.
4. Add per-session branch concurrency guard with deterministic rejection payload.
5. Run verify gates and update roadmap/spec artifacts.

## Affected Modules
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/tests/config_and_direct_message.rs`
- `crates/tau-agent-core/src/tests/streaming_and_budgets.rs`
- `specs/2602/spec.md`
- `specs/2602/plan.md`
- `specs/2602/tasks.md`
- `specs/milestones/m103/index.md`
- `tasks/spacebot-comparison.md`

## Risks / Mitigations
- Risk: Branch follow-up execution could recurse or leak user-facing tools.
  - Mitigation: hard-filter branch tool registry to `memory_*` tools only.
- Risk: Parallel tool execution may race branch follow-up state.
  - Mitigation: atomic active-branch guard and deterministic limit rejection payload.
- Risk: Directive regressions (skip/react/send_file) from tool-result path edits.
  - Mitigation: preserve directive parsing path and run existing directive regression tests.

## Interfaces / Contracts
- New config contract:
  - `AgentConfig.max_concurrent_branches_per_session: usize` (clamped to minimum 1 at runtime)
- Branch follow-up success payload extensions:
  - `reason_code: "branch_conclusion_ready"`
  - `branch_conclusion: <string>`
  - `branch_followup: { status, tools_mode, branch_message_count, ... }`
- Branch follow-up error payload reason codes:
  - `branch_prompt_missing`
  - `branch_concurrency_limit_exceeded`
  - `branch_execution_failed`

## ADR
- Not required: no external dependency additions and no architecture-level wire/protocol changes.
