---
date: 2026-03-24
topic: radical-simplification-self-improvement
focus: vast improvement, simplification, power, usefulness, ease of install, self-improvement by updating own files
---

# Ideation: Radical Simplification & Self-Improvement

## Codebase Context

Tau is a 46-crate Rust agent runtime (v0.1.0) with a coding agent, orchestrator, memory, training pipeline (6 crates with PPO/GAE/APO), multi-channel runtime (Slack, Discord, GitHub Issues), dashboard, gateway (~30 routes), and TUI. It has strong spec-driven governance (2,339 spec files, 679 spec dirs) and TDD culture.

Key structural observations:
- **Self-improvement infrastructure is built but dormant** — `ActionHistoryStore`, `SessionFeedback`, `FailurePattern`, `ToolEffectiveness`, APO algorithm, `TraceBasedRewardInference` all exist. `action_history_enabled` defaults to `false`.
- **Three competing extension mechanisms** — `tau-extensions` (WASM), `tau-custom-command`, `tau-skills` (richest infra with package manifest, trust roots, lockfile sync).
- **224 shell scripts** across 7 directories with only 9 exposed via justfile. `tau-ops` crate exists but isn't wired to most scripts.
- **22 onboarding source files**, 17 `TAU_*` env vars, 659-line `tau-unified.sh` launcher — significant first-run friction.
- **Six training crates** (types, store, tracer, runner, proxy, trainer) with RL harness, rollout management, and APO — all disconnected from production sessions.
- **No pre-built binary distribution** — requires Rust toolchain and full source build.

Past ideation: `2026-03-23-autonomous-operator-mission-control-ideation.md` covers TUI/operator experience. This ideation focuses on the structural and self-improvement dimensions.

## Ranked Ideas

### 1. Learn-by-Default with Failure-Pattern-Aware Tool Routing
**Description:** Flip `action_history_enabled` to `true`. Feed `failure_patterns()` and `tool_success_rates()` into planning and recovery decisions. When the agent encounters a tool with declining success rates, inject hints from `RecoveryStrategy::RetryWithHint` proactively. Add retention policy and a `tau learn clear` command.
**Rationale:** All infrastructure exists — ActionHistoryStore, FailurePattern, ToolEffectiveness, SessionFeedback, circuit breaker, recovery strategies. This is a flag flip + query layer on data already collected. Highest ROI idea on the list. Makes Tau measurably smarter with every session.
**Downsides:** Storage growth over 1000+ sessions needs retention policy. Privacy implications of recording all actions.
**Confidence:** 90%
**Complexity:** Low
**Status:** Unexplored

### 2. Unify Extension Mechanisms into Skills-Only
**Description:** Consolidate tau-extensions (WASM), tau-custom-command, and tau-skills into a single "skills" extension surface. Extend the skill manifest to optionally include WASM entrypoints and tool schemas. One format, one trust model, one install path.
**Rationale:** Three competing extension mechanisms at v0.1.0 is a design smell. Skills already have the richest infrastructure (package manifest, trust roots, lockfile sync, signature verification). Directly addresses "super simple to work with."
**Downsides:** Needs gap analysis to ensure no capabilities lost. Migration path for existing extensions.
**Confidence:** 82%
**Complexity:** Medium
**Status:** Unexplored

### 3. Closed-Loop Self-Training (Scoped: Sessions to Training Store)
**Description:** Build one bridge: completed production sessions automatically write scored rollouts to tau-training-store using TraceBasedRewardInference. Don't automate the full flywheel yet — just close the first hop. APO prompt optimization can be triggered manually initially.
**Rationale:** Six training crates represent massive investment sitting idle. The reward inference trait exists. Session recording exists. The missing piece is a ~200-line async bridge between session completion events and training store writes. Foundation for all self-improvement.
**Downsides:** Reward inference quality depends on signal design. SQLite-backed training store may need scaling review.
**Confidence:** 78%
**Complexity:** Medium
**Status:** Unexplored

