# Plan: Issue #3656 - Wire gateway mission loop into Tau action history and learning insights

## Approach
1. Add a gateway-local learning runtime module that:
   - resolves the gateway action-history path under `state_dir/openresponses/`
   - loads/saves `ActionHistoryStore`
   - distills `LearningInsight` from failure patterns and tool success rates
2. Extend the gateway execution handler to capture `ToolExecutionEnd` events
   from the agent runtime and persist them as action-history records linked to
   the current session/mission request.
3. Append formatted learning guidance to the system prompt for new gateway
   mission requests, preserving the existing Cortex bulletin composition.
4. Keep the compatibility boundary additive:
   - no new required request fields
   - no response schema change
   - learning context is advisory prompt material, not a new hard blocker

## Proposed Design
### Persistence
- Path: `state_dir/openresponses/action-history.jsonl`
- Format: existing `tau-memory::action_history::ActionRecord`
- Write timing: after each outer attempt completes, persist any new tool-end
  records captured during that attempt

### Distillation
- Use `ActionHistoryStore::failure_patterns(lookback)` for failing-tools insight
- Use `ActionHistoryStore::tool_success_rates(lookback)` for declining-rate
  insight
- Convert directly into `tau_agent_core::LearningInsight`
- Format with `format_learning_bulletin`

### Prompt Injection
- Base prompt remains `state.resolved_system_prompt()`, which already includes
  Cortex bulletin context when present
- If learned guidance is non-empty, append it before auto-selected skill
  augmentation so each new request gets:
  base system prompt + Cortex bulletin + Learning Insights + skill prompts

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3656"
  change_surface:
    - symbol: "Gateway request execution side effects"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "gateway request execution does not write Tau action-history records"
      proposed: "gateway request execution persists tool outcomes to action history"
      compatibility: "caution"
      reason: "adds durable learning artifacts but keeps the request/response contract unchanged"
    - symbol: "Gateway system prompt composition"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "system prompt includes configured prompt plus optional Cortex bulletin and skills"
      proposed: "system prompt may also append learned insights derived from prior action history"
      compatibility: "caution"
      reason: "changes model context, not API shape"
  overall_compatibility: "caution"
  approach:
    strategy: "Direct additive learning integration"
    steps:
      - "Persist gateway tool outcomes into ActionHistoryStore"
      - "Distill LearningInsight from recent gateway action history"
      - "Append formatted learning bulletin to request system prompts"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: low-value noise pollutes the system prompt.
  Mitigation: only append formatted insights when the distilled learning section
  is non-empty.
- Risk: action-history writes become duplicated across retries.
  Mitigation: persist only the new captured tool-end events for each attempt.
- Risk: learning context fights with Cortex bulletin context.
  Mitigation: keep the learning section separately headed and append it after
  the base Cortex-composed prompt.

## Verification
- Unit coverage for learning-insight distillation from action history
- Regression coverage for action-history persistence and learned prompt injection
- Existing mission-loop/session retry tests stay green
