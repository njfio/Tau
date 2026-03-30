---
title: "feat: Radical Simplification & Self-Improvement — 7-Feature Implementation Plan"
type: feat
status: active
date: 2026-03-24
origin: ../ideation/2026-03-24-radical-simplification-self-improvement-ideation.md
---

# feat: Radical Simplification & Self-Improvement

## Overview

Implement all 7 surviving ideas from the radical simplification and self-improvement ideation to make Tau vastly simpler, more powerful, easier to work with, and able to improve itself by updating its own files and functionality.

The 7 features are ordered by dependency — earlier phases create foundations that later phases build on. The plan is structured as 4 phases spanning 7 implementation units.

## Problem Statement / Motivation

Tau has extraordinary architectural breadth (46 crates, coding agent, orchestrator, memory, training pipeline, multi-channel runtime, dashboard, gateway, TUI) but suffers from:

1. **Dormant intelligence** — `action_history_enabled` defaults to `false`, meaning the learning infrastructure (ActionHistoryStore, FailurePattern, ToolEffectiveness, SessionFeedback, APO, TraceBasedRewardInference) sits idle.
2. **Extension fragmentation** — Three competing mechanisms (tau-extensions WASM hooks, tau-custom-command operator workflows, tau-skills prompt packages) with overlapping surfaces.
3. **Training pipeline disconnect** — LiveRlBridge exists and bridges sessions to rollouts, but reward inference doesn't feed back into APO by default.
4. **Configuration sprawl** — 22 onboarding files, 17 `TAU_*` env vars, 659-line shell launcher.
5. **Script accumulation** — 224 shell scripts across 7 directories with only 9 exposed via justfile.
6. **Limited composability** — MCP server exposes 13 tools but not session management, orchestration, or learning surfaces.
7. **No self-modification** — The coding agent can write code for user projects but never targets Tau's own files.

(see origin: ../ideation/2026-03-24-radical-simplification-self-improvement-ideation.md)

## Proposed Solution

Four phases, dependency-ordered:

| Phase | Features | Dependencies |
|-------|----------|-------------|
| **Phase 1: Activate Intelligence** | 1. Learn-by-Default, 3. Self-Training Bridge | None — foundational |
| **Phase 2: Simplify Surfaces** | 2. Unify Extensions, 5. Script Consolidation | Independent of Phase 1 |
| **Phase 3: Unify Configuration & Composability** | 4. Declarative .tau.toml, 6. MCP Server Expansion | Phase 2 (extensions unified before config) |
| **Phase 4: Self-Improvement** | 7. Reflexive Source Patching | Phases 1+2+3 (needs learning, safety, training) |

Phases 1 and 2 can run **in parallel** — they touch non-overlapping code surfaces.

---

## Technical Approach

### Phase 1: Activate Intelligence

#### Feature 1: Learn-by-Default with Failure-Pattern-Aware Tool Routing

**Goal:** Make Tau measurably smarter with every session by activating the dormant learning infrastructure.

**Key Files:**
- `crates/tau-agent-core/src/lib.rs` — `AgentConfig` defaults (line ~264: `action_history_enabled: false`)
- `crates/tau-memory/src/action_history.rs` — `ActionHistoryStore`, `FailurePattern`, `ToolEffectiveness`, `SessionFeedback`
- `crates/tau-agent-core/src/recovery.rs` — `RecoveryStrategy`, `select_recovery_strategy()`
- `crates/tau-agent-core/src/cortex_runtime.rs` — `Cortex` cross-session memory

**Implementation:**

**1.1 — Enable Action History by Default**
- Flip `action_history_enabled` to `true` in `AgentConfig::default()` (line ~264)
- Set `action_history_max_records` to `1000` (currently 500)
- Add `action_history_retention_days: u32` field with default `30`
- **Critical:** The store is currently in-memory only (`store_path` exists but JSONL I/O is not wired). Must implement `load()` and `save()` for `ActionHistoryStore` before enabling by default.

```rust
// crates/tau-agent-core/src/lib.rs — AgentConfig::default()
action_history_enabled: true,            // was: false
action_history_max_records: 1000,        // was: 500
action_history_retention_days: 30,       // NEW
```

**1.2 — Implement JSONL Persistence for ActionHistoryStore**
- `ActionHistoryStore::load(path)` — reads JSONL, filters by retention_days
- `ActionHistoryStore::save(path)` — writes JSONL with atomic rename
- Call `load()` at session start, `save()` at session end
- Wire into `AgentEvent::AgentStart` / `AgentEvent::AgentEnd` handlers

