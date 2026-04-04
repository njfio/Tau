---
title: "Deepened Plan: State-of-the-Art Autonomous Agent Transformation"
type: feat
status: active
date: 2026-04-03
origin: docs/plans/2026-04-03-001-feat-tranche-one-autonomy-vertical-slice-plan.md
deepened_from:
  - docs/plans/2026-04-03-001-feat-tranche-one-autonomy-vertical-slice-plan.md
  - docs/plans/2026-03-24-001-feat-radical-simplification-self-improvement-plan.md
  - docs/plans/2026-03-24-002-feat-wiring-integration-endgame-plan.md
research_agents: 9
sections_enhanced: 12
---

# Deepened Plan: State-of-the-Art Autonomous Agent Transformation

## Enhancement Summary

**Deepened on:** 2026-04-03
**Research agents used:** 9 (state-of-the-art agents, Rust architecture, security, architecture strategy, performance, simplicity, agent-native, codebase exploration, repo research)
**Plans analyzed:** 3 (tranche-one autonomy, radical simplification, wiring integration)

### Key Improvements Discovered

1. **Radical scope reduction**: Cut 60% of proposed work. The three plans together propose 7 features + 37 wiring items + a self-modification engine. The actual critical path is 8 items.
2. **Critical security vulnerability**: The agent can modify its own safety policy via gateway API (P0 fix).
3. **Production stability blockers**: Codex subprocess zombie leak, fake streaming, synchronous I/O on hot path.
4. **Missing architectural primitive**: Missions cannot spawn sub-missions -- the single largest gap for complex autonomy.
5. **Industry convergence**: Every major agent (Claude Code, Devin, Codex) shipped multi-agent orchestration in Feb 2026. Tau's orchestrator exists but is disconnected from the gateway mission model.

### The Uncomfortable Truth

> "A plan titled 'Radical Simplification' that proposes 7 features, 37 wiring items, a self-modification engine, 33 MCP tools, and rewriting 126 shell scripts in Rust is not simplification. It is a second system. The actual radical simplification is: define 3 tasks, try to complete them, fix what breaks."
> -- Simplicity Review Agent

---

## Part 1: What Tau Needs to Become State-of-the-Art

### 1.1 Current State Assessment

**Strengths (genuinely ahead of most agents):**
- Ralph-loop verifier model with evidence-backed continuation is novel and effective
- `complete_task` tool with `success`/`partial`/`blocked` gives agents first-class checkpoint agency
- Circuit breaker + failure detector + recovery strategy system is best-in-class for Rust
- Action history infrastructure (failure patterns, tool effectiveness) is ahead of its consumers
- Read-only saturation detection prevents wasted compute on stuck loops
- Exceptionally well-governed: AGENTS.md + 882-line CI + 11-tier testing contract
- Ed25519 skill signing, AES-256-GCM credentials, dedicated safety crate

**Critical Gaps (blocking state-of-the-art):**

| Gap | Impact | Current State |
|-----|--------|---------------|
| No mission composition | Cannot decompose complex tasks | Orchestrator exists but disconnected from gateway |
| No agent self-recovery | Blocked missions require human resume | TUI has `/resume`, no agent equivalent |
| 20/33 MCP tools are stubs | External agents hit dead ends | Returns `not_yet_implemented` JSON |
| Passive metacognition only | Agent cannot query its own performance | Learning injected into system prompt, no active tools |
| Codex subprocess zombies | Resource leak on every timeout | `kill_on_drop` doesn't kill process group |
| Fake streaming | Zero progress for 1-5 min, then dump | `complete_with_stream` buffers entire response |
| Synchronous I/O on hot path | JSONL loaded/saved on every Ralph-loop iteration | Full file read-parse-write per attempt |
| Safety policy mutable by agent | Agent can disable its own safety | PUT endpoint accepts agent's bearer token |
| No formal mission state machine | State transitions spread across 3 files | Implied, not enforced |
| No context budget management | Long missions silently degrade | System prompt + learning + verifier observations grow unbounded |

### 1.2 Industry Context (2025-2026)

**The harness, not scaffold, paradigm:** The model is now good enough to orchestrate itself. The runtime provides tools, safety, context management, and persistence. Tau's gateway should be a thin harness, not a rigid pipeline.