### 4. Declarative Agent Composition (.tau.toml)
**Description:** Single config file as source of truth for agent configuration: model, skills, safety policy, channels, memory backends, training settings. 22 onboarding files stay as implementation, but users touch one file. `tau init` generates sensible defaults; `tau init --auto` skips all prompts.
**Rationale:** 22 onboarding source files, 17 TAU_* env vars, scattered CLI flags, and 659-line shell launcher confirm real fragmentation. Agent-as-code — version-controlled, diffable, shareable definitions.
**Downsides:** Adds a parser on top of existing config; doesn't eliminate implementation complexity, just hides it.
**Confidence:** 75%
**Complexity:** Medium
**Status:** Unexplored

### 5. Script Consolidation into `tau ops` CLI
**Description:** Audit 224 shell scripts. Archive orphans, merge overlapping scripts, consolidate survivors into `tau ops <subcommand>` with tab completion. tau-ops crate already exists — wire verification gates and dev scripts into it. CI gate prevents new unregistered scripts.
**Rationale:** 224 scripts across 7 directories with only 9 exposed via justfile is accumulated process debt. Moving logic into Rust gives type safety, testability, and discoverability. Bounded, actionable, directly simplifies DX.
**Downsides:** Migration effort for 224 scripts. Some legitimately one-off.
**Confidence:** 80%
**Complexity:** Medium
**Status:** Unexplored

### 6. Tau as Full MCP Server
**Description:** Audit existing mcp_server.rs and expand MCP server surface to expose memory queries, session management, multi-agent orchestration, and training runs as MCP tools. Makes Tau composable — other MCP clients use Tau as infrastructure.
**Rationale:** MCP server code already exists. Gateway's 30+ routes map naturally to MCP tool definitions. tau-memory semantic search is a high-value MCP resource. Flips Tau from competing with other agents to being infrastructure they build on.
**Downsides:** Needs audit of existing MCP server completeness. May already be 70% done.
**Confidence:** 76%
**Complexity:** Medium
**Status:** Unexplored

### 7. Reflexive Source Patching Engine (Phased)
**Description:** Tau's coding agent modifies its own files — starting with skill files, config defaults, and prompt templates (Low complexity), graduating to crate source code (High complexity) once contract testing and training pipeline are mature. All changes gated by tau-safety, tracked as training rollouts, full test suite as acceptance gate.
**Rationale:** Directly answers "self-improving by updating its own files and functionality." Coding agent, safety pipeline, and training store all exist. Phased approach makes this achievable now while building toward the bold vision.
**Downsides:** Safety surface enormous for crate-level changes. Requires mature contract testing first. Rollback mechanism needs design.
**Confidence:** 65%
**Complexity:** High (phased: Low for skills/config, High for crate source)
**Status:** Unexplored

## Rejection Summary

| # | Idea | Reason Rejected |
|---|------|-----------------|
| 1 | Crate Consolidation (46 to ~10) | Too disruptive at v0.1.0; needs concrete dependency analysis, not a target number |
| 2 | Single-Binary Distribution + Self-Update | Premature — no external user base; cargo install suffices |
| 3 | Executable Contracts replacing all specs | 2,339 specs can't convert at once; adopt incrementally instead |
| 4 | WASM Component Model plugins | No plugin ecosystem to serve; raw ABI fine for internal use |
| 5 | CRDT Sync Layer replacing Gateway | Zero CRDT infrastructure; solves nonexistent scaling problem |
| 6 | OpenAPI Auto-Generation | Low value without external API consumers |
| 7 | Episodic Memory / Causal Graphs | Too vague; natural follow-on once learn-by-default produces data |
| 8 | Cross-Channel Learning | Depends on learn-by-default being complete first |
| 9 | Runtime Tool Synthesis | Undefined implementation; not actionable |
| 10 | Incremental Build Pipeline | Valid chore ticket, not strategic initiative |
| 11 | tau doctor | Already 70% built in tau-diagnostics; single PR not roadmap item |
| 12 | Self-Rewriting AGENTS.md | Too narrow; one file's freshness doesn't warrant a project |

## Session Log
- 2026-03-24: Initial ideation — 40 raw ideas generated across 5 frames (pain/friction, missing capabilities, inversion/removal, assumption-breaking, leverage/compounding), ~28 unique after dedupe, 7 survivors after adversarial filtering