**1.3 — Feed Failure Patterns into Recovery**
- Modify `select_recovery_strategy()` in `recovery.rs` to accept an optional `&ActionHistoryStore`
- Before selecting strategy, query `failure_patterns(lookback=50)` and `tool_success_rates(lookback=50)`
- If a tool has success rate < 30%, inject `RecoveryStrategy::RetryWithHint` with the common error pattern
- If a tool has 3+ consecutive failures (from `FailurePattern.occurrence_count`), suggest `RecoveryStrategy::AlternativeApproach`

**1.4 — Inject Learning Context into Cortex Bulletin**
- Extend `Cortex::refresh_once()` to include top failure patterns and tool effectiveness data
- Add a "Learning Insights" section to the cross-session bulletin:
  - Top 3 failing tools with common errors
  - Tools with declining success rates
  - Recovery strategies that worked

**1.5 — Add `tau learn` Operator Commands**
- `tau learn status` — show action history stats (total records, top patterns, success rates)
- `tau learn clear` — clear all learned data with confirmation
- `tau learn export` — export to JSON for debugging
- Register in `crates/tau-ops/src/command_catalog.rs`

**Tests:**
- Unit: ActionHistoryStore JSONL round-trip (load/save/retention pruning)
- Unit: Failure-pattern-aware recovery strategy selection
- Integration: Multi-session learning persistence (create patterns in session 1, verify retrieval in session 2)
- Integration: Cortex bulletin includes learning insights after 5+ recorded actions

**Verification:** After 10 sessions with the same tool failing, `tau learn status` shows the failure pattern, and the agent proactively uses `RetryWithHint` instead of blind retry.

---

#### Feature 3: Closed-Loop Self-Training Bridge

**Goal:** Ensure production sessions automatically feed the training pipeline so APO can optimize prompts.

**Key Files:**
- `crates/tau-coding-agent/src/live_rl_runtime.rs` — `LiveRlBridge` (line 440+)
- `crates/tau-algorithm/src/reward_inference.rs` — `TraceBasedRewardInference`
- `crates/tau-algorithm/src/apo.rs` — `ApoAlgorithm`
- `crates/tau-training-store/src/lib.rs` — `TrainingStore` trait

**Research Finding:** LiveRlBridge already exists and is more complete than the ideation assumed. It handles `AgentEvent::AgentEnd`, creates rollouts, persists spans with enrichment (task_category, learning_trend, historical_success_rate), and triggers optimizer updates every N rollouts. The gap is:
1. Is LiveRlBridge enabled by default? (Needs verification)
2. Does reward inference score flow into rollout metadata? (Partially — `RewardInferenceInput` exists but may not be wired)
3. Does APO trigger automatically from accumulated rollouts? (Manual trigger only)

**Implementation:**

**3.1 — Ensure LiveRlBridge is Enabled by Default**
- Verify that the live RL bridge activates without requiring explicit opt-in
- If gated behind a config flag, enable it by default in `AgentConfig`
- Ensure `TrainingStore` (SQLite backend) is initialized during normal startup, not just training mode

**3.2 — Wire RewardInference into Rollout Finalization**
- In `LiveRlBridge::finalize_run()` (line 642), after building the `final_decision_span`:
  - Construct `RewardInferenceInput` from span data (has_assistant_reply, session_completed, tool_errors, safety_blocked, turns, input_chars, output_chars)
  - Call `TraceBasedRewardInference::infer()` to get composite score
  - Persist reward score as span metadata: `span.reward = Some(reward_output.composite)`
  - This gives every production rollout a deterministic reward signal

**3.3 — Add APO Trigger Threshold**
- Add config: `apo_auto_trigger_threshold: usize` (default: 20 rollouts)
- In `LiveRlBridge`, after each finalized rollout, check accumulated count
- When threshold reached, spawn background APO run using `ApoAlgorithm::run()` with:
  - `seed_prompt` from current system prompt
  - `train_examples` from recent rollouts (top/bottom reward quartiles)
  - `validation_examples` from held-out rollouts
- APO persists improved prompt via `TrainingStore::update_resources()`

**3.4 — Surface Training Status to Operator**
- Add `tau training status` command showing: rollout count, last reward score, APO run history, current prompt version
- Add `tau training trigger` to manually trigger APO optimization
- Register in command catalog

**Tests:**
- Unit: RewardInferenceInput construction from span data
- Unit: APO trigger threshold logic
- Integration: End-to-end session → rollout → reward inference → span persistence
- Integration: APO auto-trigger after N rollouts produces improved prompt

