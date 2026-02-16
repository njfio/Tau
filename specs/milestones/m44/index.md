# Milestone M44: OpenAI Codex Native Support

Status: Active

## Objective

Add first-class OpenAI Codex model support in Tau by routing Codex requests to
the OpenAI Responses API, preserving behavior for standard chat-completions
models, and validating with automated plus live-provider checks.

## Scope

In scope:

- OpenAI client routing and fallback logic for Codex-only models
- OpenAI Responses API request/response parsing for text + usage fields
- Targeted tests for Codex routing, parsing, and backward compatibility
- Live validation run against OpenAI Codex model with real API key

Out of scope:

- End-to-end Realtime API support
- Broad provider abstraction redesign outside OpenAI compatibility handling
- Non-OpenAI provider behavior changes

## Success Signals

- Direct OpenAI Codex invocation succeeds in Tau (`openai/gpt-5.2-codex`).
- Existing OpenAI chat-completions models continue to pass tests unchanged.
- Conformance tests prove routing + parser behavior under expected payloads.

## Issue Hierarchy

Milestone: GitHub milestone `M44 OpenAI Codex Native Support`

Epic:

- `#2232` Epic: M44 OpenAI Codex native support

Story:

- `#2233` Story: M44.1 Route Codex requests to OpenAI Responses API

Task:

- `#2234` Task: M44.1.1 Implement Codex endpoint routing and parser

Subtask:

- `#2235` Subtask: M44.1.1a Add correct OpenAI Codex endpoint support and validate live
