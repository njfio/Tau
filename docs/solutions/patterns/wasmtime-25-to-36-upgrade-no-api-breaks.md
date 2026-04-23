---
title: Upgrading wasmtime 25 → 36 on the core Config/Engine/Module/Store surface is API-compatible
category: patterns
date: '2026-04-23'
tags:
  - wasmtime
  - cve
  - rust
  - cargo
  - msrv
  - cranelift
  - rustsec-2026-0096
  - dependency-upgrade
---

# Upgrading wasmtime 25 → 36 on the core Config/Engine/Module/Store surface is API-compatible
## Problem
RUSTSEC-2026-0096 (CVSS 9.0, aarch64 guest heap miscompile) affects all wasmtime <36. Workspace pinned wasmtime 25.0.3 as a direct dep through tau-runtime's wasm sandbox. Upgrade path unclear: the jump spans 11 major versions and the cranelift/wasmparser/wast/wit-parser lockstep dependencies introduce many moving parts. Attempting wasmtime 44 (latest) failed the environment's rustc 1.90 because cranelift 0.131 requires rustc 1.92.
## Root cause
The workspace's wasmtime usage is narrow: `Config::new() + config.consume_fuel(true) + Engine::new(&config) + Module::new + Store::new + StoreLimitsBuilder + Linker + Memory` — a stable subset that has been API-compatible from wasmtime 15 through at least 44. The internal reorganization in wasmtime 44 (moving impl crates under `wasmtime-internal-*` and bumping cranelift MSRV to 1.92) is invisible at this surface. Pinning at wasmtime 36 (the minimum advisory fix) clears the CVE without any MSRV bump, because wasmtime 36-era cranelift still supports rustc 1.90.
## Solution
1. Edit workspace `Cargo.toml` only: `wasmtime = "36"` and `wasmparser = "0.232"` (the wasmparser pairing shipped with wasmtime 36-era crates). No per-crate Cargo.toml edits needed when the dep is a workspace dep.\n2. `cargo update` to refresh the lockfile. Multiple wasmparser versions may coexist transitively (tau-coding-agent uses wasmparser directly for module validation, wasmtime pulls its own pinned copy) — that is fine; cargo resolves them independently.\n3. `cargo check -p tau-runtime` — 2m cold build. On this codebase: zero compile errors.\n4. `cargo test -p tau-runtime` — 156/156 tests pass including the wasm sandbox suite.\n5. `cargo clippy --all-targets --no-deps -- -D warnings` — clean.\n6. `cargo audit` — RUSTSEC-2026-0096 gone.\n\nIf going to wasmtime 44+ later: needs rustc 1.92+ toolchain bump first. That is a separate rust-version-bump stage, not part of the CVE fix.
## Prevention

Treat wasmtime major-version bumps as low-risk **when** the codebase sticks to the stable `Config/Engine/Module/Store/Linker/Memory` surface. Heuristic: before any wasmtime bump, grep for `use wasmtime::{...}` — if the import list is narrow (≤ 10 symbols, all from the module root, no Component Model or WASI internals), the bump is typically a Cargo.toml edit + `cargo update`. Grep-and-count before committing to a multi-stage upgrade plan. Check the advisory's `patched_versions` before picking a target — avoid jumping to "latest" when the minimum patched version also clears the CVE and avoids MSRV ripples.
