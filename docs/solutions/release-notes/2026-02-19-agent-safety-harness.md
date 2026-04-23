---
title: 'Release notes: agent-safety-harness milestone (2026-02-19)'
category: release-notes
date: '2026-04-23'
tags:
  - release-notes
  - self-modification
  - wasmtime
  - security
  - operator-tooling
related:
  - docs/adrs/0001-self-modification-dry-run-pipeline.md
  - docs/solutions/patterns/self-modification-worktree-containment.md
  - docs/solutions/patterns/wasmtime-25-to-36-upgrade-no-api-breaks.md
  - docs/solutions/patterns/bin-crate-shared-modules-via-lib-seam.md
---

# Release notes: agent-safety-harness milestone (2026-02-19)
## Problem
Three interlocking gaps in the self-improving agent harness shipped as one milestone: (1) `self_modification_runtime` module existed with production-grade path-containment primitives but had zero callers and was allow-listed for dead_code, (2) operator-facing tooling could not exercise the dry-run pipeline end-to-end without writing custom glue, (3) workspace pinned wasmtime 25.0.3 which is affected by RUSTSEC-2026-0096 (CVSS 9.0, aarch64 guest heap miscompile).
## Root cause
The self-improving harness had been written speculatively without a call site, so dead_code masking hid both missing integration and missing observability. `tracing` was not wired into any of the safety-critical paths (agent bus message drops, worktree lifecycle). The wasmtime pin dated from an earlier milestone and had not been re-audited against current advisories.
## Solution
**New capability** — `self-mod-dry-run` binary (in `tau-coding-agent`). Operators can now run:\n\n    cargo run -p tau-coding-agent --bin self_mod_dry_run -- --target <path> [--workspace-root <dir>] [--proposal-id <id>]\n\nIt prints a JSON `SelfModificationResult` to stdout documenting policy verdict, worktree path, and safety-evaluation detail. Exit code is non-zero on validation rejection; zero on successful pipeline run (including policy-denied cases). `tracing` is silent by default; set `TAU_SELF_MOD_LOG=debug` to see step-by-step events on stderr.\n\n**Hardening** — `cleanup_self_mod_worktree` now canonicalizes with a parent-fallback for absent paths, refuses anything outside `<workspace>/.tau/self-mod-worktrees/`, and validates proposal_id against `[A-Za-z0-9._-]` with `..` rejection. `classify_modification_target` is segment-based with Windows separator normalization. 11 new adversarial tests cover symlink evasion (`/tmp` → `/private/tmp` on macOS), substring collision, Windows-style paths, and directory traversal.\n\n**Security** — wasmtime upgraded 25.0.3 → 36 clearing RUSTSEC-2026-0096. No API breaks because the workspace sticks to the stable `Config/Engine/Module/Store/Linker/Memory` surface.\n\n**Contract lock** — three-case integration test exercises the operator bin via assert_cmd so future edits cannot silently regress the argv → JSON contract.
## Prevention

Three habits this milestone encodes: (1) never ship a module behind `#![allow(dead_code)]` — if it's not called, either wire it or delete it; (2) every safety-critical lifecycle operation emits `tracing` events at the same granularity the test suite asserts on; (3) every minor release re-runs `cargo audit` and blocks on new advisories against direct deps.