**Multi-agent is table stakes:** Every major tool shipped multi-agent in Feb 2026 (Grok Build 8 agents, Windsurf 5 parallel, Claude Code Agent Teams, Codex Agents SDK). Tau has `tau-orchestrator` but it's disconnected.

**Context engineering > prompt engineering:** JetBrains Research (Dec 2025) found context management is the single most impactful factor. Simple observation masking (keep M most recent) equals or beats complex LLM summarization.

**Ground truth > LLM opinion:** Anthropic spent more time on tool interfaces than prompts for SWE-bench. Use compiler/test output, never LLM self-evaluation.

**Five-layer defense-in-depth:** Prompt-level, schema-level, runtime approval, tool-level validation, user hooks. Tau has layers 1, 3, and 5; layers 2 and 4 are gaps.

---

## Part 2: The Revised Critical Path

### Phase 0: Production Stability (Do First -- Blocks Everything)

These are bugs, not features. They must be fixed before any autonomy work.

#### P0-1: Fix Codex Subprocess Process-Group Lifecycle

**Problem:** `kill_on_drop(true)` only kills the direct child PID. Codex spawns its own workers. Timeouts leave zombie process trees consuming 200MB+ each. With retries, a single mission can spawn 3 zombie trees.

**Fix:**
```rust
// In codex_cli_client.rs
use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use std::os::unix::process::CommandExt;

// Before spawn: put child into its own process group
unsafe {
    command.pre_exec(|| {
        nix::unistd::setpgid(Pid::from_raw(0), Pid::from_raw(0))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    });
}
command.kill_on_drop(false); // We handle it ourselves

// On timeout: kill the entire process group
let child_pid = child.id().expect("child should have pid");
let _ = killpg(Pid::from_raw(child_pid as i32), Signal::SIGTERM);
tokio::time::sleep(Duration::from_millis(500)).await;
let _ = killpg(Pid::from_raw(child_pid as i32), Signal::SIGKILL);
```

**Files:** `crates/tau-provider/src/codex_cli_client.rs` (lines 132-217)

#### P0-2: Implement Real Streaming from Codex stdout

**Problem:** `complete_with_stream` calls `self.complete(request).await` then delivers the entire response as one delta. TUI shows zero progress for 1-5 minutes.

**Fix:** Stream codex stdout line-by-line using `BufReader::lines()` during execution instead of buffering with `wait_with_output()`.

**Files:** `crates/tau-provider/src/codex_cli_client.rs` (lines 203-217)

#### P0-3: Immutable Safety Floor

**Problem (CRITICAL SECURITY):** `handle_gateway_safety_policy_put` and `handle_gateway_safety_rules_put` allow modifying safety policy via authenticated HTTP. The agent's own bearer token works. A jailbroken agent can disable all safety scanning.

**Fix:**
- Hardcode an immutable safety floor that the PUT endpoint cannot weaken
- Require a separate operator confirmation token for safety mutations
- Add `safety_policy_locked` flag requiring restart to unlock
- Log all safety mutations to an append-only audit trail

**Files:** `crates/tau-gateway/src/gateway_openresponses/safety_runtime.rs` (lines 39-117, 153-216)

#### P0-4: Hoist Action History I/O Out of Per-Attempt Loop

**Problem:** The Ralph-loop loads the entire JSONL action history, deserializes every record, appends, then rewrites the entire file -- on every attempt iteration. With 1000 records and 3 retries, that's 6000 parse/serialize cycles.

**Fix:** Load once at request start, pass as `&mut` through the loop, save once at the end. Move trace persistence to `tokio::task::spawn_blocking`.

