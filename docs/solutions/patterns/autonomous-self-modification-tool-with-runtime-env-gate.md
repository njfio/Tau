---
title: Autonomous AgentTool wrapping a security-sensitive pipeline with visible-but-refused env gating
category: patterns
date: '2026-04-23'
tags:
  - agent-tools
  - autonomous-agent
  - env-gating
  - safety
  - tokio
  - test-patterns
  - self-modification
related:
  - docs/adrs/0001-self-modification-dry-run-pipeline.md
  - docs/solutions/patterns/self-modification-worktree-containment.md
  - docs/solutions/patterns/bin-crate-shared-modules-via-lib-seam.md
---

# Autonomous AgentTool wrapping a security-sensitive pipeline with visible-but-refused env gating
## Problem
The self-modification pipeline was operator-runnable via a standalone bin but had no autonomous call site. Wiring it into `tau-coding-agent`'s tool-dispatch path raised three concerns: (1) making destructive/irreversible tools silently available to the agent is unsafe, (2) hiding the tool when disabled makes the agent invent workarounds instead of observing a refusal, (3) tests that mutate process-wide env vars deadlock or tear when run in `#[tokio::test]` with a shared multi-threaded runtime.
## Root cause
Agent tool registration is a one-way door: once registered, the tool's schema is sent to the model every turn. Using registration-time gating means disabling requires a restart and makes the refusal invisible. Runtime gating via env var keeps the tool visible so the model sees a structured `reason_code` error and can adapt, and allows operators to flip the flag without redeploying. The test-deadlock is structural: `tokio::test` with `flavor = "multi_thread"` (default) runs in a shared runtime; a sync `Mutex` guard held across `.await` can park a worker thread that another test needs.
## Solution
Implement `AgentTool` where the `definition()` method always advertises the tool (stable schema), but `execute()` checks `TAU_AUTONOMOUS_SELF_MOD` at the top of the body:\n\n    if !Self::autonomous_enabled() {\n        return ToolExecutionResult::error(json!({\n            \"reason_code\": \"autonomous_self_mod_disabled\",\n            \"message\": \"set TAU_AUTONOMOUS_SELF_MOD=1 to enable dry-runs\",\n        }));\n    }\n\nThe env check is `std::env::var(...) == Some(\"1\") || == Some(\"true\")` — anything else is treated as off (including `\"0\"`, empty, missing). The tool delegates to the pipeline which itself is dry-run-only, giving defense-in-depth: even if the env gate is flipped, no mutation is possible without code changes.\n\nFor tests, avoid `#[tokio::test]` when the test body mutates `std::env`. Instead:\n\n    static ENV_LOCK: Mutex<()> = Mutex::new(());\n    fn run_with_env<F, Fut, R>(value: Option<&str>, body: F) -> R\n    where F: FnOnce() -> Fut, Fut: std::future::Future<Output = R>\n    {\n        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());\n        // set env ...\n        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();\n        let out = rt.block_on(body());\n        // restore env ...\n        out\n    }\n\nThis gives each test a fresh current-thread runtime, serializes env mutation via the guard, and releases the guard only after the future completes. `env::set_var` is `unsafe` on 2024 edition but safe here because access is gated by `ENV_LOCK` inside this test binary and no production code mutates this env var.
## Prevention

For any tool whose side effects are irreversible or security-sensitive, pair it with an explicit env/config enable flag (fail-closed) *and* keep the tool visible in the registry even when disabled so the agent observes a structured refusal rather than an unexpected absence. Visible-but-refused is teachable to the model; mysteriously-absent is not. Second rule: tests that mutate process-wide state (`std::env`, current dir) must serialize via a module-local `Mutex` and prefer a per-test tokio runtime over `#[tokio::test]` — holding a sync mutex across an `.await` in the shared test runtime causes parallelism deadlocks.
