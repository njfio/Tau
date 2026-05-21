# Tau: Gaps, Issues & Improvements (Review #31)

**Date:** 2026-05-21
**HEAD:** `e2c3f686` plus current Agent Canvas v2 working-tree follow-up (44+ workspace crates)
**Roadmap closure:** foundational roadmap closed; active follow-through is now UI/runtime proof depth, under-tested QA surfaces, and render-shell modularity

This document supersedes the older Review #31 snapshot and refreshes closure
status/evidence against current repository artifacts, ADRs, runbooks, and the
ops-chat Agent Canvas v2 follow-up.

---

## Table of Contents

1. [Previous Roadmap - Closure Status](#1-previous-roadmap---closure-status)
2. [Remaining Gaps](#2-remaining-gaps)
3. [Stubs & Foundation Code](#3-stubs--foundation-code)
4. [Testing Gaps](#4-testing-gaps)
5. [Architecture Concerns](#5-architecture-concerns)
6. [Repository Hygiene](#6-repository-hygiene)
7. [Security](#7-security)
8. [Documentation & Operational Readiness](#8-documentation--operational-readiness)
9. [Performance & Scalability](#9-performance--scalability)
10. [Prioritized Action Items](#10-prioritized-action-items)

---

## 1. Previous Roadmap - Closure Status

| # | Item | Previous Status | Current Status | Evidence |
|---|------|----------------|----------------|----------|
| 1 | Harden tau-safety tests | Done | **Done** | `crates/tau-safety/src/lib.rs` contains 40 tests |
| 2 | Fix compiler warnings | Done | **Done** | `cargo check -q` passes at HEAD |
| 3 | Add `.env.example` | Done | **Done** | `.env.example` exists |
| 4 | Audit log sanitization | Partial | **Done** | `crates/tau-runtime/src/observability_loggers_runtime.rs` includes `spec_2612_*` redaction tests |
| 5 | Integration test suite | Open | **Done** | `tests/integration/tests/agent_tool_memory_roundtrip.rs` exists (4 integration tests) |
| 6 | Expand under-tested crates | Partial | **Partial** | Depth increased in recent waves (`tau-training-proxy` 14, `kamn-core` 12); remaining lower-depth target is `kamn-sdk` (8) plus adjacent QA surfaces |
| 7 | Add CHANGELOG.md | Done | **Done** | `CHANGELOG.md` exists |
| 8 | cargo-deny / cargo-audit | Done | **Done** | `deny.toml` + `.github/workflows/security.yml` |
| 9 | Clean stale branches | Open | **Done** | `scripts/dev/stale-merged-branch-prune.sh` exists; remote heads reduced (current: 380) |
| 10 | Add rustfmt.toml | Done | **Done** | `rustfmt.toml` exists |
| 11 | Discord outbound (G10) | Done | **Done** | Mention normalization/chunking paths are present in runtime split modules |
| 12 | Encrypted secrets (G20) | Partial | **Done** | `crates/tau-provider/src/credential_store.rs` with migration and redaction wrappers |
| 13 | Provider failover | Done | **Done** | Fallback routing + circuit-breaker support in provider runtime |
| 14 | Provider rate limiting | Partial | **Done** | `crates/tau-provider/src/client.rs` token-bucket limiter + `spec_2611_*` tests |
| 15 | SQLite memory backend | Done | **Done** | `crates/tau-memory/src/runtime/backend.rs` |
| 16 | Dashboard (G18) | Done | **Done** | Tau ops/dashboard shells and endpoints are shipped under gateway runtime modules |
| 17 | Wire RL training loop | Done | **Done** | Observation/report loop remains implemented (`crates/tau-coding-agent/src/live_rl_runtime.rs`) |
| 18 | OpenTelemetry | Open | **Done** | `crates/tau-runtime/src/observability_loggers_runtime.rs` + `crates/tau-gateway/src/gateway_runtime.rs` OTel export records |
| 19 | Graph visualization (G19) | Open | **Done** | `/api/memories/graph` handlers + memory graph tests in gateway runtime |
| 20 | Multi-process (G1) | Open | **Done** | `crates/tau-agent-core/src/process_types.rs` (`ProcessType`, `ProcessManager`) |
| 21 | External coding agent (G21) | Open | **Done** | `tau-runtime` bridge module + gateway external-agent runtime endpoints |
| 22 | Browser automation | Done | **Done** | Browser automation runtime path remains integrated |
| 23 | Fuzz testing | Done | **Done** | `fuzz/fuzz_targets/` harnesses present |

**Summary:** 22/23 done, 1 partial, 0 open.

---

## 2. Remaining Gaps

### 2.1 Remaining Functional Gaps

1. **Under-tested crate wave follow-through**
   The original expansion issue closed and recent waves raised direct crate-local depth in `tau-training-proxy` and `kamn-core`; remaining lower-depth target is `kamn-sdk` plus adjacent QA surfaces.

2. **Agent Canvas v2 product proof**
   `/ops/chat` can execute tools and preview generated HTML artifacts. The
   active follow-up is making that preview operationally useful: artifact
   history, frame diagnostics, console/error visibility, pixel samples, and
   controlled click/type/probe commands.

3. **Under-tested crate wave follow-through**
   `docs/guides/test-coverage-targets.md` now defines target thresholds and
   conformance mapping. Remaining work is execution against `kamn-sdk` and
   adjacent QA surfaces, not policy definition.

### 2.2 M104 Follow-up Issues (Current State)

| Item | Issue | State |
|------|-------|-------|
| Integration test suite bootstrap | #2608 | **Closed** |
| Under-tested crate expansion wave | #2609 | **Closed** |
| Branch hygiene stale cleanup | #2610 | **Closed** |
| Provider-layer token-bucket rate limiting | #2611 | **Closed** |
| Log sanitization audit formalization | #2612 | **Closed** |
| Encrypted secret migration completion | #2613 | **Closed** |
| OpenTelemetry export | #2616 | **Closed** |
| Memory graph visualization (G19) | #2617 | **Closed** |
| Multi-process architecture staging (G1) | #2618 | **Closed** |
| External coding-agent bridge protocol | #2619 | **Closed** |

---

## 3. Stubs & Foundation Code

| Component | Location | Current State | Remaining Stub Surface |
|-----------|----------|---------------|------------------------|
| Deploy endpoint | `crates/tau-gateway/src/gateway_openresponses/deploy_runtime.rs` + `deploy_process_supervisor.rs` | Request/stop state is persisted and process supervisor can spawn configured child processes | Needs release-grade live drill coverage across configured process profiles |
| Stop endpoint | `crates/tau-gateway/src/gateway_openresponses/deploy_runtime.rs` + `deploy_process_supervisor.rs` | Agent stop routes through process termination and records stop evidence | Needs broader concurrency/race regression coverage |
| RL weight updates | `crates/tau-coding-agent/src/live_rl_runtime.rs` | Captures rollouts and emits optimization reports | Does not write updated model weights (by design) |

---

## 4. Testing Gaps

### 4.1 Under-Tested Areas (Current Snapshot)

| Area | Current Signal | Recommendation |
|------|----------------|----------------|
| Integration breadth | `tests/integration/` contains one file with 4 tests plus extensive crate-local gateway/dashboard route coverage | Expand scenario count for channel routing/compaction/delegation |
| tau-diagnostics | 11 direct test markers | Continue edge-case coverage for audit aggregation and telemetry compatibility |
| tau-training-proxy | 14 direct test markers | Continue transport/read failure and persistence boundary coverage |
| kamn-core | 12 direct test markers | Continue identity/auth malformed-input and boundary hardening coverage |
| kamn-sdk | 8 direct test markers | Add contract and integration fixture coverage for SDK call paths |

### 4.2 Missing/Light Categories

| Category | Current State | Recommendation |
|----------|---------------|----------------|
| Property-based tests | Minimal usage in core roadmap surfaces | Add `proptest` for ranking/decay/token-limit math |
| Concurrency stress | Targeted coverage exists but thin for some paths | Add race-oriented tests around memory writes and process supervision |
| Deploy runtime integration | Process supervisor tests and shell/browser route evidence exist | Add repeatable release drill for spawn/terminate/restart races |
| Ops chat canvas | Live manual proof existed before this update | Keep `scripts/dev/ops-chat-canvas-proof.sh` green as the repeatable product proof |

---

## 5. Architecture Concerns

### 5.1 Gateway Module Size (Improved, Still a Hotspot)

`crates/tau-gateway/src/gateway_openresponses.rs` is now 267 lines after route/runtime extraction. The pressure point has moved to render surfaces:

- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`: ~6.3k lines
- `crates/tau-dashboard-ui/src/lib.rs`: ~13.3k lines

Continued modularization should now split chat/canvas, deploy, harness, and
memory UI/render contracts into focused modules rather than growing the central
shell files.

### 5.2 Single-Binary Runtime Limits

The current process model supports branch/worker process semantics in one runtime, but crash/resource isolation remains bounded by a single process model.

### 5.3 Dashboard Stack Direction

Dashboard stack direction is decided in `docs/architecture/adr-006-dashboard-ui-stack.md`:
new operator dashboard route work standardizes on Leptos SSR in
`tau-dashboard-ui`, while older gateway-owned HTML/JS surfaces remain
compatibility bridges.

---

## 6. Repository Hygiene

| Item | Status | Evidence |
|------|--------|----------|
| Stale branch hygiene | **Improved** | `scripts/dev/stale-merged-branch-prune.sh`; remote branch count currently 380 |
| Dependabot backlog | **Open** | Open dependency PRs #2710-#2714 |
| CONTRIBUTING.md | **Done** | `CONTRIBUTING.md` tracked at repository root |
| SECURITY.md | **Done** | `SECURITY.md` tracked at repository root |

---

## 7. Security

| Item | Status | Evidence |
|------|--------|----------|
| Encrypted credential store | **Done** | `crates/tau-provider/src/credential_store.rs` |
| Decrypted secret redaction wrappers | **Done** | `DecryptedSecret` `Debug/Display` emit `[REDACTED]` |
| SSRF protections | **Done** | Existing gateway safety guards remain in tree |
| Secret-leak detection controls | **Done** | `crates/tau-safety/src/lib.rs` pattern + policy controls |
| Log sanitization audit formalization | **Done** | `spec_2612_*` coverage in observability logger runtime |
| Key rotation CLI | **Done** | `/integration-auth rotate` command contract in `crates/tau-provider/src/integration_auth.rs` |

---

## 8. Documentation & Operational Readiness

| Item | Status | Evidence |
|------|--------|----------|
| Operator deployment guide | **Done** | `docs/guides/operator-deployment-guide.md` |
| API reference | **Done** | `docs/guides/gateway-api-reference.md` |
| Deployment ops guide | **Done** | `docs/guides/deployment-ops.md` |
| Runbook ownership map | **Done** | `docs/guides/runbook-ownership-map.md` |
| Architecture ADR trail | **Done** | `docs/architecture/adr-00*.md` set |
| High-level dependency graph doc | **Done** | `docs/architecture/crate-dependency-diagram.md` published |
| Cortex automation scope | **Done** | `docs/architecture/cortex-automation-scope.md` keeps Cortex advisory-only until typed action-envelope gates exist |
| Coverage target policy | **Done** | `docs/guides/test-coverage-targets.md` defines crate/path thresholds and AC-to-conformance mapping |
| Dashboard stack ADR | **Done** | `docs/architecture/adr-006-dashboard-ui-stack.md` selects Leptos SSR for new operator route work |
| Key rotation runbook | **Done** | `docs/guides/key-rotation-operator-runbook.md` covers encrypted credential store rotation, verification, rollback, and release evidence |

---

## 9. Performance & Scalability

| Concern | Current State | Recommendation |
|---------|---------------|----------------|
| Gateway runtime hotspot | Root module reduced to ~267 lines; render shell files are now the hotspot | Extract chat/canvas and deploy render/runtime contracts |
| FileMemoryStore query cost | SQLite backend exists; file-backed mode still linear scans | Prefer SQLite for larger deployments |
| Memory graph rendering scale | Works for current dashboard shell usage | Add heavier-load profiling if node count targets increase |
| Provider burst control | Provider-layer token bucket shipped | Add operational dashboards for saturation visibility |

---

## 10. Prioritized Action Items

### P0 - Next High-Impact Closures

1. **Ship Agent Canvas v2 proof**: artifact history, diagnostics bridge,
   controlled preview interactions, pixel/console signals, and repeatable
   `/ops/chat` proof automation.
2. **Run full release-grade validation** for the branch, including full
   `cargo test` after targeted checks and live proof.

### P1 - Quality and Maintainability

3. **Extract chat/canvas and deploy render modules** from
   `tau-dashboard-ui/src/lib.rs` and
   `gateway_openresponses/ops_dashboard_shell.rs`.
4. **Expand under-tested crate coverage** against
   `docs/guides/test-coverage-targets.md`, starting with `kamn-sdk` and adjacent
   QA surfaces.
5. **Grow property/concurrency testing** for ranking, compaction, memory writes,
   and process supervision.
6. **Sustain contributor/security doc freshness** with release-aligned review cadence.

### P2 - Medium-Term Enhancements

7. **Run key-rotation drills** against the encrypted credential store runbook
   and record sanitized release evidence.
8. **Reconsider Cortex supervisor routing only behind the typed action-envelope
   escalation gate** in `docs/architecture/cortex-automation-scope.md`.
9. **Add heavier-load dashboard profiling** once memory graph and canvas usage
   targets increase.

---

## Summary

This refresh removes stale action items that are now covered by ADRs, runbooks,
or published docs. The main remaining gaps are product-proof depth for Agent
Canvas, full branch validation, UI/render modularization, and targeted QA depth
in lower-covered crates.