**Files:**
- `crates/tau-gateway/src/gateway_openresponses/learning_runtime.rs` (line 67)
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs` (lines 357-675)

### Research Insights for Phase 0

**Performance:**
- Add `tracing::instrument` spans to critical path: `execute_openresponses_request`, each Ralph-loop iteration, action history load/save, codex subprocess time
- Cache compiled regexes in safety scanner (`Regex::new()` called inside per-rule loop on every invocation)
- Cache MCP `tools/list` response (rebuilds 33+ tool descriptors on every call)
- Consider `simd-json` for JSONL hot paths (2-4x speedup, drop-in replacement)

**Security:**
- Replace SHA-256 with Argon2id for credential store key derivation (`credential_store.rs` lines 259-273)
- Remove legacy XOR stream cipher path (`decrypt_credential_store_secret_legacy`)
- Call `env_clear()` on codex subprocess (currently inherits full parent environment including secrets)
- Pass action history `input_summary`/`output_summary` through leak detector before JSONL persistence

---

### Phase 1: Prove Autonomy Exists (The Benchmark)

This is the single most important deliverable. Everything else is secondary.

#### 1.1: Define 3 Benchmark Tasks

Create a markdown file defining 3 tasks with explicit pass/fail criteria. Not a framework, not a harness -- a document.

**Task selection criteria (from state-of-the-art research):**
- At least one single-file fix (calibration baseline)
- At least one multi-file feature (the real test)
- At least one cross-crate change (stress test)
- All tasks should have verifiable outcomes (tests pass, clippy clean, spec conformance)

**Benchmark metrics (from industry best practices):**
| Metric | Why |
|--------|-----|
| Pass/fail (tests + clippy) | Strongest signal -- binary, unambiguous |
| Time to completion | Efficiency signal |
| Token cost | Cost efficiency |
| Retry count | Recovery effectiveness |
| Human interventions | Autonomy measurement |

**Files:** Create `docs/benchmarks/tranche-one-benchmark.md` and `tasks/fixtures/tranche-one/`

#### 1.2: Mission Result Classification

Add a simple enum to the gateway mission model:

```rust
pub enum BenchmarkResult {
    Completed { artifacts: Vec<PathBuf>, duration: Duration },
    CheckpointRequired { reason: String, next_action: String },
    Blocked { reason: String },
    RuntimeFailed { error: String },
}
```

Derive from mission/verifier/completion state, not brittle string matching.

**Files:**
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`

#### 1.3: Run the Benchmark. Fix What Breaks.

This is the actual work. Run each benchmark task through the system as-is. Document every failure. The failures become the roadmap.

### Research Insights for Phase 1

**State-of-the-art benchmarks:**
- SWE-bench Verified (500 tasks, Python) is the standard but Tau targets Rust
- FeatureBench (2025) better represents real feature development work
- Context-Bench (Letta, Oct 2025) tests multi-step consistency -- directly relevant
- Design three tiers: conformance (every PR), capability (weekly), realism (monthly)

