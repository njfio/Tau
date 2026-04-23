---
category: patterns
slug: fallibility-audit-workspace-2026-04
title: "Audit: workspace-wide production unwrap/expect/panic-family sweep (2026-04)"
tags: [audit, fallibility, unwrap, expect, panic, workspace, dead-code, tech-debt]
related:
  - docs/solutions/patterns/fallibility-audit-tau-coding-agent-tau-memory-2026-04.md
---

# Workspace-wide fallibility audit

## Problem

Following the zero-hit result in the tau-coding-agent + tau-memory audit,
the Gyre SE loop extended the sweep to the entire workspace (~47 crates).
The question: **how many production `unwrap()` / `expect()` / panic-family
sites exist outside the two crates already audited, and which deserve
conversion to `Result` propagation?**

## Root cause (audit methodology)

Ripgrep across `crates/**/src` excluding `tests/`, `tests.rs`, `benches/`,
`examples/`, and body-after-`#[cfg(test)]` markers. Per-file heuristic:

```bash
for f in $(find crates -name "*.rs" -type f); do
  case "$f" in */tests/*|*/tests.rs|*/benches/*|*/examples/*) continue ;; esac
  cfg=$(rg -n '^#\[cfg\(test\)\]' "$f" | head -1 | cut -d: -f1)
  body=${cfg:+$(awk -v c="$cfg" 'NR<c' "$f")}; body=${body:-$(cat "$f")}
  n=$(printf '%s' "$body" | rg -c 'unwrap\(|\.expect\(')
  [ "$n" -gt 0 ] && echo "$n $f"
done
```

## Solution (audit results & prioritized fix list)

**Raw result: 69 sites across 6 crates.** After classification, only ~15 are
*actual* production sites; the other 54 split into:

- 17 sites in **four orphaned files** in `tau-tui/src/interactive/` that no
  `mod` declaration references — dead code that the compiler never sees.
- 38 sites in `*_tests.rs` submodules that *are* wired in via
  `#[cfg(test)] mod X;` in the parent `mod.rs` but don't carry
  `#[cfg(test)]` at the top of their own file, so the naive heuristic
  counted them as production.

### Category A — Dead code (orphaned files in tau-tui)  **PRIORITY 1**

Four files, 249 lines total, zero `mod` references anywhere:

| File | Lines | Contents |
|---|---|---|
| `crates/tau-tui/src/interactive/session_state.rs` | prod | `InteractiveSessionState` type + `load/save_interactive_session_state()` JSON persistence |
| `crates/tau-tui/src/interactive/session_state_tests.rs` | 125 tests | builds `App` with `local_state_path`, asserts load/save roundtrip |
| `crates/tau-tui/src/interactive/transcript_state.rs` | prod | `InteractiveTranscriptState` + load/save |
| `crates/tau-tui/src/interactive/transcript_state_tests.rs` | 124 tests | same pattern for transcripts |

`git log` traces them to commit `8926bd4a` "feat(provider): codex app-server
WebSocket integration" — completely unrelated to their content. Classic
orphaned-during-merge scenario. The production code they reference
(`App::new(AppConfig { local_state_path, local_transcript_path, .. })`) still
has those fields on `AppConfig`, but the load/save functions are never called
anywhere outside the dead test files themselves.

**Decision (pending user confirmation):** wire them back into `mod.rs` AND add
the load/save calls to the appropriate App lifecycle hooks, OR delete all
four files. Unknown without product intent. Surfaced to `.gyre/tech-debt.md`
for the next stage.

### Category B — Legitimate infallible-by-construction `expect()` **PRIORITY 3 (acceptable)**

These are idiomatic Rust and do not warrant conversion:

| Site | Reason |
|---|---|
| `tau-safety/src/audit.rs:148,154,160,166,174` | `Regex::new(const_literal).expect("valid X regex")` — compile-time regex |
| `tau-safety/src/lib.rs:669,674,747` | same pattern, `Regex::new(rule.pattern)` where patterns are crate-const |
| `tau-provider/src/client.rs:106,147` | `Mutex::lock().expect("lock poisoned")` — panic on poisoning is **correct** for rate-limiter state |
| `tau-provider/src/codex_appserver_client.rs:320` | `guard.as_mut().expect("connection just established")` — invariant 2 lines up |
| `tau-orchestrator/src/multi_agent_router.rs:474` | `candidates.iter().find(...).expect("selected role should exist")` — selector result from own candidate list |
| `tau-trainer/src/bin/rl_e2e_harness.rs:112` | `serde_json::to_string_pretty(&artifact).expect(...)` — startup fatal in `main.rs`-adjacent binary |

These could all be upgraded to `LazyLock<Regex>` / `OnceLock` / `?`-propagation
with log-then-exit in the bin, but the cost/benefit is poor: 10 sites, each
individually fine, conversion churn high, risk of regressions non-zero.
**Recommendation: leave as-is**, optionally hoist the regex sites into
`LazyLock` for startup cost (cosmetic).

### Category C — One borderline site **PRIORITY 2**

`tau-training-runner/src/lib.rs:432`:
```rust
.expect("invalid runner config: retry-backoff settings");
```

This is in production config-load code (not in a cfg(test) block). A
malformed config would panic the runner at startup instead of surfacing a
user-readable error. Worth converting to `anyhow::bail!` or `?`.
Not critical (startup-only path), but a clear improvement.

## Prevention

1. **Fix Category A first.** Dead tests are negative-value code — they won't
   catch regressions (they never run), but they mislead audits. Either wire
   them up or delete them.
2. **Add a `cargo deny` / `cargo machete` check** to catch orphaned modules at
   CI time. Alternative: a periodic `rg -L` + `mod ` sweep.
3. **Category B sites are fine.** The `clippy::unwrap_used` deny at workspace
   level has an explicit `clippy::expect_used = "allow"` intentional carve-out
   for infallible-by-construction expects (idiomatic regex compilation, lock
   poisoning). The audit confirms that carve-out is used narrowly.
4. **Fix Category C** in a separate small deliverable: convert line 432 to
   `Result` propagation + structured error.

## Reframing for next stage

The prioritized fix list is:

1. **Decide on tau-tui orphaned files** (wire up or delete) — requires product
   intent, so askQuestions-worthy.
2. **Fix `tau-training-runner/src/lib.rs:432`** — mechanical, small, can
   happen any time.
3. (Optional) Hoist `Regex::new(...)` calls in tau-safety into `LazyLock` for
   one-time startup cost instead of per-call.

Items 1 and 2 are the real follow-ups. The workspace is in much better shape
than the audit's framing presumed.
