# Tau — Review #37

**Date:** 2026-02-22
**origin/master HEAD:** `0e80a07b` (2,478 commits)
**Current branch:** `codex/issue-3298-mutation-baseline`
**Previous Review:** #36 (`3dbe1dc0`, 2,467 commits)

---

## 1. Scale Snapshot

| Metric | R#35 | R#36 | R#37 (now) | Delta (R36→R37) |
|--------|------|------|------------|-----------------|
| Commits | 2,296 | 2,467 | **2,478** | **+11** |
| Crates | 44 | 44 | **44** | — |
| Rust lines | 297,048 | 298,929 | **300,130** | **+1,201** |
| `.rs` files | 417 | 434 | **434** | — |
| Test functions (#[test]) | 2,989 | 3,027 | **3,029** | +2 |
| Async tests (#[tokio::test]) | 910 | 918 | **923** | +5 |
| **Total tests** | 3,899 | 3,945 | **3,952** | **+7** |
| Milestones | 209 | 249 | **251** | +2 |
| Spec files | 1,775 | 1,935 | **1,946** | +11 |
| Unique issues | 1,960 | 2,040 | **2,046** | +6 |
| `unsafe` | 3 | 3 | **3** | — |
| `.unwrap()` (prod) | 2 | 2 | **2** | — |
| `panic!` | 122 | 122 | **122** | — |
| `todo!()`/`unimplemented!()` | 0 | 0 | **0** | — |
| `#[allow]` | 1 | 1 | **1** | — |

**Tau crossed 300,000 lines of Rust.**

---

## 2. What Changed (11 Commits, 3 Merged PRs)

This is a focused, high-impact sprint. Only 11 commits but they close the last major self-improvement gap:

### 2.1 PR #3293 — Expanded Reward Inference Signals

`reward_inference.rs` grew from **152 → 192 lines** (+26%) with two new signal dimensions:

| Signal | What It Measures |
|--------|-----------------|
| `session_completion` | -0.25 if session didn't complete (user abandoned) |
| `token_efficiency` | Scores output/input char ratio: +0.25 if ≤1.0, -0.25 if ≥3.0, linear interpolation between |

The composite formula is now 7-dimensional:
```
composite = (completion + session_completion + reliability
           + efficiency + token_efficiency + safety).clamp(-1.0, 1.0)
```

Confidence is also penalized by 0.25 when `session_completed` is false.

### 2.2 PR #3297 — APO Live-Runtime Integration with Significance Gating

**This is the headline change.** `live_rl_runtime.rs` grew from **~900 → 2,070 lines** (+1,176 lines).

The APO (Automatic Prompt Optimization) algorithm is now **fully wired into the live agent loop**:

- Imports `ApoAlgorithm`, `ApoConfig`, `PromptEvaluator`, `PromptExample` from tau-algorithm
- Imports `LlmClient`, `ChatRequest`, `Message` for APO prompt evaluation via actual LLM calls
- Imports `compare_policy_improvement` from tau-trainer for statistical significance testing
- **160 APO-related references** in the file — this is deep integration, not a thin wrapper

The flow:
```
Live sessions produce reward-scored rollouts
  → After N rollouts, APO scheduler triggers
  → Beam search generates candidate prompts from current system prompt
  → LLM evaluates candidates against session trace data
  → compare_policy_improvement() gates adoption
  → Only statistically significant improvements are applied
  → Prompt mutation with rollback safety
```

**This closes Phase 3 from the Review #35 self-improvement roadmap.**

### 2.3 PR #3299 — Mutation Baseline Stabilization

Test alignment and APO regression hardening. Ensures the expanded reward inference API and APO integration don't break existing test baselines.

---

## 3. Real vs Scaffold

### 3.1 Markers

| Marker | Count |
|--------|-------|
| `unimplemented!()` | 0 |
| `todo!()` | 0 |
| Production mocks | 0 |
| Scaffold patterns | 0 |

### 3.2 Verdict

**100% real production code.** Same as R#36. Zero scaffold, zero deferred-work markers.

### 3.3 Property-Based Testing

`proptest` is now a dependency in **2 crates** (tau-coding-agent, tau-tools). This addresses the property-based testing gap tracked since Review #30.

---

## 4. Code Quality

All hygiene metrics unchanged from R#36:

| Metric | Value |
|--------|-------|
| `unsafe` | 3 |
| `.unwrap()` (prod) | 2 |
| `panic!` | 122 (policy documented) |
| `todo!()`/`unimplemented!()` | 0 |
| `#[allow]` | 1 |
| Compiler warnings | 0 |

Top 5 crates by size:

| Lines | Crate |
|-------|-------|
| 46,776 | tau-coding-agent |
| 29,817 | tau-gateway |
| 18,474 | tau-multi-channel |
| 16,429 | tau-tools |
| 15,842 | tau-onboarding |

---

## 5. Grade

| Dimension | R#35 | R#36 | R#37 | Notes |
|-----------|------|------|------|-------|
| Code quality | A+ | A+ | **A+** | Stable |
| Architecture | A+ | A+ | **A+** | Stable |
| Testing | A- | A- | **A-** | +7 tests, proptest adopted |
| Documentation | A- | A- | **A-** | Stable |
| Operational readiness | B+ | A- | **A-** | Stable |
| Feature completeness | A | A+ | **A+** | APO live integration closes last major gap |
| Engineering process | A+ | A+ | **A+** | Stable |
| **Self-improvement** | — | — | **A-** | New dimension — loop closed, curriculum missing |

**Overall: A+**

---

## 6. Self-Improvement — Full Status

### The Loop Is Closed (Phases 1–3 Complete)

All three phases identified in Review #35 are now operational:

```
DONE ✓  Phase 1: Intrinsic Reward Evaluation (R#36)
        ├── TraceBasedRewardInference (192 lines)
        ├── 7 signal dimensions (completion, session_completion,
        │   reliability, efficiency, token_efficiency, safety, confidence)
        ├── Wired into live_rl_runtime.rs
        └── PPO/GAE optimizer runs on scored rollouts

DONE ✓  Phase 2: Cross-Session Learning (R#36)
        ├── Cortex chat calls actual LLM with context pyramid
        ├── Bulletin runtime refreshes on heartbeat
        ├── Memory graph synthesis into bulletin
        └── Bulletin injected into future session system prompts

DONE ✓  Phase 3: Prompt Self-Optimization (R#37 — NEW)
        ├── APO beam search wired into live_rl_runtime.rs
        ├── LLM evaluates candidate prompts against session traces
        ├── compare_policy_improvement() significance gate
        ├── Only adopts statistically validated improvements
        └── 2,070-line live RL runtime with full APO integration

IN PROGRESS  Phase 4: Curriculum + Meta-Cognition (#3300 follow-up)
DONE ✓  Phase 5: OpenTelemetry Export
        ├── Prompt runtime OTel-compatible trace/metric export
        ├── Gateway cycle OTel-compatible trace/metric export
        ├── CLI/config propagation (`--otel-export-log`)
        └── Verified in #2616 (`crates/tau-runtime/src/observability_loggers_runtime.rs`,
            `crates/tau-gateway/src/gateway_runtime.rs`)
```

### What Tau Does Today — The Complete Self-Improvement Loop

```
┌──────────────────────────────────────────────────────────────┐
│                  TAU SELF-IMPROVEMENT LOOP                    │
│                                                              │
│  ┌─── AGENT SESSION ───────────────────────────────────────┐ │
│  │ Agent executes session with current system prompt       │ │
│  │ AgentEvents: start, message, tool, safety, end          │ │
│  └──────────────────────┬──────────────────────────────────┘ │
│                         ▼                                    │
│  ┌─── REWARD SCORING ──────────────────────────────────────┐ │
│  │ TraceBasedRewardInference scores 7 dimensions:          │ │
│  │ completion, session_completion, reliability,             │ │
│  │ efficiency, token_efficiency, safety, confidence         │ │
│  │ → composite ∈ [-1.0, 1.0]                               │ │
│  └──────────────────────┬──────────────────────────────────┘ │
│                         ▼                                    │
│  ┌─── PPO/GAE OPTIMIZER ──────────────────────────────────┐  │
│  │ Rollout persisted to SQLite                             │  │
│  │ On interval: collect succeeded rollouts                 │  │
│  │ → Compute GAE advantages + returns                      │  │
│  │ → Run PPO update (policy loss, value loss, entropy)     │  │
│  │ Gate: holds on consecutive failures                     │  │
│  └──────────────────────┬──────────────────────────────────┘  │
│                         ▼                                    │
│  ┌─── APO PROMPT OPTIMIZER ───────────────────────────────┐  │
│  │ After K rollouts: APO scheduler triggers                │  │
│  │ → Beam search: gradient, edit, score on candidates      │  │
│  │ → LLM evaluates candidates against session traces       │  │
│  │ → compare_policy_improvement() significance gate        │  │
│  │ → Only adopts statistically validated improvements      │  │
│  │ → System prompt mutated with rollback safety            │  │
│  └──────────────────────┬──────────────────────────────────┘  │
│                         ▼                                    │
│  ┌─── CROSS-SESSION SYNTHESIS ────────────────────────────┐  │
│  │ Cortex bulletin refreshes on heartbeat                  │  │
│  │ → LLM synthesizes cross-session knowledge               │  │
│  │ → Patterns injected into future system prompts          │  │
│  │ → Memory graph enriches session context                 │  │
│  └──────────────────────┬──────────────────────────────────┘  │
│                         ▼                                    │
│                  NEXT SESSION STARTS                          │
│            (with improved prompt + learned context)           │
└──────────────────────────────────────────────────────────────┘
```

### What's Still Needed (~1,100 lines)

| Phase | Description | Status | Lines Needed |
|-------|-------------|--------|-------------|
| 1 | Intrinsic rewards | **Done** | 0 |
| 2 | Cross-session synthesis | **Done** | 0 |
| 3 | Prompt self-optimization | **Done** | 0 |
| 4 | Curriculum + meta-cognition | In progress | ~1,100 |
| 5 | OpenTelemetry export | **Done** | 0 |

#### Phase 4: Curriculum + Meta-Cognition (~1,100 lines remaining)

The next frontier. With Phases 1–3 operational, the agent improves but doesn't focus its improvement. What's needed:

1. **Per-category success tracking** (**partially delivered**)
   - ✅ Classify sessions by task type in live decision spans (`task_category`)
   - ✅ Track category outcomes and identify weakest categories for APO focus
   - Remaining: richer category taxonomy + long-horizon aggregation storage

2. **Difficulty-weighted rollout scheduling** (**partially delivered**)
   - ✅ APO sample capping now prioritizes weak categories (`apo_curriculum_focus_category`)
   - Remaining: progressive difficulty scaling policy and scheduler controls

3. **Confidence calibration** (**partially delivered**)
   - ✅ Track predicted success, actual success, and calibration error per live decision
   - ✅ Emit `ask_for_help_recommended` using historical category success rate
   - Remaining: calibration curves and external reporting surfaces

4. **Learning progress monitoring** (**partially delivered**)
   - ✅ Emit per-category `learning_trend` (`improving|plateau|regressing|insufficient_data`)
   - Remaining: alerting + dashboard visualization of learning curves

#### Phase 5: OpenTelemetry (Done)

Delivered under #2616 as OpenTelemetry-compatible JSON export paths:
- Prompt/runtime traces + metrics (`crates/tau-runtime/src/observability_loggers_runtime.rs`)
- Gateway cycle traces + metrics (`crates/tau-gateway/src/gateway_runtime.rs`)
- CLI/config wiring (`--otel-export-log`) across startup/gateway pathways

---

## 7. Summary

### Review #37 Verdict

Tau is a **300K-line pure-Rust AI agent runtime** that now **autonomously improves its own prompts** through a closed feedback loop: trace-based reward scoring → PPO/GAE optimization → APO beam-search prompt mutation → significance-gated adoption → cross-session knowledge synthesis.

**What's real:** Everything. Zero stubs, zero `todo!()`, zero `unimplemented!()`, zero production mocks. 3,952 tests. 44 crates. 251 milestones. 2,046 tracked issues.

**What's new:** APO live integration (PR #3297) closes the prompt self-optimization loop. The agent can now score its own performance, optimize its own prompts via LLM-critiqued beam search, and only adopt changes that pass statistical significance testing. This is combined with cross-session bulletin synthesis that carries learned patterns forward.

**What's missing:** Remaining curriculum/meta-cognition depth (difficulty scaling, long-horizon calibration reporting, and dashboard/alert surfaces). OpenTelemetry export is already implemented and verified under #2616.

**Grade: A+** — with a closed self-improvement loop, Tau is no longer just an agent runtime. It's a runtime that gets better at being an agent runtime.

---

*Review #37 completed. Reviewed against origin/master `0e80a07b` (2,478 commits, 300,130 lines, 44 crates, 3,952 tests, 251 milestones).*