**Verification:** After 20 production sessions, `tau training status` shows accumulated rollouts with reward scores, and at least one APO optimization run has completed.

---

### Phase 2: Simplify Surfaces

#### Feature 2: Unify Extension Mechanisms into Skills-Only

**Goal:** Consolidate three extension surfaces into one.

**Key Files:**
- `crates/tau-skills/src/lib.rs` — Skill loading, catalog, selection, trust, lockfile
- `crates/tau-extensions/src/lib.rs` — ExtensionManifest, hooks, WASM runtime, tool/command registration
- `crates/tau-custom-command/src/lib.rs` — CustomCommand contract, policy, runtime

**Gap Analysis (from research):**

| Capability | tau-skills | tau-extensions | tau-custom-command |
|-----------|-----------|---------------|-------------------|
| Prompt augmentation | Yes (core purpose) | Via message transforms | No |
| Tool registration | No | Yes (JSON schema tools) | No |
| Command registration | No | Yes | Yes (templates) |
| WASM execution | No | Yes | No |
| Process execution | No | Yes | Via templates |
| Lifecycle hooks | No | Yes (6 hook types) | No |
| Policy overrides | No | Yes | Yes (execution policy) |
| Trust/signing | Yes (Ed25519) | No | No |
| Package management | Yes (manifest, lockfile) | Manifest only | No |
| Auto-selection | Yes (token scoring) | No | No |

**Implementation:**

**2.1 — Extend Skill Manifest to Support Tools and Commands**
- Add optional fields to `Skill` struct:
  ```rust
  pub struct Skill {
      pub name: String,
      pub description: String,
      pub content: String,           // prompt content (existing)
      pub path: Option<PathBuf>,
      pub base_dir: Option<PathBuf>,
      // NEW fields:
      pub tools: Option<Vec<SkillToolDefinition>>,      // JSON schema tool defs
      pub commands: Option<Vec<SkillCommandDefinition>>, // operator commands
      pub hooks: Option<Vec<SkillHook>>,                 // lifecycle hooks
      pub runtime: Option<SkillRuntime>,                 // Process or Wasm
      pub entrypoint: Option<String>,                    // for runtime execution
      pub permissions: Option<Vec<SkillPermission>>,     // capability grants
  }
  ```
- `SkillToolDefinition` mirrors `ExtensionRegisteredTool` schema
- `SkillCommandDefinition` mirrors `CustomCommandSpec` template model
- `SkillHook` mirrors `ExtensionHook` (RunStart, RunEnd, PreToolCall, etc.)

**2.2 — Add Skill Execution Runtime**
- New module: `crates/tau-skills/src/skill_runtime.rs`
- Implements tool dispatch for skills that declare tools (delegates to WASM sandbox or process execution)
- Implements hook dispatch for skills that declare hooks
- Implements command dispatch for skills that declare commands
- Reuses existing `WasmSandboxRuntime` and `Command::new()` patterns from tau-extensions

**2.3 — Migrate Extension Capabilities into Skills**
- For each built-in extension: create equivalent skill package
- For each custom command: create equivalent skill with command definition
- Ensure trust model applies uniformly — all skills go through Ed25519 verification

**2.4 — Deprecate tau-extensions and tau-custom-command**
- Add deprecation notices pointing to skill equivalents
- Keep crates compilable but mark as `#[deprecated]`
- Remove from default feature set in a future release
- Update AGENTS.md to reference skills as the single extension surface

**Tests:**
- Unit: Skill manifest parsing with tool/command/hook fields
- Unit: Skill runtime tool dispatch (WASM and process)
- Integration: End-to-end skill with tools registers and executes via MCP
- Integration: Lifecycle hook dispatch from skill
- Migration: All existing extensions have skill equivalents that pass same test cases

**Verification:** `tau skills list` shows unified catalog including tools, commands, and hooks. `tau extensions list` shows deprecation notice.

---

#### Feature 5: Script Consolidation into `tau ops` CLI

**Goal:** Replace 224 shell scripts with discoverable `tau ops` subcommands.

**Key Files:**
- `crates/tau-ops/src/lib.rs` — existing module structure
- `crates/tau-ops/src/command_catalog.rs` — 45 existing command specs
- `scripts/` — 224 shell scripts across 7 directories

**Implementation:**

**5.1 — Audit and Classify Scripts**
- Run script audit: for each of the 224 scripts, determine:
  - (a) Referenced by CI, justfile, or AGENTS.md → **migrate**
  - (b) Referenced by any spec or doc → **migrate**
  - (c) Orphaned / one-off analysis → **archive** to `scripts/archive/`
