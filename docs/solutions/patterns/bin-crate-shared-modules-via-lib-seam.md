---
title: Sharing modules between main.rs and src/bin/*.rs via a minimal lib.rs seam
category: patterns
date: '2026-04-23'
tags:
  - rust
  - cargo
  - bin-target
  - module-system
  - dead-code
  - compilation-units
  - lib-seam
related:
  - docs/adrs/0001-self-modification-dry-run-pipeline.md
  - docs/solutions/patterns/self-modification-worktree-containment.md
---

# Sharing modules between main.rs and src/bin/*.rs via a minimal lib.rs seam
## Problem
A Cargo package whose top-level target is a binary (`src/main.rs`) needs to add an auxiliary binary under `src/bin/*.rs` that reuses code compiled into the main binary. Rust's compilation model gives each binary target its own independent crate root, so `mod foo;` declarations in `main.rs` are invisible to anything under `src/bin/`. Naive approaches — copy-pasting modules, declaring `mod foo;` in both, or `#[path = "../foo.rs"] mod foo;` — either duplicate compilation, fragment the module tree, or produce spurious `dead_code` warnings when one target uses the module and the other does not. In this repo that showed up wiring `self_modification_pipeline` into a standalone `self-mod-dry-run` operator binary: after adding the bin, `cargo clippy -- -D warnings` broke because `main.rs`'s compilation unit saw all of `self_modification_runtime::*` as unused, even though the new bin used them heavily.
## Root cause
Each Rust binary target is a separate crate root. `mod foo;` in one binary's entry file does not make the module visible to sibling binaries under `src/bin/`. Working around this with `#[path = "..."] mod foo;` in each bin technically compiles the file into every binary but: (a) multiplies compile time, (b) defeats cross-binary type identity if the modules expose types by value, and (c) creates confusing `dead_code` diagnostics because each compilation unit sees different subsets of the module as used. The `lib.rs` seam solves all three by giving every binary a single shared crate (the lib) to import from, so the module compiles once and visibility is evaluated against a stable, public API surface.
## Solution
Promote the shared modules to a minimal `src/lib.rs`. The package becomes a lib+bin package — Cargo handles this automatically with no manifest changes. Concretely:

1. Create `src/lib.rs` containing only `pub mod foo; pub mod bar;` for each shared module and a short doc comment explaining the seam.
2. Remove the corresponding `mod foo;` / `mod bar;` declarations from `src/main.rs`. Main.rs continues to own all binary-internal plumbing; only cross-binary surface migrates.
3. In the new `src/bin/*.rs`, `use <crate_name>::foo::...` to reach the shared code. No `#[path]` attributes, no duplication.
4. Integration tests under `tests/` can also now import from the crate — a free side-benefit.

The compilation unit for `lib.rs` owns the module, so `dead_code` is evaluated against the lib's surface (everything `pub`, so no warning). Both binaries link the same rlib — modules compile exactly once.

In this repo the seam is at `crates/tau-coding-agent/src/lib.rs`, which re-exports `self_modification_pipeline` and `self_modification_runtime`. The `self-mod-dry-run` bin at `crates/tau-coding-agent/src/bin/self_mod_dry_run.rs` uses `tau_coding_agent::self_modification_pipeline::run_dry_run_pipeline`. 37 self_modification tests, clippy `-D warnings`, and three adversarial smoke invocations all pass.
## Prevention

Heuristic: the moment a file lives in `src/bin/` and needs *any* symbol currently defined under `src/` (non-bin), add or reuse a `src/lib.rs`. Do not reach for `#[path]` unless the file is genuinely single-binary-local (test fixtures, bin-specific helpers). Lint rule / review checklist: flag new `#[path = "..."] mod` attributes in `src/bin/` — they are almost always the symptom of a missing lib seam. When promoting modules to lib.rs, keep the lib surface minimal (only what crosses the bin boundary); binary-internal modules stay in main.rs's tree so the lib API does not balloon.
