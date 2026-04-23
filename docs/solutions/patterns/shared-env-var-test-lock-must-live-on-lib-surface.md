---
title: Env-var test locks must live on the lib crate surface, not per-module
category: patterns
date: '2026-04-23'
tags:
  - rust
  - testing
  - env-vars
  - test-parallelism
  - race-condition
  - mutex-scope
related:
  - docs/solutions/patterns/autonomous-self-modification-tool-with-runtime-env-gate.md
---

# Env-var test locks must live on the lib crate surface, not per-module
## Problem
Two test modules in the same Rust test binary each defined their own `static ENV_LOCK: Mutex<()> = Mutex::new(())` intended to serialize mutation of the `TAU_AUTONOMOUS_SELF_MOD` environment variable. Tests in isolation passed. When both modules' tests ran together, one module's test would observe an env var value set by a concurrent test in the other module, causing `execute_refuses_when_env_gate_is_off` to fail with the env gate set to `"1"` from a parallel synthesis test. Serializing with `--test-threads=1` made them pass, confirming a data race on the process-global env var.
## Root cause
`static FOO: Mutex<()>` declared inside `mod a::tests` and `static FOO: Mutex<()>` declared inside `mod b::tests` are TWO distinct statics — each module has its own. They do not serialize anything shared between them. Environment variables, on the other hand, are process-global. Cargo runs test functions on a thread pool by default, so test functions from different modules can execute concurrently; each holds its own module-local lock while touching the same process-global env var. Classic lock-granularity mismatch: the thing being protected (env var) has wider scope than the lock protecting it (module-local static).
## Solution
Move the lock to the shared crate root under `#[cfg(test)]` so every module can reference the same instance:\n\n```rust\n// crates/<crate>/src/lib.rs\n#[cfg(test)]\npub(crate) static AUTONOMOUS_SELF_MOD_ENV_LOCK: std::sync::Mutex<()> =\n    std::sync::Mutex::new(());\n```\n\nThen in each test module:\n\n```rust\n#[cfg(test)]\nmod tests {\n    use crate::AUTONOMOUS_SELF_MOD_ENV_LOCK as ENV_LOCK;\n    // ... existing with_env helper unchanged ...\n}\n```\n\nOne static, many modules — serialization now spans the full test binary. Alternative: use `serial_test::serial` attribute from the `serial_test` crate, which achieves the same effect without a hand-rolled mutex but adds a dev-dependency.
## Prevention

When multiple test modules in the same test binary each need to mutate the same environment variable, declare ONE shared lock on the lib crate surface (`pub(crate) static NAME: std::sync::Mutex<()> = std::sync::Mutex::new(());`) under `#[cfg(test)]`, and have every test module import it with `use crate::NAME as ENV_LOCK`. Module-local `static ENV_LOCK: Mutex<()>` declarations LOOK correct but each is a distinct instance, so they do not serialize across modules — cargo's default parallel test runner will happily interleave them. Symptom: a test that's green in isolation fails intermittently when a sibling module is added.
