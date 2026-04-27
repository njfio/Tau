---
title: 'ADR 0004: Experimental direct OpenAI Responses OAuth/session transport'
category: adrs
date: '2026-04-27'
tags:
  - adr
  - issue-3676
  - tau-provider
  - openai
  - responses
  - oauth
  - session
  - fallback
related:
  - docs/adrs/0004-experimental-openai-responses-oauth-session-transport.md
  - specs/3676/spec.md
---

# ADR 0004: Experimental direct OpenAI Responses OAuth/session transport
## Problem
Issue #3676 needs an optional way to bypass the Codex CLI subprocess for OpenAI OAuth/session-style agent runtime work while preserving supported API-key behavior and fallback safety.
## Root cause
The Codex CLI subprocess adds prompt translation, timeout, and empty-response failure modes, but direct OAuth/session use against the public OpenAI Responses endpoint is not a documented supported contract.
## Solution
Add an explicit experimental opt-in in tau-provider that can route resolved OAuth/session bearer credentials through the direct OpenAI Responses HTTP transport, while defaulting to existing paths and falling back to Codex CLI when disabled or unresolved.
## Prevention

Keep the experiment disabled by default, preserve Codex CLI fallback, cover provider selection with focused tests, and treat OAuth/session direct transport as reversible undocumented behavior.
