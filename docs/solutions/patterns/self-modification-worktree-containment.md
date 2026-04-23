---
title: Containment + input validation for self-modification worktree helpers
category: patterns
date: '2026-04-23'
tags:
  - self-modification
  - agent-safety
  - path-traversal
  - input-validation
  - containment
  - tracing
  - rust
  - tau-coding-agent
---

# Containment + input validation for self-modification worktree helpers
## Problem
`create_self_mod_worktree` and `cleanup_self_mod_worktree` in tau-coding-agent's `self_modification_runtime` module took arbitrary inputs without validation. `proposal_id` was passed straight into `PathBuf::join`, so a hostile string like `"../../escape"` could write the worktree outside the intended `.tau/self-mod-worktrees/` root. `cleanup` accepted any `&Path` — the single most dangerous capability for a self-improving agent, because a mis-routed call could `remove_dir_all` on the workspace root or worse. The classifier also used loose `path.contains("prompt")` substring matches that would flip category on directory names that happen to contain those substrings, producing incorrect auto-apply decisions.
## Root cause
The helpers were written as if all call sites would be internal and well-behaved, but in a self-improving loop the "caller" is an LLM-produced proposal — any field in that proposal must be treated as untrusted input. The classifier used `str::contains` semantics when the invariant it actually wanted was "a `prompts/` or `templates/` *segment* exists" — substring versus segment semantics are easy to conflate in Rust where `Path` is thin over `&str`.
## Solution
1. Add `validate_proposal_id` with an explicit ASCII allow-list ([A-Za-z0-9._-]), plus defence-in-depth rejection of `..`, path separators, and control characters. Call it at the top of `create_self_mod_worktree`. 2. Change `cleanup_self_mod_worktree` to take `workspace_root` and enforce that the passed path resolves (via `canonicalize`, with a fallback for not-yet-existing paths that canonicalises the parent) to a strict descendant of `<workspace_root>/.tau/self-mod-worktrees/`. Refuse the root itself and anything outside, return `InvalidInput`. 3. Switch the classifier to segment-based matching with separator normalisation and filename-stem checks for prompt/template assets. 4. Add `tracing` events at every critical transition (proposal id validation, worktree created, cleanup refused, cleanup done). 5. Add adversarial unit tests: hostile proposal ids, paths outside containment, the workspace root itself, symlink-resolved tempdirs on macOS, Windows-style separators, substring-collision directory names.
## Prevention

Treat every field of a self-modification proposal as untrusted input. For any helper that takes a path-like identifier, validate with an explicit allow-list before joining it to a filesystem path. For any helper that performs destructive filesystem ops, take the containment root as an explicit parameter and verify (after canonicalisation, with a parent-canonicalisation fallback for absent targets) that the target is a strict descendant. Prefer segment-based path classification (`.split('/').collect()` then `.contains(&name)`) over substring matching when the semantic invariant is "has this path component." On macOS, remember that `/tmp`, `/var/folders` resolve to `/private/...` — any containment check that canonicalises one side must canonicalise (or fall back correctly for) both.