- Expected outcome: ~40-60 scripts to migrate, ~160 to archive

**5.2 — Define `tau ops` Subcommand Groups**
- Extend command catalog with operational subcommands:
  ```
  tau ops verify <gate>     — run verification gates (from scripts/verify/)
  tau ops dev <task>        — dev workflow tasks (from scripts/dev/)
  tau ops release <step>    — release pipeline steps (from scripts/release/)
  tau ops demo <scenario>   — demo scenarios (from scripts/demo/)
  tau ops qa <suite>        — QA loop tasks (from scripts/qa/)
  ```
- Each group lists available subcommands via `tau ops verify --list`

**5.3 — Implement Core Verification Gates in Rust**
- Migrate the 35 `scripts/verify/m*` scripts into `crates/tau-ops/src/verification_gates.rs`
- Each gate becomes a function: `async fn verify_m296_ga_readiness() -> GateResult`
- `GateResult` struct: name, passed, duration_ms, failures (Vec<String>)
- `tau ops verify all` runs all gates with aggregated pass/fail report
- `tau ops verify all --json` for CI consumption

**5.4 — Migrate `fast-validate.sh` Logic**
- Rewrite impacted-package detection in Rust using `cargo metadata`
- `tau ops validate` replaces `fast-validate.sh`
- `tau ops validate --full` for pre-merge gate (equivalent to `--full` flag)
- `tau ops validate --check-only` for quick check

**5.5 — Add CI Gate for New Scripts**
- CI check: any new `.sh` file in `scripts/` (outside `scripts/archive/`) must be registered in the command catalog or the PR is blocked
- Prevents future script accumulation

**5.6 — Update justfile**
- Replace shell script invocations with `tau ops` commands
- Add tab completion generation: `tau ops completions zsh > _tau`

**Tests:**
- Unit: Each migrated verification gate produces correct GateResult
- Unit: Impacted-package detection matches fast-validate.sh output
- Integration: `tau ops verify all` runs and reports aggregate results
- CI: Gate that blocks unregistered scripts

**Verification:** `tau ops --help` shows all operational commands. `just verify` delegates to `tau ops verify all`. Zero scripts in `scripts/dev/` or `scripts/verify/` that aren't mirrored in the ops CLI.

---

### Phase 3: Unify Configuration & Composability

#### Feature 4: Declarative Agent Composition (.tau.toml)

**Goal:** Single config file as the source of truth for agent configuration.

**Key Files:**
- `crates/tau-onboarding/src/startup_config.rs` — `ProfileDefaults` (line 215-227)
- `crates/tau-onboarding/src/onboarding_wizard.rs` — wizard plan and non-interactive mode
- `crates/tau-onboarding/src/startup_dispatch.rs` — startup orchestration

**Implementation:**

**4.1 — Define `.tau.toml` Schema**
```toml
[agent]
name = "my-agent"
model = "claude-sonnet-4-6"
fallback_models = ["claude-haiku-4-5-20251001"]

[session]
enabled = true
path = ".tau/sessions"
import_mode = "auto"

[policy]
tool_policy_preset = "standard"
bash_profile = "default"
os_sandbox_mode = "relaxed"
bash_timeout_ms = 30000

[memory]
action_history_enabled = true
action_history_retention_days = 30
cortex_enabled = true

[training]
live_rl_enabled = true
apo_auto_trigger_threshold = 20

[safety]
enabled = true
mode = "warn"
secret_leak_detection = true

[channels]
# Optional multi-channel config
slack = { enabled = false }
discord = { enabled = false }
github_issues = { enabled = false }

[skills]
# Skill packages to load
include = ["web-game-phaser"]
auto_select = true

[auth]
openai_auth_mode = "api_key"
anthropic_auth_mode = "api_key"
```

**4.2 — Implement Config Parser**
- New module: `crates/tau-onboarding/src/config_file.rs`
- `TauConfig` struct mirrors `.tau.toml` schema with `#[derive(Deserialize)]`
- `load_tau_config(path)` — reads `.tau.toml`, validates, returns `TauConfig`
- `TauConfig::to_profile_defaults()` — converts to existing `ProfileDefaults`
- Precedence: `.tau.toml` < environment variables < CLI flags (most specific wins)

**4.3 — Wire Config into Startup**
- In `startup_dispatch.rs`, check for `.tau.toml` in project root before running wizard
- If found, use it as the base config (skip wizard prompts for any configured fields)
- If not found, fall through to existing wizard/env-var path