**Ground-truth verification (Anthropic's key finding):**
- Auto-run `cargo check` after every file edit, feed errors back into loop
- Auto-run `cargo test -p <crate>` after implementation, feed failures as structured critique
- Never rely on LLM self-evaluation for code quality

**Doom-loop prevention:**
- After 3 identical failures, escalate to different approach or human
- AGENTS.md already specifies this as process convention; enforce at runtime level
- Tau's read-only saturation detection is a smart pattern -- generalize it

---

### Phase 2: Enable Composition (The Missing Architectural Primitive)

#### 2.1: Formal Mission State Machine

**Problem:** Mission lifecycle is implied across `mission_supervisor_runtime.rs`, `mission_completion_runtime.rs`, and `openresponses_execution_handler.rs`. No compile-time transition validation.

**Fix:** Create `mission_state_machine.rs` with explicit enum and consuming transition methods:

```rust
pub enum MissionPhase {
    Planning { objectives: Vec<String>, started_at: DateTime<Utc> },
    Executing { plan: MissionPlan, current_step: usize, checkpoint: Option<Checkpoint> },
    Blocked { reason: BlockReason, blocked_since: DateTime<Utc>, resume_hint: Option<String> },
    Checkpointing { snapshot: MissionSnapshot, prior_phase: Box<MissionPhase> },
    Completed { summary: String, artifacts: Vec<PathBuf> },
    Failed { error: MissionError, partial_results: Vec<String> },
}

impl MissionPhase {
    pub fn begin_execution(self, plan: MissionPlan) -> Result<Self, MissionTransitionError> {
        match self {
            MissionPhase::Planning { .. } => Ok(MissionPhase::Executing {
                plan,
                current_step: 0,
                checkpoint: None,
            }),
            other => Err(MissionTransitionError::InvalidTransition {
                from: other.phase_name(),
                to: "Executing",
            }),
        }
    }
}
```

**Research insight:** Hand-rolled enum is best for Tau because (a) mission state must be serializable for checkpointing, (b) transitions depend on runtime conditions, (c) you already use this pattern implicitly.

#### 2.2: Mission Composition (Parent/Child Missions)

**Problem:** Complex tasks require decomposition. Currently the entire spec-to-PR flow runs as one flat mission with one retry budget. Step 3 failure restarts from step 1.

**Fix:** Add `parent_mission_id: Option<String>` to `GatewayMissionState`. Wire `agent_spawn` MCP tool to create child missions inheriting parent session context.

**Research insight:** Every major agent shipped multi-agent in Feb 2026. The orchestrator-workers pattern with dynamic spawning is the convergence point. Tau's `tau-orchestrator` has DAG-based plan decomposition but is disconnected from the gateway.

#### 2.3: Agent Self-Recovery

**Problem:** Blocked missions require human `/resume`. The agent cannot observe its own blocked state and retry with adjusted strategy.

**Fix:** Expose `resume_mission` gateway endpoint callable by agents. Allow resume with adjusted prompt addressing the block reason. `GatewayMissionState` already stores `latest_verifier` and `latest_completion` with block reasons.

#### 2.4: Adaptive Retry Budget

**Problem:** Fixed 2 retries (`ACTION_TOOL_EVIDENCE_MAX_RETRIES = 2`) regardless of task complexity or failure type.

**Fix:** Read `ActionHistoryStore` failure patterns at retry decision time. If history shows a known workaround for the failure pattern, inject it. If similar tasks consistently need 3 retries, allow 3.

### Research Insights for Phase 2

**Rust patterns:**
- Replace `CooperativeCancellationToken` (hand-rolled `Arc<AtomicBool>`) with `tokio_util::sync::CancellationToken` for tree-structured cancellation of missions/branches
- Child token pattern: `let branch_token = mission_token.child_token();` -- auto-cancelled when mission cancelled
- Switch async event handlers from `std::sync::mpsc::sync_channel` to `tokio::sync::mpsc` for native `select!` integration
- Use `tokio::task::JoinSet` for dynamic subagent management

**Architecture:**
- Bridge orchestrator's plan decomposition into gateway mission model
- When verifier detects multi-step task, invoke orchestrator to create plan, execute each step as sub-mission
- Add `MissionPhase` sub-states within Ralph-loop for partial-progress persistence

**Agent-native parity gap:**

| Operator Action | Agent Self-Service | Fix |
|---|---|---|
| `/resume <id>` | No equivalent | Add `resume_mission` endpoint |
| `/missions` | `session_list` stub | Implement MCP handler |
| `/status` | `learn_status` stub | Implement MCP handler |
| Spawn sub-agent | `agent_spawn` stub | Connect to gateway with parent/child |
| Detach from mission | No equivalent | Add `detach_mission` endpoint |

---

### Phase 3: Activate Learning (Minimal Version)

#### 3.1: Wire Action History Persistence

**What:** Flip `action_history_enabled` to `true` (it already is in config defaults but never read). Wire `load()` at session start, `save()` at session end.

**One PR. Two function calls.**

**Files:**
- `crates/tau-agent-core/src/lib.rs` (verify config is read)
- Session startup path (call `ActionHistoryStore::load()`)
- `AgentEvent::AgentEnd` handler (call `ActionHistoryStore::save()`)

#### 3.2: Feed Failure Patterns into Recovery

**What:** Modify `select_recovery_strategy()` to accept optional `&ActionHistoryStore`. Query `failure_patterns(lookback=50)` and `tool_success_rates(lookback=50)`. Inject learned recovery strategies.

**Files:** `crates/tau-agent-core/src/recovery.rs`

#### 3.3: Implement 7 Priority MCP Tools (Not 20)

| Tool | Delegates To | Why Now |
|------|-------------|---------|
| `session_list` | SessionRuntime | Operators need mission visibility |
| `session_resume` | SessionRuntime | Enables agent self-recovery |
| `agent_spawn` | Gateway + Orchestrator | Enables mission composition |
| `agent_status` | Gateway | Required by agent_spawn |
| `agent_cancel` | Gateway | Cleanup for agent_spawn |
| `learn_status` | ActionHistoryStore | Active metacognition |
| `skills_list` | Skill catalog | Basic extensibility inspection |

The remaining 13 stubs should be hidden (not advertised in `tools/list`) until implemented. Advertising broken tools damages trust.

### Research Insights for Phase 3

**Context management (JetBrains Research finding):**
- Simple observation masking (keep M most recent) equals complex LLM summarization
- Implement progressive compaction: summarize -> entity extract -> truncate -> episodic compress -> prune
- Add token budget tracking to Ralph-loop: cumulative system prompt + learning + verifier observation tokens

**Dual-memory architecture:**
- Episodic memory: older observations compressed to summaries
- Working memory: recent interactions at full fidelity
- Implement in gateway session state, not as new infrastructure

**Event-driven system reminders:**
- Detect when codex has been running >60s, inject reminder about breaking work into smaller steps
- Combat instruction fade-out in long sessions with targeted guidance at decision points

---

### Phase 4: Simplify Surfaces (Only What's Needed)

#### 4.1: Declarative `.tau.toml` (Schema + Startup Wire)

**Minimum viable:** Define schema, wire into startup dispatch. Skip `tau init` variants and config doctor.

**Files:**
- `crates/tau-onboarding/src/config_file.rs` (parser exists)
- `crates/tau-onboarding/src/startup_dispatch.rs` (wire it)

#### 4.2: Script Cleanup (Not Rewrite)

**Do:** Delete archive duplicates still in non-archive directories. Move one-off scripts to `scripts/archive/`. Add `scripts/README.md` index.

**Do NOT:** Rewrite 126 shell scripts in Rust. They work. This is a day of cleanup, not weeks of rewriting.

#### 4.3: Skills Gain a `tools` Field (Minimal Unification)

**Do:** Add `tools: Option<Vec<SkillToolDefinition>>` to Skill struct. That's it.

**Do NOT:** Add hooks, runtime, commands, permissions in v1. The current skills manifest should not become a fourth, more complex extension mechanism.

---

## Part 3: What to Cut

### Cut Entirely

| Item | Reason |
|------|--------|
| Feature 7: Reflexive Source Patching | Cannot self-improve before proving basic autonomy. Eliminates 9 wiring items (M6-M8, B2-B5, B8, B10). |
| Feature 3: APO Auto-Trigger | No meaningful training data until benchmark passes consistently |
| Rewriting 126 scripts in Rust | Zero user-visible value. Organize, don't rewrite. |
| 13 low-priority MCP stubs | Hide until implemented. Advertising broken tools damages trust. |
| Feature 5 as proposed | Replace with simple cleanup PR |

### Defer Until After Benchmark Passes

| Item | Trigger |
|------|---------|
| Feature 6: Full MCP expansion (33+ tools) | After 7 priority tools prove the pattern |
| Feature 1 sub-tasks 1.4-1.5 (Cortex bulletin, operator commands) | After learning persistence proves useful |
| Feature 3 (LiveRlBridge APO) | After 50+ successful benchmark runs generate meaningful data |
| B1-B10 (all Bold items in wiring plan) | After tranche-one benchmark passes |
| Training pipeline consolidation (7->3 crates) | After training is productized |

---

## Part 4: Architecture Improvements

### 4.1: Crate Structure Recommendations

**Merge (reduce from 46 to ~39):**

| Action | Crates | Rationale |
|--------|--------|-----------|
| Merge into `tau-core` | `tau-contract`, `tau-session`, `tau-events` | Thin type/utility crates with no independent deployment |
| Delete | `tau-extensions`, `tau-custom-command` | Deprecated in AGENTS.md; `tau-coding-agent` still depends on both |
| Merge into `tau-training` | `tau-training-types`, `tau-training-store`, `tau-training-tracer` | Pre-production infrastructure doesn't need 6 crates |

**Split:**

| Action | Current | Into | Rationale |
|--------|---------|------|-----------|
| Split `tau-agent-core/lib.rs` | 3,831 lines | `agent.rs`, `config.rs`, `types.rs`, `tests/mocks.rs` | God object with test infra embedded in library |
| Split `openresponses_execution_handler.rs` | 1,756 lines + 17,527-line test file | `attempt_retry_runtime.rs`, `attempt_trace_runtime.rs`, `skill_prompt_runtime.rs` | Merge conflict magnet, comprehension bottleneck |
| Extract coding-agent runtime to library | `runtime_loop.rs` (2,327 lines), `live_rl_runtime.rs` (2,182 lines) in binary crate | `tau-coding-agent-lib` | Binary doing library work; not testable independently |

**Create:**

| Crate | Purpose |
|-------|---------|
| `tau-traits` (or repurpose `tau-contract`) | Central trait crate: `AgentTool`, `LlmClient`, `GatewayToolRegistrar`, `TrainingStore`. Currently scattered across implementation crates. |

### 4.2: Consolidate Facade Modules

13 single-function re-export modules in `tau-coding-agent/src/` (`auth_commands.rs`, `mcp_client.rs`, `mcp_server.rs`, `tools.rs`, `project_index.rs`, etc.) should be consolidated into one `facades.rs` or inlined into main.

### 4.3: Clean Up Dead Config Fields

10 `AgentConfig` fields are defined but never read anywhere in the codebase:

| Field | Phase | Action |
|-------|-------|--------|
| `action_history_enabled` | Phase 7 | Wire it (Phase 3.1 above) |
| `action_history_max_records` | Phase 7 | Wire it |
| `action_history_retention_days` | Phase 7 | Wire it |
| `failure_detection_enabled` | Phase 8 | Remove until implemented |
| `failure_repeated_threshold` | Phase 8 | Remove until implemented |
| `failure_no_progress_turns` | Phase 8 | Remove until implemented |
| `circuit_breaker_failure_threshold` | Phase 2 | Wire it (already partially used) |
| `circuit_breaker_recovery_timeout_ms` | Phase 2 | Wire it |
| `context_compaction_predictive` | Phase 4 | Remove until implemented |

---

## Part 5: Security Hardening Roadmap

### P0 (Do Immediately)

| Finding | Fix | Files |
|---------|-----|-------|
| C1: Safety policy mutable by agent | Immutable safety floor + separate operator token | `safety_runtime.rs` |
| C3: SHA-256 without KDF for credentials | Replace with Argon2id | `credential_store.rs` |
| H2: Codex inherits full environment | `env_clear()` + safelisted vars | `codex_cli_client.rs` |

### P1 (Do With Phase 0)

| Finding | Fix | Files |
|---------|-----|-------|
| C2: Self-mod checks path not content | Add diff content scanning + blocklist for Cargo.toml/build.rs | `tau-safety/src/lib.rs` |
| H4: Action history stores unredacted secrets | Pass through leak detector before JSONL persistence | `learning_runtime.rs` |
| H1: Legacy XOR cipher still callable | Re-encrypt to v2, remove v1 path | `credential_store.rs` |

### P2 (Do With Phase 2)

| Finding | Fix | Files |
|---------|-----|-------|
| H5: Training data poisoning | HMAC-sign rollout records + anomaly detection | `training_runtime.rs` |
| M2: Prompt injection bypassable via Unicode | NFKC normalization + strip zero-width chars | `tau-safety/src/lib.rs` |
| M4: WASM limits are manifest-declared | Enforce hard caps overriding manifest | `tau-extensions/src/lib.rs` |

---

## Part 6: Execution Order

```
Phase 0: Production Stability          [1-2 weeks]
  P0-1: Fix codex process-group kill
  P0-2: Implement real streaming
  P0-3: Immutable safety floor
  P0-4: Hoist action history I/O
  + P0 security fixes (credential KDF, env_clear)

Phase 1: Prove Autonomy               [1-2 weeks]
  1.1: Define 3 benchmark tasks
  1.2: Mission result classification
  1.3: Run benchmark, document failures

Phase 2: Enable Composition           [2-3 weeks]
  2.1: Formal mission state machine
  2.2: Mission composition (parent/child)
  2.3: Agent self-recovery endpoint
  2.4: Adaptive retry budget
  + Wire CancellationToken tree, JoinSet for subagents

Phase 3: Activate Learning            [1-2 weeks]
  3.1: Wire action history persistence
  3.2: Feed failure patterns into recovery
  3.3: Implement 7 priority MCP tools

Phase 4: Simplify Surfaces            [1 week]
  4.1: .tau.toml schema + startup wire
  4.2: Script cleanup (not rewrite)
  4.3: Skills gain tools field
```

**Total: 6-10 weeks of focused work instead of months of sprawling features.**

---

## Part 7: Success Criteria

After completing all phases, Tau should demonstrate:

1. **Autonomy**: Complete 3 benchmark tasks with only auth and major-direction checkpoints
2. **Composition**: Decompose a multi-step task into sub-missions that execute independently
3. **Self-recovery**: Resume from a blocked state without human intervention
4. **Learning**: Failure patterns from session N influence recovery strategy in session N+1
5. **Stability**: Zero zombie processes after timeout, real-time streaming to TUI, <100ms action history I/O
6. **Security**: Safety policy cannot be weakened by the agent, credentials properly derived, secrets not leaked to subprocesses

### What Makes This State-of-the-Art

| Capability | Industry Status | Tau After This Plan |
|------------|----------------|-------------------|
| Verifier-backed continuation | Novel (only Tau) | Strengthened with adaptive retry |
| Mission composition | Table stakes (Feb 2026) | Parent/child missions via orchestrator |
| Agent self-recovery | Rare (Devin only) | Resume endpoint + failure pattern routing |
| Ground-truth verification | Best practice | Auto cargo check/test after every edit |
| Context budget management | Active research | Token tracking in Ralph-loop |
| Five-layer safety | Best practice (OpenDev) | All 5 layers implemented |
| Internal benchmark suite | Emerging practice | 3-tier (conformance/capability/realism) |

---

## Sources and References

### Origin Documents
- [Tranche-One Autonomy Plan](2026-04-03-001-feat-tranche-one-autonomy-vertical-slice-plan.md)
- [Radical Simplification Plan](2026-03-24-001-feat-radical-simplification-self-improvement-plan.md)
- [Wiring Integration Endgame](2026-03-24-002-feat-wiring-integration-endgame-plan.md)

### External Research
- [Anthropic: Building Effective Agents](https://www.anthropic.com/research/building-effective-agents) -- Harness over scaffold, tool design > prompt design
- [Building AI Coding Agents for the Terminal (arXiv, March 2026)](https://arxiv.org/html/2603.05344v1) -- Five-layer defense-in-depth
- [JetBrains Research: Smarter Context Management (Dec 2025)](https://blog.jetbrains.com/research/2025/12/efficient-context-management/) -- Simple masking equals complex summarization
- [SWE-bench Verified Leaderboard](https://epoch.ai/benchmarks/swe-bench-verified/) -- Current SOTA ~80.8%
- [DSPy GEPA: Reflective Prompt Evolution](https://dspy.ai/api/optimizers/GEPA/overview/) -- Trajectory-based prompt optimization
- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25) -- Official MCP Rust SDK is `rmcp`
- [Official Rust MCP SDK (rmcp)](https://github.com/modelcontextprotocol/rust-sdk)
- [Docker: Coding Agent Safety](https://www.docker.com/blog/docker-sandboxes-a-new-approach-for-coding-agent-safety/)
- [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/)

### Internal Key Files
- `crates/tau-provider/src/codex_cli_client.rs` -- Subprocess lifecycle (P0-1, P0-2)
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs` -- Ralph-loop hot path (P0-4)
- `crates/tau-gateway/src/gateway_openresponses/safety_runtime.rs` -- Safety policy mutation (P0-3)
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs` -- Mission state (2.1)
- `crates/tau-agent-core/src/lib.rs` -- Agent config, recovery, circuit breaker
- `crates/tau-tools/src/mcp_server_runtime.rs` -- MCP server with 33+ tools (21 stubs)
- `crates/tau-memory/src/action_history.rs` -- Action history store
- `crates/tau-orchestrator/src/` -- Plan decomposition (disconnected from gateway)
- `crates/tau-safety/src/lib.rs` -- Safety scanning, self-modification evaluation
- `crates/tau-provider/src/credential_store.rs` -- Credential encryption
