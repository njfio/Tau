# Provider Tool-Choice Matrix v1

Status: accepted
Date: 2026-04-27
Scope: Ralph-loop recovery provider prompt contracts for CLI-backed providers.

## Purpose

The provider tool-choice matrix records how Tau maps gateway retry policy into provider-facing CLI prompt contracts. It exists so Ralph-loop recovery can require tool evidence without depending on provider-specific native tool APIs.

## Current matrix

| Provider | Auto | Required | Concrete tool |
| --- | --- | --- | --- |
| Codex CLI | Allows plain assistant text only when no tool is required; tells the model to return textual `tool_calls` JSON if it needs a Tau tool. | States `A Tau tool call is required for this turn.` and requires assistant text containing exact `tool_calls` JSON. | States `A Tau tool call to \`<name>\` is required for this turn.` and requires textual `tool_calls` JSON naming the concrete tool. |
| Claude CLI | Allows plain assistant text only when no tool is required; tells the model to return textual `tool_calls` JSON if it needs a Tau tool. | States `A Tau tool call is required for this turn.` and requires assistant text containing exact `tool_calls` JSON. | States `A Tau tool call to \`<name>\` is required for this turn.` and requires textual `tool_calls` JSON naming the concrete tool. |
| Gemini CLI | Allows plain assistant text only when no tool is required; tells the model to return textual `tool_calls` JSON if it needs a Tau tool. | States `A Tau tool call is required for this turn.` and requires assistant text containing exact `tool_calls` JSON. | States `A Tau tool call to \`<name>\` is required for this turn.` and requires textual `tool_calls` JSON naming the concrete tool. |

## Gateway retry policy relationship

Ralph-loop recovery currently chooses provider tool choice in the gateway:

- First attempt stays `ToolChoice::Auto`.
- Generic mutation recovery retries fall back to `ToolChoice::Required` when there is no safe concrete write hint.
- Safe create/write recovery can select a concrete `ToolChoice::Tool { name: "write" }` before provider dispatch.
- If a Required retry still returns no tool evidence, the gateway blocks with `required_tool_evidence_missing_exhausted` and now surfaces that through operator snapshots.

## Audit result

The Required contract is explicit across Codex, Claude, and Gemini. Auto is intentionally permissive so normal assistant text remains available. Concrete Tool is now explicit across all three CLI adapters, so gateway concrete write recovery becomes a named provider prompt requirement rather than a soft Auto-style hint.

## Follow-up test contract

Provider regressions should cover the full matrix by provider:

- Auto prompts must retain the plain-text escape hatch.
- Required prompts must remove the plain-text escape hatch and require textual `tool_calls` JSON.
- Concrete Tool prompts must require the named tool and remove the plain-text escape hatch.

## Compatibility

This matrix preserves the textual `tool_calls` bridge used by `promote_assistant_textual_tool_calls`. It does not add provider-native tool APIs or new dependencies.
