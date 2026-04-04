# Spec: Issue #3664 - Align tau-unified request timeout with CLI provider backend timeouts

Status: Implemented

## Problem Statement
`tau-unified.sh` forwards `--request-timeout-ms` to `tau-coding-agent`, but
interactive provider requests can still run through CLI-backed providers with
their own timeout flags. In the OpenAI oauth/session path this leaves the
launcher advertising a 600000ms budget while the Codex CLI backend still times
out at its default 120000ms.

## Scope
In scope:
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `specs/3664/spec.md`
- `specs/3664/plan.md`
- `specs/3664/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing provider timeout defaults in Rust
- changing TUI request timeout handling
- redesigning auth/provider backend selection

## Acceptance Criteria
### AC-1 Unified runtime request timeout aligns with CLI provider backend timeouts
Given `tau-unified.sh up` is invoked with a request timeout,
when it builds the `tau-coding-agent` command,
then it forwards that timeout to the CLI-backed provider timeout flags so the
provider does not fail earlier than the advertised request budget.

### AC-2 Launcher tests cover default and override propagation
Given the launcher tests run,
when they inspect the recorded `up` command,
then they assert both the default timeout and an explicit override are
forwarded to the provider backend timeout flags.

### AC-3 Existing launcher behavior remains intact
Given the launcher timeout forwarding is added,
when existing `tau-unified` flows run,
then behavior remains unchanged apart from the aligned provider timeout budget.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `tau-unified.sh up` records `--openai-codex-timeout-ms <request_timeout_ms>`
  in the spawned command.
- C-02 / AC-1 / Functional:
  the launcher also forwards the same value to the Anthropic and Google CLI
  backend timeout flags for consistency.
- C-03 / AC-2 / Regression:
  the launcher test suite asserts both default and overridden timeout
  propagation.
- C-04 / AC-3 / Regression:
  existing launcher tests remain green.

## Success Metrics / Observable Signals
- OpenAI oauth/session gateway requests no longer fail at 120000ms when the
  unified launcher was configured with a larger request timeout.
- The recorded runtime command shows one aligned timeout budget across the
  request and provider CLI backend flags.

## Files To Touch
- `specs/3664/spec.md`
- `specs/3664/plan.md`
- `specs/3664/tasks.md`
- `specs/milestones/m334/index.md`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