**4.4 — Add `tau init` Command**
- `tau init` — interactive mode: asks key questions, writes `.tau.toml`
- `tau init --auto` — non-interactive: writes `.tau.toml` with sensible defaults (no prompts)
- `tau init --from-env` — generates `.tau.toml` from current environment variables
- Register in command catalog

**4.5 — Config Validation and Doctor Integration**
- `tau config validate` — validates `.tau.toml` against schema, reports errors
- `tau config show` — shows resolved config (merged from file + env + flags)
- Wire into existing `tau-diagnostics` health checks

**Tests:**
- Unit: TOML parsing round-trip for all config fields
- Unit: Precedence resolution (file < env < CLI)
- Unit: `to_profile_defaults()` produces correct ProfileDefaults
- Integration: `tau init --auto` generates valid `.tau.toml` that starts a working session
- Integration: Config from `.tau.toml` matches config from equivalent env vars

**Verification:** A new user can `tau init --auto && tau` and get a working agent session with zero env vars and zero manual configuration.

---

#### Feature 6: Tau as Full MCP Server

**Goal:** Expand MCP server surface to make Tau composable infrastructure.

**Key Files:**
- `crates/tau-tools/src/mcp_server_runtime.rs` — current 13 tools + 3 context providers
- `crates/tau-session/src/lib.rs` — SessionStore, SessionRuntime
- `crates/tau-orchestrator/src/lib.rs` — multi-agent orchestration
- `crates/tau-memory/src/action_history.rs` — learning data

**Current MCP Surface (13 tools):**
| Tool | Category |
|------|----------|
| `tau.read`, `tau.write`, `tau.edit` | File I/O |
| `tau.memory_write/read/search/tree` | Memory |
| `tau.jobs_create/list/status/cancel` | Background Jobs |
| `tau.http`, `tau.bash` | Execution |

**Implementation:**

**6.1 — Add Session Management Tools**
```
tau.session_list       — list recent sessions
tau.session_resume     — resume a previous session
tau.session_search     — search session history
tau.session_stats      — session statistics
tau.session_export     — export session as JSON
```

**6.2 — Add Orchestration Tools**
```
tau.agent_spawn        — spawn a sub-agent with a task
tau.agent_status       — check sub-agent status
tau.agent_cancel       — cancel a running sub-agent
tau.plan_create        — create a structured plan (DAG)
tau.plan_validate      — validate plan structure
```

**6.3 — Add Learning & Training Tools**
```
tau.learn_status       — current learning insights (failure patterns, tool rates)
tau.learn_failure_patterns — query failure patterns
tau.learn_tool_rates   — query tool success rates
tau.training_status    — rollout count, APO history
tau.training_trigger   — trigger APO optimization
```

**6.4 — Add Skills Management Tools**
```
tau.skills_list        — list installed skills
tau.skills_search      — search skill catalog
tau.skills_install     — install a skill package
tau.skills_info        — skill details and trust status
```

**6.5 — Add Context Providers**
```
tau.context.learning   — inject learning insights into context
tau.context.training   — inject training status into context
tau.context.config     — inject current agent config
```

**6.6 — Implement Tool Handlers**
- Each new tool gets a handler in `mcp_server_runtime.rs`
- Handlers delegate to existing runtime functions (SessionRuntime, ActionHistoryStore, etc.)
- All tools go through `ToolPolicy` enforcement before execution
- Add tool schemas with JSON Schema parameters

**Tests:**
- Unit: Each new MCP tool handler returns correct schema
- Integration: MCP client can call each tool and get valid response
- Integration: Tool policy blocks unauthorized MCP tool calls
- E2E: External MCP client (Claude Desktop) discovers and calls Tau tools

**Verification:** `mcp tools/list` on Tau's MCP server returns 30+ tools covering file I/O, memory, sessions, orchestration, learning, training, and skills.

---

### Phase 4: Self-Improvement

#### Feature 7: Reflexive Source Patching Engine (Phased)

**Goal:** Tau modifies its own files — starting with low-risk targets, graduating to crate source.

**Key Files:**
- `crates/tau-coding-agent/src/` — coding agent runtime
- `crates/tau-safety/src/lib.rs` — SafetyPolicy, SafetyMode, SafetyRule
- `crates/tau-training-store/src/lib.rs` — TrainingStore for tracking patches
- `crates/tau-skills/src/lib.rs` — skill files (Phase 7A target)

