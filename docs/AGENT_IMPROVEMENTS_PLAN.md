# Tau Agent Improvements — Implementation & Integration Plan

> Comprehensive plan for addressing all identified limitations and gaps in the Tau coding agent codebase across error handling, tool execution, context management, planning, parallel execution, learning/adaptation, self-repair, and guardrails.

---

## Table of Contents

1. [Phase 1: Foundation — Config Defaults & Observability](#phase-1-foundation--config-defaults--observability)
2. [Phase 2: Resilience — Adaptive Retry & Circuit Breaker](#phase-2-resilience--adaptive-retry--circuit-breaker)
3. [Phase 3: Tool Intelligence — Health Metrics & Tool-Level Retry](#phase-3-tool-intelligence--health-metrics--tool-level-retry)
4. [Phase 4: Context — Predictive Compaction & Semantic Ranking](#phase-4-context--predictive-compaction--semantic-ranking)
5. [Phase 5: Planning — Structured Plans with DAG Representation](#phase-5-planning--structured-plans-with-dag-representation)
6. [Phase 6: Parallelism — Inter-Agent Communication & Dynamic Concurrency](#phase-6-parallelism--inter-agent-communication--dynamic-concurrency)
7. [Phase 7: Learning — Persistent Action History & Feedback Loops](#phase-7-learning--persistent-action-history--feedback-loops)
8. [Phase 8: Self-Repair — General Failure Recovery](#phase-8-self-repair--general-failure-recovery)
9. [Phase 9: Safety — Hard Limits, PII Redaction & Contextual Analysis](#phase-9-safety--hard-limits-pii-redaction--contextual-analysis)
10. [Cross-Cutting Concerns](#cross-cutting-concerns)
11. [Migration & Rollout Strategy](#migration--rollout-strategy)

---

## Phase 1: Foundation — Config Defaults & Observability

**Goal:** Tune existing defaults for better out-of-box performance, add observability foundation.

### 1.1 Config Default Tuning

**File:** `crates/tau-agent-core/src/lib.rs` (lines 153–201, `AgentConfig::default()`)

| Parameter | Current | Proposed | Rationale |
|-----------|---------|----------|-----------|
| `request_max_retries` | 2 | 3 | More resilience against transient failures |
| `structured_output_max_retries` | 1 | 3 | JSON formatting issues often need 2+ attempts |
| `react_max_replans_on_tool_failure` | 1 | 2 | One replan is rarely enough for complex tasks |
| `max_concurrent_branches_per_session` | 2 | 4 | Modern tasks benefit from more parallelism |
| `memory_retrieval_limit` | 3 | 5 | More memory context = better decisions |
| `memory_max_chars_per_item` | 180 | 320 | 180 chars truncates useful context |

**Implementation:**
```rust
// crates/tau-agent-core/src/lib.rs — AgentConfig::default()
request_max_retries: 3,
structured_output_max_retries: 3,
react_max_replans_on_tool_failure: 2,
max_concurrent_branches_per_session: 4,
memory_retrieval_limit: 5,
memory_max_chars_per_item: 320,
```

**Tests to update:** Any test asserting default values in `crates/tau-agent-core/src/lib.rs` and `crates/tau-agent-core/src/runtime_turn_loop.rs`.

### 1.2 Agent Metrics Infrastructure

**New file:** `crates/tau-agent-core/src/metrics.rs`

Create a lightweight, lock-free metrics collection struct:

```rust
/// Per-session agent metrics collected during execution.
#[derive(Debug, Default)]
pub struct AgentMetrics {
    // LLM request metrics
    pub llm_requests_total: AtomicU64,
    pub llm_retries_total: AtomicU64,
    pub llm_failures_total: AtomicU64,
    pub llm_latency_sum_ms: AtomicU64,

    // Tool metrics
    pub tool_executions_total: AtomicU64,
    pub tool_failures_total: AtomicU64,
    pub tool_timeouts_total: AtomicU64,
    pub tool_latency_sum_ms: AtomicU64,

    // Context metrics
    pub compactions_total: AtomicU64,
    pub compaction_messages_dropped: AtomicU64,

    // Replan metrics
    pub replans_total: AtomicU64,
    pub replan_successes: AtomicU64,

    // Per-tool breakdown (name -> ToolHealthStats)
    pub tool_health: Mutex<HashMap<String, ToolHealthStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct ToolHealthStats {
    pub executions: u64,
    pub failures: u64,
    pub timeouts: u64,
    pub total_latency_ms: u64,
}
```

**Integration points:**
- Wire into `AgentRuntime` (lib.rs ~line 395) as `Arc<AgentMetrics>`
- Instrument `complete_with_retry` (lib.rs ~line 2947) for LLM metrics
- Instrument `execute_tool_calls` (lib.rs ~line 3074) for tool metrics
- Instrument `compact_messages_for_tier` (runtime_turn_loop.rs) for compaction metrics
- Expose `metrics_snapshot()` method on `AgentRuntime` for external consumption

### 1.3 Structured Event Logging

**File:** `crates/tau-events/src/lib.rs`

Add new event variants for observability:

```rust
pub enum AgentEvent {
    // Existing events...

    // New observability events
    ToolHealthReport {
        tool_name: String,
        success_rate: f64,
        avg_latency_ms: u64,
        consecutive_failures: u32,
    },
    CircuitBreakerTripped {
        tool_name: String,
        reason: String,
    },
    ContextCompactionTriggered {
        tier: String,
        messages_before: usize,
        messages_after: usize,
        estimated_tokens_freed: u32,
    },
    ReplanTriggered {
        reason: String,
        attempt: usize,
        failed_tools: Vec<String>,
    },
}
```

---

## Phase 2: Resilience — Adaptive Retry & Circuit Breaker

**Goal:** Make retry logic smarter and add circuit breaker protection.

### 2.1 Categorized Error Classification

**File:** `crates/tau-agent-core/src/runtime_turn_loop.rs` (extend `is_retryable_ai_error`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Transient — retry with backoff (rate limits, network blips)
    Transient,
    /// Degraded — retry with longer backoff, consider fallback
    Degraded,
    /// Permanent — do not retry (auth failures, bad requests)
    Permanent,
    /// Unknown — retry once conservatively
    Unknown,
}

pub fn categorize_error(error: &TauAiError) -> ErrorCategory {
    match error {
        TauAiError::RateLimited { .. } => ErrorCategory::Transient,
        TauAiError::ServerError { status, .. } if *status >= 500 => ErrorCategory::Transient,
        TauAiError::Timeout { .. } => ErrorCategory::Transient,
        TauAiError::AuthError { .. } => ErrorCategory::Permanent,
        TauAiError::BadRequest { .. } => ErrorCategory::Permanent,
        TauAiError::ModelOverloaded { .. } => ErrorCategory::Degraded,
        _ => ErrorCategory::Unknown,
    }
}
```

### 2.2 Adaptive Backoff Strategy

**File:** `crates/tau-agent-core/src/lib.rs` (modify `complete_with_retry` ~line 2947)

Replace fixed exponential backoff with adaptive strategy:

```rust
fn compute_adaptive_backoff(
    attempt: usize,
    category: ErrorCategory,
    config: &AgentConfig,
) -> Duration {
    let base = config.request_retry_initial_backoff_ms;
    let max = config.request_retry_max_backoff_ms;

    let multiplier = match category {
        ErrorCategory::Transient => 2u64.pow(attempt as u32),
        ErrorCategory::Degraded => 3u64.pow(attempt as u32),  // More aggressive backoff
        ErrorCategory::Unknown => 2u64.pow(attempt as u32 + 1), // Conservative
        ErrorCategory::Permanent => return Duration::ZERO, // Don't retry
    };

    let delay_ms = (base * multiplier).min(max);
    // Add jitter (±25%)
    let jitter = delay_ms / 4;
    let jittered = delay_ms + (rand::random::<u64>() % (jitter * 2)) - jitter;

    Duration::from_millis(jittered.min(max))
}
```

**Modify `complete_with_retry`** to use `categorize_error` and `compute_adaptive_backoff` instead of the current fixed backoff logic.

### 2.3 Circuit Breaker for LLM Providers

**New file:** `crates/tau-agent-core/src/circuit_breaker.rs`

```rust
/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing — reject requests immediately
    HalfOpen,  // Testing recovery — allow one request through
}

pub struct CircuitBreaker {
    state: AtomicU8,  // 0=Closed, 1=Open, 2=HalfOpen
    consecutive_failures: AtomicU32,
    last_failure_time: AtomicU64,
    failure_threshold: u32,        // Default: 5
    recovery_timeout_ms: u64,      // Default: 30_000
}

impl CircuitBreaker {
    pub fn should_allow_request(&self) -> bool { ... }
    pub fn record_success(&self) { ... }
    pub fn record_failure(&self) { ... }
}
```

**Integration:**
- Add `CircuitBreaker` field to `AgentRuntime` (lib.rs ~line 395)
- Check circuit breaker state at start of `complete_with_retry`
- Record success/failure after each LLM call
- When circuit is open, return `AgentError::CircuitBreakerOpen` immediately
- Add new `AgentConfig` fields: `circuit_breaker_failure_threshold` (default: 5), `circuit_breaker_recovery_timeout_ms` (default: 30_000)

### 2.4 Fallback Model Support

**File:** `crates/tau-agent-core/src/lib.rs`

Add fallback model chain to `AgentConfig`:

```rust
pub struct AgentConfig {
    // ... existing fields ...
    pub fallback_models: Vec<String>,  // Ordered list of fallback models
}
```

**Modify `complete_with_retry`:** When primary model exhausts retries, attempt fallback models in order before returning error.

---

## Phase 3: Tool Intelligence — Health Metrics & Tool-Level Retry

**Goal:** Make tools self-healing with per-tool retry and health tracking.

### 3.1 Per-Tool Health Tracking

**File:** `crates/tau-agent-core/src/metrics.rs` (from Phase 1)

The `ToolHealthStats` struct tracks per-tool performance. Add rolling window tracking:

```rust
pub struct ToolHealthTracker {
    /// Per-tool stats, keyed by tool name
    stats: DashMap<String, ToolHealthWindow>,
}

pub struct ToolHealthWindow {
    /// Ring buffer of last N execution results
    recent_results: VecDeque<ToolExecutionRecord>,
    window_size: usize,  // Default: 20
}

pub struct ToolExecutionRecord {
    pub success: bool,
    pub latency_ms: u64,
    pub timestamp_ms: u64,
}

impl ToolHealthTracker {
    /// Returns success rate over the rolling window (0.0 to 1.0)
    pub fn success_rate(&self, tool_name: &str) -> f64 { ... }

    /// Returns p50/p95/p99 latency estimates
    pub fn latency_percentiles(&self, tool_name: &str) -> LatencyPercentiles { ... }

    /// Returns true if tool appears to be in a degraded state
    pub fn is_degraded(&self, tool_name: &str) -> bool {
        self.success_rate(tool_name) < 0.5
    }
}
```

### 3.2 Tool-Level Retry with Circuit Breaker

**File:** `crates/tau-agent-core/src/runtime_tool_bridge.rs` (modify `execute_tool_call_inner`)

Wrap tool execution with per-tool retry:

```rust
pub struct ToolRetryPolicy {
    pub max_retries: usize,          // Default: 1
    pub retry_backoff_ms: u64,       // Default: 500
    pub retryable_on_timeout: bool,  // Default: true
    pub retryable_on_error: bool,    // Default: false (tool errors are usually deterministic)
}
```

**Add to `AgentConfig`:**
```rust
pub tool_retry_max_attempts: usize,       // Default: 1 (= one retry after failure)
pub tool_retry_backoff_ms: u64,           // Default: 500
pub tool_retry_on_timeout: bool,          // Default: true
```

**Implementation in `execute_tool_call_inner`:**
1. Execute tool
2. If timeout and `tool_retry_on_timeout`, retry up to `tool_retry_max_attempts`
3. Record result in `ToolHealthTracker`
4. If tool is degraded (via health tracker), emit `ToolHealthReport` event

### 3.3 Progressive Replan Trigger

**File:** `crates/tau-agent-core/src/lib.rs` (modify replan logic ~line 1934+)

Current behavior: Replan only when ALL tools fail.
New behavior: Replan when failure ratio exceeds threshold.

```rust
pub struct ReplanPolicy {
    pub max_replans: usize,                     // From config
    pub tool_failure_ratio_threshold: f64,      // Default: 0.5 (replan if ≥50% tools fail)
    pub partial_failure_replan_enabled: bool,   // Default: true
}
```

**Add to `AgentConfig`:**
```rust
pub react_replan_failure_ratio_threshold: f64,  // Default: 0.5
pub react_partial_failure_replan: bool,          // Default: true
```

**Modified replan check** in the turn loop:
```rust
let failure_ratio = tool_failures as f64 / total_tool_calls as f64;
let should_replan = if config.react_partial_failure_replan {
    failure_ratio >= config.react_replan_failure_ratio_threshold
} else {
    failure_ratio >= 1.0  // Current behavior: all must fail
};
```

---

## Phase 4: Context — Predictive Compaction & Semantic Ranking

**Goal:** Prevent context budget overruns proactively, preserve important context.

### 4.1 Proactive Token Budget Forecasting

**File:** `crates/tau-agent-core/src/runtime_turn_loop.rs`

Add pre-request token forecasting:

```rust
pub struct TokenBudgetForecast {
    pub current_usage_tokens: u32,
    pub estimated_response_tokens: u32,
    pub estimated_next_turn_tokens: u32,
    pub budget_tokens: u32,
    pub will_exceed_budget: bool,
    pub recommended_tier: Option<ContextCompactionTier>,
}

pub fn forecast_token_budget(
    request: &ChatRequest,
    config: &AgentConfig,
    recent_response_tokens: &[u32],  // Last N response sizes for estimation
) -> TokenBudgetForecast {
    let current = estimate_chat_request_tokens(request);
    let avg_response = if recent_response_tokens.is_empty() {
        4096  // Conservative default
    } else {
        recent_response_tokens.iter().sum::<u32>() / recent_response_tokens.len() as u32
    };

    let projected_next = current.input_tokens + avg_response * 2;  // Current + response + next response
    let budget = config.max_estimated_input_tokens.unwrap_or(120_000);

    let utilization = projected_next as f64 / budget as f64;
    let recommended_tier = if utilization >= 0.95 {
        Some(ContextCompactionTier::Emergency)
    } else if utilization >= 0.85 {
        Some(ContextCompactionTier::Aggressive)
    } else if utilization >= 0.80 {
        Some(ContextCompactionTier::Warn)
    } else {
        None
    };

    TokenBudgetForecast {
        current_usage_tokens: current.input_tokens,
        estimated_response_tokens: avg_response,
        estimated_next_turn_tokens: projected_next,
        budget_tokens: budget,
        will_exceed_budget: projected_next > budget,
        recommended_tier,
    }
}
```

**Integration:** Call `forecast_token_budget` BEFORE building the next ChatRequest (in the turn loop, lib.rs ~line 1700). If forecasted to exceed, compact preemptively.

### 4.2 Improved Token Estimation

**File:** `crates/tau-agent-core/src/runtime_turn_loop.rs` (modify `estimate_text_tokens`)

Replace `chars / 4` heuristic with a more accurate estimator:

```rust
fn estimate_text_tokens(text: &str) -> u32 {
    // Hybrid estimation:
    // 1. ASCII-heavy text: ~4 chars per token
    // 2. Code: ~3.5 chars per token (more special chars)
    // 3. Unicode-heavy: ~2 chars per token
    let ascii_ratio = text.bytes().filter(|b| b.is_ascii()).count() as f64 / text.len().max(1) as f64;
    let special_char_ratio = text.bytes().filter(|b| !b.is_ascii_alphanumeric() && !b.is_ascii_whitespace()).count() as f64 / text.len().max(1) as f64;

    let chars_per_token = if ascii_ratio > 0.9 {
        if special_char_ratio > 0.2 { 3.5 } else { 4.0 }  // Code vs. prose
    } else if ascii_ratio > 0.5 {
        3.0  // Mixed content
    } else {
        2.0  // Heavy Unicode
    };

    (text.len() as f64 / chars_per_token).ceil() as u32
}
```

### 4.3 Semantic Message Importance Scoring

**New file:** `crates/tau-agent-core/src/context_ranking.rs`

Score messages for importance during compaction:

```rust
#[derive(Debug, Clone, Copy)]
pub struct MessageImportance {
    pub score: f64,  // 0.0 (droppable) to 1.0 (critical)
    pub reason: ImportanceReason,
}

#[derive(Debug, Clone, Copy)]
pub enum ImportanceReason {
    SystemMessage,         // Always critical (1.0)
    RecentTurn,           // Recency bonus (0.8-1.0)
    ContainsDecision,     // Keywords: "decided", "chose", "plan" (0.7)
    ContainsError,        // Contains error info user needs (0.7)
    ToolResult,           // Tool output — often bulky, lower priority (0.3-0.5)
    ConversationalFiller, // Low information density (0.1)
}

pub fn score_message_importance(
    message: &Message,
    position: usize,
    total_messages: usize,
) -> MessageImportance {
    let mut score = 0.5;  // Baseline

    // Recency: last 30% of messages get a bonus
    let recency_ratio = position as f64 / total_messages.max(1) as f64;
    if recency_ratio > 0.7 {
        score += 0.3 * (recency_ratio - 0.7) / 0.3;
    }

    // System messages are always critical
    if message.role == MessageRole::System {
        return MessageImportance { score: 1.0, reason: ImportanceReason::SystemMessage };
    }

    // Tool results are often bulky — lower base priority
    if message.role == MessageRole::Tool {
        score = 0.3;
        // But error results are important
        if message.is_error {
            score = 0.7;
        }
    }

    // Decision keywords boost importance
    let text = message.text_content();
    if contains_decision_keywords(&text) {
        score = (score + 0.2).min(1.0);
    }

    MessageImportance { score, reason: classify_reason(message, recency_ratio) }
}
```

**Integration into `compact_messages_for_tier`:** Sort messages by importance score, drop lowest-scored messages first instead of oldest-first. Preserve the `retain_percent` budget but choose which messages to retain based on score.

### 4.4 Hierarchical Summarization

**File:** `crates/tau-agent-core/src/runtime_turn_loop.rs`

When compacting, create layered summaries:

```rust
pub struct HierarchicalSummary {
    pub headline: String,       // 1-sentence overview (max 200 chars)
    pub key_decisions: Vec<String>,  // Extracted decisions (max 5)
    pub tool_trace: String,     // Compact tool execution trace (max 500 chars)
    pub full_summary: String,   // Current summary format (max 2000 chars)
}
```

**Modify summary generation** to produce structured summaries. The `headline` and `key_decisions` are always preserved; `tool_trace` and `full_summary` can be dropped in emergency compaction.

Increase `CONTEXT_SUMMARY_MAX_CHARS` from 2000 → 4000 for non-emergency tiers.

---

## Phase 5: Planning — Structured Plans with DAG Representation

**Goal:** Replace freeform text plans with structured, validatable task graphs.

### 5.1 Plan Data Model

**New file:** `crates/tau-orchestrator/src/plan.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredPlan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub depends_on: Vec<String>,      // Step IDs this depends on
    pub tools_required: Vec<String>,  // Tools needed for this step
    pub estimated_turns: usize,       // Estimated turns to complete
    pub status: PlanStepStatus,
    pub condition: Option<PlanCondition>,  // Optional conditional execution
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed { reason: String },
    Skipped { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanCondition {
    /// Execute only if the referenced step succeeded
    IfSucceeded(String),
    /// Execute only if the referenced step failed
    IfFailed(String),
    /// Always execute (default)
    Always,
}

impl StructuredPlan {
    /// Returns steps that are ready to execute (all dependencies satisfied)
    pub fn ready_steps(&self) -> Vec<&PlanStep> {
        self.steps.iter().filter(|step| {
            step.status == PlanStepStatus::Pending
                && step.depends_on.iter().all(|dep_id| {
                    self.steps.iter().any(|s| s.id == *dep_id && s.status == PlanStepStatus::Completed)
                })
                && step.condition_satisfied(self)
        }).collect()
    }

    /// Returns true if the plan is complete (all steps completed/skipped)
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| matches!(s.status, PlanStepStatus::Completed | PlanStepStatus::Skipped { .. }))
    }

    /// Validates plan DAG (no cycles, all dependencies exist, tools available)
    pub fn validate(&self, available_tools: &[String]) -> Result<(), PlanValidationError> { ... }

    /// Returns steps that can be executed in parallel
    pub fn parallelizable_groups(&self) -> Vec<Vec<&PlanStep>> { ... }
}
```

### 5.2 Plan Generation Prompt Engineering

**File:** `crates/tau-orchestrator/src/orchestrator.rs` (modify plan prompt ~line 700+)

Update the plan generation prompt to request JSON-structured output:

```rust
fn build_structured_plan_prompt(user_prompt: &str, available_tools: &[String]) -> String {
    format!(r#"
Analyze the following task and create a structured execution plan.

## Task
{user_prompt}

## Available Tools
{tools}

## Output Format
Respond with a JSON plan:
```json
{{
  "goal": "brief goal description",
  "steps": [
    {{
      "id": "step_1",
      "description": "What this step does",
      "depends_on": [],
      "tools_required": ["bash", "read"],
      "estimated_turns": 2,
      "condition": null
    }}
  ]
}}
```

Rules:
- Use `depends_on` to express ordering constraints
- Steps without dependencies can run in parallel
- Use `condition` for branching: {{"IfSucceeded": "step_id"}} or {{"IfFailed": "step_id"}}
- Be specific about which tools each step needs
- Keep steps atomic and testable
"#, tools = available_tools.join(", "))
}
```

### 5.3 Plan Validation & Execution Engine

**File:** `crates/tau-orchestrator/src/plan_executor.rs` (new)

```rust
pub struct PlanExecutor<R: OrchestratorRuntime> {
    plan: StructuredPlan,
    runtime: R,
    max_step_retries: usize,
}

impl<R: OrchestratorRuntime> PlanExecutor<R> {
    pub async fn execute(&mut self) -> Result<PlanExecutionReport> {
        // 1. Validate plan
        self.plan.validate(&self.available_tools())?;

        // 2. Execute in topological order
        while !self.plan.is_complete() {
            let ready = self.plan.ready_steps();
            if ready.is_empty() {
                // Deadlock: remaining steps have unsatisfied deps
                return Err(PlanError::Deadlock { remaining: self.pending_steps() });
            }

            // Execute parallelizable steps concurrently
            let groups = self.plan.parallelizable_groups();
            for group in groups {
                let results = futures::future::join_all(
                    group.iter().map(|step| self.execute_step(step))
                ).await;

                for (step, result) in group.iter().zip(results) {
                    self.plan.update_step_status(step.id, result);
                }
            }
        }

        Ok(self.build_report())
    }

    async fn execute_step(&mut self, step: &PlanStep) -> PlanStepStatus { ... }
}
```

### 5.4 Plan Revision on Failure

When a step fails, generate a revised plan:

```rust
pub async fn revise_plan_on_failure(
    runtime: &mut impl OrchestratorRuntime,
    plan: &mut StructuredPlan,
    failed_step: &PlanStep,
    error: &str,
) -> Result<()> {
    let revision_prompt = format!(
        "Step '{}' failed with error: {}\n\nCurrent plan state:\n{}\n\n\
         Revise the remaining plan steps to recover from this failure.",
        failed_step.description, error,
        serde_json::to_string_pretty(&plan)?
    );

    // Generate revised steps via LLM
    let revised = runtime.run_prompt_with_cancellation(&revision_prompt, ...).await?;

    // Parse and merge revised steps
    plan.merge_revision(revised)?;
    Ok(())
}
```

---

## Phase 6: Parallelism — Inter-Agent Communication & Dynamic Concurrency

**Goal:** Enable branch workers to share information, dynamically scale parallelism.

### 6.1 Inter-Agent Message Channel

**New file:** `crates/tau-agent-core/src/agent_channel.rs`

```rust
use tokio::sync::broadcast;

/// Shared message bus for inter-agent communication within a session
pub struct AgentMessageBus {
    sender: broadcast::Sender<AgentMessage>,
    capacity: usize,
}

#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub from_agent_id: String,
    pub to_agent_id: Option<String>,  // None = broadcast
    pub message_type: AgentMessageType,
    pub payload: Value,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone)]
pub enum AgentMessageType {
    /// Share a discovery (file found, error identified)
    Discovery,
    /// Signal completion of a sub-task
    StepCompleted,
    /// Request help or coordination
    CoordinationRequest,
    /// Share partial results
    PartialResult,
}

impl AgentMessageBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, capacity }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentMessage> {
        self.sender.subscribe()
    }

    pub fn send(&self, message: AgentMessage) -> Result<(), AgentMessage> {
        self.sender.send(message).map(|_| ()).map_err(|e| e.0)
    }
}
```

**Integration:**
- Add `message_bus: Arc<AgentMessageBus>` to `AgentRuntime` (lib.rs ~line 395)
- Pass bus reference when spawning branch workers (lib.rs ~line 1934+)
- Add `receive_messages` tool for agents to poll the bus
- Add `send_message` tool for agents to post to the bus

### 6.2 Dynamic Concurrency Scaling

**File:** `crates/tau-agent-core/src/lib.rs` (modify branch execution)

```rust
pub struct DynamicConcurrencyController {
    min_concurrent: usize,   // Default: 1
    max_concurrent: usize,   // From config.max_concurrent_branches_per_session
    current: AtomicUsize,
    active_branches: AtomicUsize,

    // Scaling signals
    recent_branch_latencies_ms: Mutex<VecDeque<u64>>,
    recent_branch_failures: AtomicU32,
}

impl DynamicConcurrencyController {
    /// Returns how many branches can be spawned right now
    pub fn available_slots(&self) -> usize {
        let effective_max = self.compute_effective_max();
        effective_max.saturating_sub(self.active_branches.load(Ordering::Relaxed))
    }

    fn compute_effective_max(&self) -> usize {
        let latencies = self.recent_branch_latencies_ms.lock().unwrap();
        let failures = self.recent_branch_failures.load(Ordering::Relaxed);

        // Scale down if branches are failing
        if failures > 2 {
            return self.min_concurrent;
        }

        // Scale up if branches are completing fast
        if let Some(avg_latency) = latencies.iter().copied().sum::<u64>().checked_div(latencies.len() as u64) {
            if avg_latency < 5_000 { self.max_concurrent }
            else if avg_latency < 15_000 { self.max_concurrent / 2 + 1 }
            else { self.min_concurrent }
        } else {
            self.max_concurrent / 2  // Conservative default
        }
    }
}
```

### 6.3 Branch Result Sharing

**File:** `crates/tau-agent-core/src/lib.rs` (modify branch result handling)

Currently branch results are opaque text summaries. Enhance to include structured data:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchResult {
    pub branch_id: String,
    pub status: BranchStatus,
    pub text_summary: String,
    pub tool_trace: Vec<ToolTraceEntry>,
    pub discoveries: Vec<AgentMessage>,  // Messages posted to bus
    pub artifacts: HashMap<String, Value>,  // Structured outputs
    pub metrics: BranchMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMetrics {
    pub turns_used: usize,
    pub tools_executed: usize,
    pub duration_ms: u64,
}
```

---

## Phase 7: Learning — Persistent Action History & Feedback Loops

**Goal:** Enable the agent to learn from past actions within and across sessions.

### 7.1 Action History Store

**New file:** `crates/tau-memory/src/action_history.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub session_id: String,
    pub turn: usize,
    pub action_type: ActionType,
    pub tool_name: Option<String>,
    pub input_summary: String,    // Truncated input (max 500 chars)
    pub output_summary: String,   // Truncated output (max 500 chars)
    pub success: bool,
    pub latency_ms: u64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    ToolExecution,
    LlmRequest,
    Replan,
    BranchSpawn,
    ContextCompaction,
}

pub struct ActionHistoryStore {
    store_path: PathBuf,
    max_records_per_session: usize,  // Default: 500
    max_total_records: usize,        // Default: 10_000
}

impl ActionHistoryStore {
    /// Append an action record
    pub async fn record(&self, action: ActionRecord) -> Result<()> { ... }

    /// Query actions by tool name, success/failure, time range
    pub async fn query(&self, filter: ActionFilter) -> Result<Vec<ActionRecord>> { ... }

    /// Get tool success rates across recent sessions
    pub async fn tool_success_rates(&self, lookback_sessions: usize) -> HashMap<String, f64> { ... }

    /// Get common failure patterns
    pub async fn failure_patterns(&self, lookback_sessions: usize) -> Vec<FailurePattern> { ... }
}
```

### 7.2 Feedback Loop Integration

**File:** `crates/tau-agent-core/src/lib.rs`

Add feedback collection at session end:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFeedback {
    pub session_id: String,
    pub outcome: SessionOutcome,
    pub user_satisfaction: Option<UserSatisfaction>,
    pub tool_effectiveness: HashMap<String, ToolEffectiveness>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionOutcome {
    Completed,
    Abandoned,     // User stopped mid-session
    Failed,        // Agent couldn't complete task
    PartialSuccess,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UserSatisfaction {
    Positive,
    Neutral,
    Negative,
}
```

**Integrate into session lifecycle:**
- At session end, auto-generate `SessionFeedback` from metrics
- Store in action history
- Load recent feedback at session start to adjust behavior

### 7.3 Cross-Session Knowledge Enhancement

**File:** `crates/tau-agent-core/src/cortex_runtime.rs` (enhance Cortex)

Extend Cortex bulletins to include learned patterns:

```rust
pub struct EnhancedCortexBulletin {
    pub memory_summary: String,          // Existing behavior
    pub tool_recommendations: Vec<ToolRecommendation>,  // New
    pub known_pitfalls: Vec<String>,     // New
    pub preferred_patterns: Vec<String>, // New
}

pub struct ToolRecommendation {
    pub tool_name: String,
    pub historical_success_rate: f64,
    pub avg_latency_ms: u64,
    pub recommendation: String,  // "Prefer for file operations" etc.
}
```

**Integration:** When building the Cortex bulletin, also query `ActionHistoryStore.tool_success_rates()` and `failure_patterns()` to generate actionable recommendations.

---

## Phase 8: Self-Repair — General Failure Recovery

**Goal:** Move beyond tool-specific self-repair to general agent failure recovery.

### 8.1 Failure Detection Framework

**New file:** `crates/tau-agent-core/src/failure_detector.rs`

```rust
#[derive(Debug, Clone)]
pub enum FailureSignal {
    /// Tool keeps failing with same error
    RepeatedToolFailure { tool: String, count: usize, error: String },
    /// Agent is looping (similar messages repeated)
    ConversationLoop { similarity: f64, loop_length: usize },
    /// Context exhaustion imminent
    ContextExhaustion { utilization: f64 },
    /// No progress (N turns without meaningful tool output)
    NoProgress { turns_without_progress: usize },
    /// Budget approaching limit
    BudgetExhaustion { utilization: f64 },
}

pub struct FailureDetector {
    config: FailureDetectorConfig,
}

pub struct FailureDetectorConfig {
    pub repeated_failure_threshold: usize,   // Default: 3
    pub loop_similarity_threshold: f64,      // Default: 0.85
    pub no_progress_turn_limit: usize,       // Default: 3
    pub budget_warning_threshold: f64,       // Default: 0.9
}

impl FailureDetector {
    /// Analyze recent turn history for failure signals
    pub fn detect(&self, history: &[Message], metrics: &AgentMetrics) -> Vec<FailureSignal> { ... }

    /// Detect conversation loops by comparing recent assistant messages
    fn detect_loops(&self, messages: &[Message]) -> Option<FailureSignal> {
        // Compare last N assistant messages for high similarity
        // using simple Jaccard similarity on word sets
        ...
    }
}
```

### 8.2 Recovery Strategies

**New file:** `crates/tau-agent-core/src/recovery.rs`

```rust
pub enum RecoveryStrategy {
    /// Retry with different approach (inject hint into context)
    RetryWithHint { hint: String },
    /// Abandon current path, try alternative
    AlternativeApproach { new_prompt: String },
    /// Compact context aggressively and retry
    CompactAndRetry,
    /// Escalate to user
    EscalateToUser { message: String },
    /// Terminate gracefully with partial results
    GracefulTermination { summary: String },
}

pub fn select_recovery_strategy(
    signal: &FailureSignal,
    attempt: usize,
    config: &AgentConfig,
) -> RecoveryStrategy {
    match signal {
        FailureSignal::RepeatedToolFailure { tool, error, .. } => {
            if attempt == 0 {
                RecoveryStrategy::RetryWithHint {
                    hint: format!("Tool '{}' failed with: {}. Try an alternative approach.", tool, error),
                }
            } else {
                RecoveryStrategy::EscalateToUser {
                    message: format!("Tool '{}' is repeatedly failing. Need guidance.", tool),
                }
            }
        }
        FailureSignal::ConversationLoop { .. } => {
            RecoveryStrategy::AlternativeApproach {
                new_prompt: "You appear to be repeating the same actions. Step back, reassess the problem, and try a fundamentally different approach.".to_string(),
            }
        }
        FailureSignal::ContextExhaustion { .. } => RecoveryStrategy::CompactAndRetry,
        FailureSignal::NoProgress { .. } => {
            if attempt == 0 {
                RecoveryStrategy::RetryWithHint {
                    hint: "No progress detected. Break the problem into smaller steps and verify each step.".to_string(),
                }
            } else {
                RecoveryStrategy::EscalateToUser {
                    message: "Unable to make progress on this task. Would you like to provide more guidance?".to_string(),
                }
            }
        }
        FailureSignal::BudgetExhaustion { .. } => {
            RecoveryStrategy::GracefulTermination {
                summary: "Approaching budget limit. Here is what has been accomplished so far.".to_string(),
            }
        }
    }
}
```

### 8.3 Integration into Turn Loop

**File:** `crates/tau-agent-core/src/lib.rs` (modify turn loop ~line 1700+)

Add failure detection check at end of each turn:

```rust
// After each turn completes:
let signals = self.failure_detector.detect(&self.messages, &self.metrics);
for signal in signals {
    let strategy = select_recovery_strategy(&signal, self.recovery_attempts, &self.config);
    match strategy {
        RecoveryStrategy::RetryWithHint { hint } => {
            self.inject_system_hint(&hint);
            self.recovery_attempts += 1;
        }
        RecoveryStrategy::CompactAndRetry => {
            self.compact_context(ContextCompactionTier::Aggressive)?;
        }
        RecoveryStrategy::EscalateToUser { message } => {
            self.emit_event(AgentEvent::EscalationRequired { message });
            break;
        }
        RecoveryStrategy::GracefulTermination { summary } => {
            self.emit_event(AgentEvent::GracefulTermination { summary });
            return Ok(());
        }
        _ => {}
    }
}
```

---

## Phase 9: Safety — Hard Limits, PII Redaction & Contextual Analysis

**Goal:** Strengthen safety guardrails with hard enforcement and smarter detection.

### 9.1 Hard Token/Cost Budget Enforcement

**File:** `crates/tau-agent-core/src/lib.rs` (modify cost tracking)

Currently cost budgets are advisory. Add hard enforcement:

```rust
pub struct AgentConfig {
    // ... existing ...
    pub cost_budget_hard_limit: bool,  // Default: false (backwards compatible)
}
```

**In the turn loop**, before each LLM call:
```rust
if self.config.cost_budget_hard_limit {
    if let Some(budget) = self.config.cost_budget_usd {
        let current_cost = self.cost_snapshot().estimated_cost_usd;
        if current_cost >= budget {
            return Err(AgentError::BudgetExceeded {
                budget_usd: budget,
                spent_usd: current_cost,
            });
        }
    }
}
```

### 9.2 PII Detection and Redaction

**File:** `crates/tau-safety/src/lib.rs` (extend safety rules)

```rust
pub struct PiiDetector {
    patterns: Vec<PiiPattern>,
}

pub struct PiiPattern {
    pub category: PiiCategory,
    pub regex: Regex,
    pub redaction_token: String,
}

pub enum PiiCategory {
    Email,
    PhoneNumber,
    SocialSecurityNumber,
    CreditCard,
    IpAddress,
    StreetAddress,
}

impl PiiDetector {
    pub fn default_patterns() -> Self {
        Self {
            patterns: vec![
                PiiPattern {
                    category: PiiCategory::Email,
                    regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
                    redaction_token: "[EMAIL_REDACTED]".to_string(),
                },
                PiiPattern {
                    category: PiiCategory::PhoneNumber,
                    regex: Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap(),
                    redaction_token: "[PHONE_REDACTED]".to_string(),
                },
                PiiPattern {
                    category: PiiCategory::SocialSecurityNumber,
                    regex: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
                    redaction_token: "[SSN_REDACTED]".to_string(),
                },
                PiiPattern {
                    category: PiiCategory::CreditCard,
                    regex: Regex::new(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b").unwrap(),
                    redaction_token: "[CC_REDACTED]".to_string(),
                },
                // ... more patterns
            ],
        }
    }

    pub fn scan_and_redact(&self, text: &str) -> (String, Vec<PiiDetection>) { ... }
}
```

**Integration:**
- Add `PiiDetector` to `DefaultSanitizer`
- Add `pii_redaction_enabled: bool` to `AgentConfig` (default: false)
- When enabled, scan all outbound messages for PII

### 9.3 Contextual Prompt Injection Detection

**File:** `crates/tau-safety/src/lib.rs`

Replace pure substring matching with context-aware scoring:

```rust
pub struct ContextualInjectionDetector {
    literal_rules: Vec<SafetyRule>,      // Existing rules
    scoring_rules: Vec<ScoringRule>,      // New: weighted scoring
    threshold: f64,                       // Default: 0.7
}

pub struct ScoringRule {
    pub pattern: Regex,
    pub weight: f64,
    pub description: String,
}

impl ContextualInjectionDetector {
    pub fn score_input(&self, text: &str, context: &InjectionContext) -> f64 {
        let mut score = 0.0;

        // Apply literal rules (existing behavior)
        for rule in &self.literal_rules {
            if rule.matches(text) {
                score += 0.5;  // High weight for known patterns
            }
        }

        // Apply scoring rules with context
        for rule in &self.scoring_rules {
            if rule.pattern.is_match(text) {
                // Reduce weight if content is in a code block or documentation context
                let weight = if context.is_code_context {
                    rule.weight * 0.3  // Code discussing injection is less suspicious
                } else {
                    rule.weight
                };
                score += weight;
            }
        }

        score.min(1.0)
    }
}

pub struct InjectionContext {
    pub is_code_context: bool,       // Inside code block or file content
    pub is_documentation: bool,      // Part of docs/readme
    pub source: ContentSource,       // User input vs. tool output vs. file content
}
```

### 9.4 Safety Audit Logging

**New file:** `crates/tau-safety/src/audit.rs`

```rust
pub struct SafetyAuditLog {
    log_path: PathBuf,
    max_entries: usize,  // Default: 10_000
}

#[derive(Debug, Serialize)]
pub struct SafetyAuditEntry {
    pub timestamp_ms: u64,
    pub session_id: String,
    pub stage: SafetyStage,
    pub action: SafetyAction,
    pub rule_matched: Option<String>,
    pub content_snippet: String,  // First 200 chars of matched content
    pub severity: SafetySeverity,
}

#[derive(Debug, Serialize)]
pub enum SafetyAction {
    Allowed,
    Warned,
    Redacted,
    Blocked,
}

#[derive(Debug, Serialize)]
pub enum SafetySeverity {
    Info,
    Warning,
    Critical,
}
```

---

## Cross-Cutting Concerns

### Testing Strategy

Each phase includes:
1. **Unit tests** — Per-function tests for new logic (same file `#[cfg(test)]` modules)
2. **Integration tests** — Cross-crate tests in `tests/integration/`
3. **Property-based tests** — For circuit breaker state machine, plan DAG validation, token estimation
4. **Backward compatibility tests** — Ensure existing behavior with default configs is unchanged

### New `AgentConfig` Fields Summary

All new config fields with defaults that preserve existing behavior:

```rust
// Phase 2: Resilience
pub circuit_breaker_failure_threshold: u32,       // 5
pub circuit_breaker_recovery_timeout_ms: u64,     // 30_000
pub fallback_models: Vec<String>,                 // []

// Phase 3: Tool Intelligence
pub tool_retry_max_attempts: usize,               // 1
pub tool_retry_backoff_ms: u64,                   // 500
pub tool_retry_on_timeout: bool,                  // true
pub tool_health_window_size: usize,               // 20
pub react_replan_failure_ratio_threshold: f64,    // 0.5
pub react_partial_failure_replan: bool,           // true

// Phase 4: Context
pub context_compaction_predictive: bool,          // false (opt-in)
pub context_summary_max_chars: usize,             // 4000

// Phase 7: Learning
pub action_history_enabled: bool,                 // false (opt-in)
pub action_history_max_records: usize,            // 500

// Phase 8: Self-Repair
pub failure_detection_enabled: bool,              // true
pub failure_repeated_threshold: usize,            // 3
pub failure_no_progress_turns: usize,             // 3

// Phase 9: Safety
pub cost_budget_hard_limit: bool,                 // false
pub pii_redaction_enabled: bool,                  // false
pub safety_audit_log_enabled: bool,               // false
```

### Dependency Changes

| Crate | New Dependencies |
|-------|-----------------|
| `tau-agent-core` | `dashmap` (lock-free map), `rand` (jitter) |
| `tau-safety` | `regex` (already present), no new deps |
| `tau-orchestrator` | No new deps |
| `tau-memory` | No new deps |

### File Change Summary

| Phase | New Files | Modified Files |
|-------|-----------|----------------|
| 1 | `tau-agent-core/src/metrics.rs` | `tau-agent-core/src/lib.rs`, `tau-events/src/lib.rs` |
| 2 | `tau-agent-core/src/circuit_breaker.rs` | `tau-agent-core/src/lib.rs`, `runtime_turn_loop.rs` |
| 3 | — | `tau-agent-core/src/lib.rs`, `runtime_tool_bridge.rs` |
| 4 | `tau-agent-core/src/context_ranking.rs` | `runtime_turn_loop.rs` |
| 5 | `tau-orchestrator/src/plan.rs`, `plan_executor.rs` | `tau-orchestrator/src/orchestrator.rs` |
| 6 | `tau-agent-core/src/agent_channel.rs` | `tau-agent-core/src/lib.rs` |
| 7 | `tau-memory/src/action_history.rs` | `tau-agent-core/src/lib.rs`, `cortex_runtime.rs` |
| 8 | `tau-agent-core/src/failure_detector.rs`, `recovery.rs` | `tau-agent-core/src/lib.rs` |
| 9 | `tau-safety/src/audit.rs` | `tau-safety/src/lib.rs`, `tau-agent-core/src/lib.rs` |

---

## Migration & Rollout Strategy

### Ordering

Phases should be implemented in order (1→9) because:
- **Phase 1** provides the metrics foundation all later phases depend on
- **Phase 2** provides the error classification Phase 3 uses
- **Phase 3** provides health tracking Phase 8 consumes
- **Phase 4** is independent but benefits from Phase 1 metrics
- **Phase 5** is independent (orchestrator layer)
- **Phase 6** depends on Phase 5 for plan-based parallelization
- **Phase 7** depends on Phase 1 metrics and Phase 3 health tracking
- **Phase 8** depends on Phase 1, 3, and 7
- **Phase 9** is mostly independent

**Parallelizable groups:**
- Phases 4, 5, and 9 can be developed in parallel with Phase 2–3
- Phase 6 can start once Phase 5 is complete

### Feature Flags

All new behaviors are behind feature flags (config booleans) defaulting to OFF or existing behavior, ensuring:
- Zero breaking changes on upgrade
- Gradual opt-in per deployment
- Easy rollback by toggling config

### Backward Compatibility

- All new `AgentConfig` fields have defaults matching current behavior
- No existing public API signatures change
- No existing behavior changes unless new config flags are explicitly enabled
- New modules are added alongside existing ones, not replacing them
