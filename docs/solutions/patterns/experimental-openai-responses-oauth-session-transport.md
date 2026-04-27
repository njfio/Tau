---
title: Experimental OpenAI Responses OAuth/session direct transport
category: patterns
date: '2026-04-27'
tags:
  - pattern
  - issue-3676
  - tau-provider
  - openai
  - responses
  - oauth
  - session
  - codex-fallback
related:
  - docs/adrs/0004-experimental-openai-responses-oauth-session-transport.md
  - specs/3676/spec.md
  - crates/tau-provider/src/client.rs
  - crates/tau-cli/src/cli_args.rs
---

# Experimental OpenAI Responses OAuth/session direct transport
## Problem
OpenAI OAuth/session auth had to travel through the Codex CLI subprocess, which added prompt translation, timeout, and empty-response failure modes to agent runtime recovery.
## Root cause
Provider selection did not have an explicit opt-in path for treating resolved OAuth/session credentials as bearer material for Tau's existing direct OpenAI Responses HTTP client, and the unsupported nature of that credential contract needed an ADR boundary.
## Solution
Add a disabled-by-default CLI/env flag for experimental direct OAuth/session transport, route only OpenAI OAuth/session credentials with resolved bearer material through the direct HTTP client when enabled, preserve API-key direct HTTP including API-key fallback, and keep Codex CLI fallback for disabled or unresolved cases.
## Prevention

Keep the experimental flag explicit, retain Codex fallback, cover the selector with focused tests for enabled, disabled, missing credential, session-token, and API-key fallback cases, and document the unsupported external API contract in ADR 0004.