**Prerequisites from Earlier Phases:**
- Phase 1: Learning active → agent knows which patterns fail
- Phase 1: Training bridge active → patches become training rollouts
- Phase 2: Skills unified → single target surface for self-modification
- Phase 3: Config declarative → config changes are safe, bounded

**Implementation:**

**7A — Skill & Config Self-Modification (Low Risk)**

**7A.1 — Define Self-Modification Safety Policy**
- New `SafetyStage::SelfModification` variant
- New safety rules:
  - `self_mod_skills_only`: allow modification of files in `skills/` directory
  - `self_mod_config_only`: allow modification of `.tau.toml`
  - `self_mod_prompt_only`: allow modification of prompt templates
  - `self_mod_block_source`: block modification of `crates/*/src/**` files
- Default policy: `SelfModification` stage uses `SafetyMode::Block` for source, `SafetyMode::Warn` for skills/config

**7A.2 — Self-Modification Pipeline**
```
Trigger → Propose → Test → Gate → Apply → Track
```
1. **Trigger:** Agent detects a learnable improvement (e.g., skill consistently underperforms via `ToolEffectiveness`)
2. **Propose:** Agent generates a diff for the target file
3. **Test:** Run relevant test suite against the proposed change
4. **Gate:** Safety pipeline evaluates the change (stage: `SelfModification`)
5. **Apply:** If tests pass and safety clears, apply the change
6. **Track:** Record as a training rollout with reward signal from test results

**7A.3 — Implement Self-Modification Runtime**
- New module: `crates/tau-coding-agent/src/self_modification_runtime.rs`
- `SelfModificationProposal` struct: target_path, diff, rationale, trigger_source
- `SelfModificationResult` struct: applied, rollout_id, test_results, safety_evaluation
- `evaluate_self_modification(proposal)` → runs safety pipeline, tests, and applies
- Operator notification: always notify when a self-modification is proposed (even if auto-approved)

**7A.4 — Operator Controls**
- `tau self-modify status` — show recent self-modifications
- `tau self-modify review` — review pending proposals
- `tau self-modify rollback <id>` — revert a specific modification
- `tau self-modify policy` — show/configure self-modification policy
- Config in `.tau.toml`:
  ```toml
  [self_improvement]
  enabled = true
  auto_apply_skills = true      # auto-apply skill improvements
  auto_apply_config = false     # require approval for config changes
  auto_apply_source = false     # always require approval for source changes
  ```

**7B — Source Code Self-Modification (High Risk, Future)**

**7B.1 — Prerequisites Gate**
- Requires: contract testing mature (executable specs exist for target crate)
- Requires: training pipeline producing reliable reward signals (20+ scored rollouts)
- Requires: git worktree isolation (patches applied in worktree, not main)

**7B.2 — Source Modification Pipeline**
```
Trigger → Worktree → Propose → Full Test Suite → Safety Review → PR → Human Approval → Merge
```
1. **Trigger:** Recurring failure pattern with clear fix hypothesis
2. **Worktree:** Create isolated git worktree for the patch
3. **Propose:** Agent writes the fix in the worktree
4. **Full Test Suite:** Run `cargo test` across entire workspace
5. **Safety Review:** Full safety scan of the diff
6. **PR:** Create a draft PR with the proposed change
7. **Human Approval:** Always requires human merge — never auto-merge source changes
8. **Merge:** After approval, merge and record as training rollout

**7B.3 — Rollback and Recovery**
- Every source modification creates a git commit with `self-mod(#rollout-id):` prefix
- `tau self-modify rollback <id>` creates a revert commit
- If tests fail after merge, auto-create a revert PR

