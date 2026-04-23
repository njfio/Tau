---
category: patterns
slug: fallibility-audit-tau-coding-agent-tau-memory-2026-04
title: "Audit: no production unwrap/expect in tau-coding-agent or tau-memory (2026-04)"
tags: [audit, fallibility, unwrap, expect, panic, tau-coding-agent, tau-memory]
related: []
---

# Audit: no production `unwrap`/`expect` in tau-coding-agent or tau-memory

## Problem

The Gyre SE loop picked "audit top-200 `unwrap`/`expect` sites in tau-coding-agent
+ tau-memory" as the next hardening stage — the assumption being that the two
highest-stakes crates in the workspace (the autonomous coding agent and its
persistent memory store) would surface a meaningful panic surface deserving
conversion to `Result`-based error propagation. Panic-on-None / panic-on-Err in
autonomous paths is a particularly nasty footgun: an agent that panics mid-run
loses its working state, tool-call context, and any in-flight user conversation.

## Root cause

The assumption turned out to be wrong, in a good way: **the fallibility discipline
in both crates is already tight**. Ripgrep reports (2026-04, at HEAD `ecb221d0`):

- `tau-coding-agent/src` — 2142 matches for `unwrap(|\.expect(` total, but **100%
  of them sit below a `#[cfg(test)]` marker** (every file's test module starts at
  lines 40–1590, and every production section above that line matches zero).
- `tau-memory/src` — 316 matches, same story: every single one is in a
  `#[cfg(test)] mod tests { … }` block or a `runtime/tests/*.rs` submodule.

Panic-family macros (`panic!`, `todo!`, `unimplemented!`, `unreachable!`) surface
exactly one production site across both crates:

- `crates/tau-coding-agent/src/self_modification_synthesis_tool.rs:194` — an
  `impl Default for SelfModificationSynthesizeTool` whose body is
  `panic!("has no Default; construct via ::new(client, model)")`.

The comment above the impl explicitly reads: *"We do NOT implement Default to
force the call site to provide a client + model."* The impl block itself
contradicts that comment — it **does** implement Default, just panics if anyone
uses it. That is precisely the footgun the comment warned against: the type
satisfies the `Default` trait bound, so generic code that pulls `T: Default` will
compile and then panic at runtime instead of failing at the type system level.

## Solution

Two-part:

1. **Delete the panicking `Default` impl.** The comment's intent
   ("we do NOT implement Default") is now literally true. No workspace caller
   uses `SelfModificationSynthesizeTool::default()` — verified by ripgrep
   across `crates/`, `tests/`, and `examples/`. Single-line deletion at
   `crates/tau-coding-agent/src/self_modification_synthesis_tool.rs:188-196`.

2. **Record this audit as an active invariant**, not a one-off observation.
   Future changes to either crate that introduce `unwrap()` / `expect()` /
   `panic!` / `todo!` / `unimplemented!` / `unreachable!` outside a
   `#[cfg(test)]` block should be treated as a regression. The existing
   workspace lints configuration already denies `clippy::unwrap_used` and
   `clippy::expect_used` at `workspace = lints.clippy` — both crates inherit
   those. This audit confirms the policy is being followed in practice, not
   just declared.

Post-fix counts (2026-04, commit after this doc):
- Production `unwrap`/`expect`: **0 / 0** (both crates).
- Production `panic!`-family: **0 / 0** (both crates, after Default impl removal).

## Prevention

Three layers:

1. **Inherited workspace lints** — `Cargo.toml` declares
   `[workspace.lints.clippy] unwrap_used = "deny"` and `expect_used = "deny"`
   (where applicable). Both audited crates inherit via `lints.workspace = true`.

2. **Test files are the only exempted region**. The convention both crates
   follow is: all `unwrap()` / `expect()` sit inside `#[cfg(test)] mod tests { … }`
   blocks (or under `runtime/tests/` sibling modules for tau-memory). Production
   error paths go through `anyhow::Result` / `thiserror` / bespoke error enums.

3. **Recurring audit**. Re-run this rg sweep any time the workspace clippy gate
   is bypassed (e.g. `--allow` overrides), or any time new files are added to
   either crate:

   ```bash
   # Any hit here is a regression:
   rg 'unwrap\(|\.expect\(|\bpanic!|\btodo!|\bunimplemented!|\bunreachable!' \
     crates/tau-coding-agent/src crates/tau-memory/src \
     -g '!**/tests/**' -g '!**/tests.rs' | \
     awk -F: '{cmd="awk -v l="$2" \"NR==1,NR==l && /^#\\\\[cfg\\\\(test\\\\)\\\\]/ {exit 0} END{exit 1}\" "$1; if ((cmd | getline) > 0) next; print}'
   ```

   A simpler approximation: compare `rg -c` before and after a change; any
   increase in the non-test count is a regression to investigate.

## Reframing the stage

Because the audit target produced zero production sites, the remaining hardening
budget (originally planned for 200 `unwrap`-to-`?` conversions) is available for
other crates whose `unwrap_used` lint config may be looser, or for the broader
workspace panic-family scan (10+ crates with single-digit panic-family hits —
mostly `panic!` in `main.rs` startup code, which is the standard idiom for fatal
configuration errors).
