# Spec: Issue #3668 - Isolate CLI provider backends from repo context bleed

Status: Reviewed

## Problem Statement
The CLI provider adapters currently spawn `codex`, `claude`, and `gemini`
subprocesses from the caller's working directory. In the gateway/TUI Ralph-loop
path that means the provider backend inherits repo-local `AGENTS.md`, `.tau`
artifacts, and other workspace context on top of the explicit Tau conversation
already injected into the request. Live `gpt-5.3-codex` repros show this causes
material latency and completion drift after early read-only tool calls.

## Scope
In scope:
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-provider/src/claude_cli_client.rs`
- `crates/tau-provider/src/gemini_cli_client.rs`
- `specs/3668/spec.md`
- `specs/3668/plan.md`
- `specs/3668/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- redesigning the gateway Ralph-loop verifier policy
- changing launcher timeout budgets
- adding new provider-side tracing files

## Acceptance Criteria
### AC-1 CLI provider subprocesses run from an isolated ephemeral directory
Given a CLI provider adapter handles a chat request,
when it spawns the backing CLI subprocess,
then it uses an isolated temporary working directory rather than inheriting the
caller repository cwd.

### AC-2 Textual tool-call promotion remains compatible
Given the provider returns textual `tool_calls` JSON from the isolated working
directory,
when Tau reads the last assistant message,
then the adapter still promotes the payload into structured tool calls exactly
as before.

### AC-3 Existing CLI provider behaviors remain green
Given the existing provider adapter tests run,
when the isolation change is applied,
then the adapters still return normal assistant text and existing regression
coverage remains green.

## Conformance Cases
- C-01 / AC-1 / Regression:
  a mock CLI backend reports its working directory, and the adapter test proves
  it is not the repository cwd.
- C-02 / AC-2 / Functional:
  the codex textual tool-call adapter regression still succeeds while running
  from the isolated working directory.
- C-03 / AC-3 / Regression:
  existing codex / claude / gemini adapter test suites stay green after the
  isolation change.

## Success Metrics / Observable Signals
- Gateway/TUI CLI-provider runs no longer inherit repo-local context as an
  accidental second prompt layer.
- Direct repros of the post-`ls` Codex backend prompt behave closer to the
  isolated-temp-dir path than the repo-cwd path.

## Files To Touch
- `specs/3668/spec.md`
- `specs/3668/plan.md`
- `specs/3668/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-provider/src/claude_cli_client.rs`
- `crates/tau-provider/src/gemini_cli_client.rs`