**Tests:**
- Unit: Self-modification safety policy correctly gates by file type
- Unit: Proposal generation and diff formatting
- Integration: Skill self-modification end-to-end (propose → test → apply → track)
- Integration: Source modification creates draft PR in worktree (7B)
- Integration: Rollback reverts the exact change
- Safety: Attempt to self-modify `crates/tau-safety/src/lib.rs` is BLOCKED (safety can't weaken itself)

**Verification:**
- 7A: After 20 sessions where a skill underperforms, Tau proposes an improvement to the skill file, runs tests, and applies it if approved.
- 7B: After detecting a recurring tool failure with a clear fix, Tau creates a draft PR with the fix in a worktree. Human reviews and merges.

---

## System-Wide Impact

### Interaction Graph

```
Session Start
  → ActionHistoryStore.load() (Feature 1)
  → .tau.toml resolution (Feature 4)
  → Skill catalog load with tools/commands/hooks (Feature 2)
  → MCP server starts with full tool surface (Feature 6)

Session Turn
  → Tool execution recorded to ActionHistoryStore
  → Failure patterns queried before recovery strategy selection (Feature 1)
  → Cortex bulletin includes learning insights (Feature 1)
  → MCP tools available to external clients (Feature 6)

Session End
  → ActionHistoryStore.save() (Feature 1)
  → LiveRlBridge creates rollout with reward score (Feature 3)
  → If rollout count >= threshold → APO trigger (Feature 3)
  → If learnable improvement detected → self-modification proposal (Feature 7)

Ops Commands
  → tau ops verify/validate/dev (Feature 5)
  → tau learn status/clear/export (Feature 1)
  → tau training status/trigger (Feature 3)
  → tau self-modify status/review/rollback (Feature 7)
```

### Error & Failure Propagation

- **ActionHistoryStore persistence failure:** Degrade gracefully — continue session without persistence, log warning. Never block session execution.
- **LiveRlBridge failure:** Session continues normally. Training data lost for that session but no user impact.
- **APO trigger failure:** Log error, increment retry counter. Do not block sessions. Alert operator via `tau training status`.
- **Self-modification test failure:** Proposal rejected. No change applied. Operator notified. Recorded as negative training signal.
- **Self-modification safety block:** Proposal rejected. Logged for audit. No escalation path — safety blocks are final.
- **Config file parse failure:** Fall through to env var / CLI flag path. Log warning with specific parse error.
- **MCP tool handler failure:** Return MCP error response. Do not crash server. Log for debugging.

### State Lifecycle Risks

- **ActionHistoryStore growth:** Mitigated by retention_days (30) and max_records (1000). JSONL pruned on load.
- **Training rollout accumulation:** SQLite backend handles growth. Add periodic vacuum to `tau ops maintenance`.
- **Self-modification race:** Only one self-modification proposal active at a time (mutex). Worktree isolation prevents conflicts.
- **Config file conflicts:** `.tau.toml` is the source of truth. Env vars and CLI flags override but don't persist.

### API Surface Parity

- All new operator commands (`tau learn`, `tau training`, `tau self-modify`, `tau ops`) must be accessible via:
  1. CLI commands
  2. MCP tools (Feature 6)
  3. TUI command palette (existing command catalog integration)
  4. Gateway API routes (for dashboard/remote access)

---

## Acceptance Criteria

### Functional Requirements

- [ ] Action history records persist across sessions (JSONL)
- [ ] Failure patterns influence recovery strategy selection
- [ ] Cortex bulletin includes learning insights
- [ ] `tau learn status/clear/export` commands work
- [ ] Skills manifest supports tools, commands, hooks, and runtime fields
- [ ] Existing extensions have skill equivalents
- [ ] Production sessions produce scored training rollouts
- [ ] APO triggers automatically after configurable threshold
- [ ] `.tau.toml` configures all agent settings
- [ ] `tau init --auto` produces working config in < 2 seconds
- [ ] 224 shell scripts audited; survivors migrated to `tau ops`
- [ ] `tau ops verify all` runs all verification gates
- [ ] MCP server exposes 30+ tools covering all major surfaces
- [ ] Self-modification pipeline works for skill files
- [ ] Self-modification safety blocks source code changes by default
- [ ] Operator controls for self-modification (status, review, rollback)

### Non-Functional Requirements

- [ ] ActionHistoryStore load/save < 100ms for 1000 records
- [ ] `.tau.toml` parse < 10ms
- [ ] MCP tool discovery (tools/list) < 50ms
- [ ] Self-modification safety evaluation < 500ms
- [ ] No regression in existing test suite
- [ ] All new code has > 80% test coverage

### Quality Gates

- [ ] All existing tests pass after each feature
- [ ] New tests follow TDD (red → green → refactor)
- [ ] Each feature has integration tests covering cross-system interactions
- [ ] Self-modification safety tests include adversarial cases
- [ ] AGENTS.md updated to reflect new command surface

---

## Dependencies & Prerequisites

| Feature | Depends On | Blocks |
|---------|-----------|--------|
| 1. Learn-by-Default | None | 3, 7 |
| 2. Unify Extensions | None | 4, 7 |
| 3. Self-Training Bridge | 1 (learning data feeds training) | 7 |
| 4. Declarative Config | 2 (skills unified first) | 7 |
| 5. Script Consolidation | None | None |
| 6. MCP Server Expansion | None (but benefits from 1, 2, 3) | None |
| 7. Reflexive Source Patching | 1, 2, 3, 4 | None |

**Parallel execution possible:**
- Phase 1 (Features 1, 3) and Phase 2 (Features 2, 5) can run simultaneously
- Feature 6 can start anytime after Features 1 and 2 are underway

---

## Risk Analysis & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| ActionHistoryStore persistence breaks existing sessions | Medium | High | Feature flag `action_history_persist: bool` with fallback to in-memory |
| Extension → Skill migration misses capabilities | Medium | Medium | Gap analysis table (Section 2.1) as acceptance gate before deprecation |
| APO auto-trigger produces worse prompts | Low | High | A/B test: run optimized prompt on 10% of sessions, compare reward scores |
| Self-modification applies harmful change | Low | Critical | Safety pipeline + test gate + operator notification. Source changes always require human approval |
| `.tau.toml` conflicts with env var workflow | Medium | Low | Clear precedence: file < env < CLI. Document in README |
| Script consolidation breaks CI | Medium | Medium | Parallel run period: keep scripts + ops commands, verify identical output |

---

## Implementation Phases & Estimated Effort

| Phase | Features | Estimated PRs | Can Parallel |
|-------|----------|--------------|-------------|
| Phase 1 | 1 (Learn-by-Default), 3 (Self-Training) | 6-8 PRs | Yes (with Phase 2) |
| Phase 2 | 2 (Unify Extensions), 5 (Scripts) | 8-10 PRs | Yes (with Phase 1) |
| Phase 3 | 4 (.tau.toml), 6 (MCP Server) | 6-8 PRs | After Phase 2 |
| Phase 4 | 7A (Skill self-mod), 7B (Source self-mod) | 4-6 PRs | After Phases 1-3 |

---

## Post-Deploy Monitoring & Validation

### What to Monitor
- **Logs:** `action_history_store`, `live_rl_bridge`, `apo_trigger`, `self_modification` log prefixes
- **Metrics:** action_history_records_total, training_rollouts_total, apo_runs_total, self_modifications_proposed/applied/rejected

### Validation Checks
- `tau learn status` shows non-zero records after first session
- `tau training status` shows rollouts accumulating
- `tau ops verify all` produces aggregate report
- `tau config show` renders resolved configuration
- MCP `tools/list` returns 30+ tools

### Expected Healthy Behavior
- Action history grows by 5-20 records per session
- Reward scores cluster between 0.3-0.8 for normal sessions
- APO trigger fires after configurable threshold (default 20 rollouts)
- Self-modification proposals are rare (< 1 per 50 sessions)

### Failure Signals / Rollback Trigger
- Action history file corruption → revert to in-memory mode
- APO produces prompt with reward < 0.1 → revert to previous prompt version
- Self-modification breaks tests → auto-rollback, disable auto-apply
- MCP server crashes → restart with reduced tool surface

### Validation Window & Owner
- Window: 2 weeks after each phase deployment
- Owner: project maintainer

---

## Sources & References

### Origin
- **Origin document:** [../ideation/2026-03-24-radical-simplification-self-improvement-ideation.md](../ideation/2026-03-24-radical-simplification-self-improvement-ideation.md) — 7 surviving ideas from 40 raw candidates across 5 ideation frames

### Internal References
- Action history: `crates/tau-memory/src/action_history.rs`
- Recovery strategies: `crates/tau-agent-core/src/recovery.rs`
- LiveRlBridge: `crates/tau-coding-agent/src/live_rl_runtime.rs:440`
- APO algorithm: `crates/tau-algorithm/src/apo.rs`
- Reward inference: `crates/tau-algorithm/src/reward_inference.rs`
- Extension manifest: `crates/tau-extensions/src/lib.rs:289`
- Skill catalog: `crates/tau-skills/src/lib.rs`
- Custom commands: `crates/tau-custom-command/src/lib.rs`
- MCP server: `crates/tau-tools/src/mcp_server_runtime.rs`
- Safety pipeline: `crates/tau-safety/src/lib.rs`
- Config defaults: `crates/tau-onboarding/src/startup_config.rs:215`
- Command catalog: `crates/tau-ops/src/command_catalog.rs`
- Cortex runtime: `crates/tau-agent-core/src/cortex_runtime.rs`
- Agent events: `crates/tau-agent-core/src/lib.rs` (AgentEvent enum)

### Related Plans
- Self-improvement engine: `docs/plans/2026-03-23-004-feat-self-improvement-engine-plan.md`
- Agent improvements: `docs/AGENT_IMPROVEMENTS_PLAN.md`
- Autonomous operator mission control: `docs/ideation/2026-03-23-autonomous-operator-mission-control-ideation.md`
