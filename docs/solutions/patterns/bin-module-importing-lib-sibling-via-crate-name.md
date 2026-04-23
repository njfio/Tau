---
title: Bin-tree modules must import lib siblings by crate name, not `crate::`
category: patterns
date: '2026-04-23'
tags:
  - rust
  - cargo
  - bin-lib-split
  - module-resolution
  - lib-seam
  - E0432
related:
  - docs/solutions/patterns/bin-crate-shared-modules-via-lib-seam.md
---

# Bin-tree modules must import lib siblings by crate name, not `crate::`
## Problem
In a Cargo crate that ships both a library (`src/lib.rs`) and a binary (`src/main.rs`), modules declared inside the binary tree (e.g. `mod tools;` in `main.rs` → `src/tools.rs`) cannot reach modules declared in `src/lib.rs` via `crate::`. `crate::` inside `src/tools.rs` resolves to the bin crate root, where the lib's `pub mod self_modification_tool` does NOT exist. The compiler error is `E0432 unresolved import crate::self_modification_tool` with the helpful suggestion `a similar path exists: tau_coding_agent::self_modification_tool`.
## Root cause
A Cargo package with both `src/lib.rs` and `src/main.rs` produces two distinct crates from the compiler's perspective (one library crate, one or more binary crates). They share `Cargo.toml` and dependency graph but each has its own crate root. `crate::` in a bin-tree module resolves to the bin crate root; modules declared in `lib.rs` are siblings only inside the lib crate. To cross from bin into lib you must go through the public name of the lib crate (the package name with hyphens replaced by underscores), e.g. `tau_coding_agent::module`. This is invisible until you migrate a module from the bin tree to the lib tree (the "lib seam" refactor) and a bin-tree consumer tries to import it.
## Solution
Inside any module that lives in the bin tree (anything pulled in by `mod X;` from `src/main.rs` or `src/bin/*.rs`), import lib-promoted modules via the crate's package name with underscores: `use tau_coding_agent::self_modification_tool::SelfModificationProposeTool;`. NOT `use crate::self_modification_tool::...`. Conversely, modules inside the lib tree (`src/lib.rs` and its descendants) MUST use `crate::` to refer to lib siblings — using the crate name there is non-idiomatic.
## Prevention

When promoting a module from bin-tree to lib-tree (the "lib seam" refactor), grep the bin tree for `crate::<promoted_module>` and rewrite each to `<crate_name>::<promoted_module>`. A `cargo check -p <crate> --all-targets` after the rename catches the error instantly via E0432 with a `similar path exists` suggestion that names the correct fix. Consider a doc-comment near the lib seam declaration recording the import convention so future contributors don't try `crate::` first.
