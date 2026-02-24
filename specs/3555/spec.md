# Spec: Issue #3555 - Wire interactive request timeout/retry policy into local runtime + unified TUI

Status: Implemented

## Problem Statement
Interactive TUI turns currently allow long model request latency windows (for
example 120s timeout per attempt with multiple retries), which creates poor
operator UX for trivial prompts. Additionally, local runtime wiring ignores the
CLI request timeout when constructing `AgentConfig`, so user-provided timeout
controls are not enforced consistently.

## Scope
In scope:
- Ensure local runtime agent request timeout uses CLI timeout settings.
- Add `tau-tui agent` timeout/retry launch controls and pass them through to
  `tau-coding-agent`.
- Add fast-fail default timeout/retry policy in `tau-unified.sh tui` with
  explicit override options.
- Add/update tests to verify timeout/retry command wiring and runtime timeout
  enforcement.
- Update run/docs guidance for new controls.

Out of scope:
- Provider-side retry strategy redesign.
- New dashboard UI layout work.
- Non-interactive transport runtime timeout policy changes.

## Acceptance Criteria
### AC-1 Local runtime honors CLI request timeout for agent turn execution
Given local runtime prompt execution is invoked with a low
`--request-timeout-ms` value,
when the model client stalls longer than the timeout,
then execution fails with agent request-timeout behavior aligned to the CLI
value rather than a hardcoded default.

### AC-2 `tau-tui agent` forwards timeout/retry controls to coding agent launch
Given operator runs `tau-tui agent` with timeout/retry options,
when interactive command is rendered/launched,
then command includes `--request-timeout-ms` and
`--agent-request-max-retries` with provided values.

### AC-3 `tau-unified.sh tui` applies fast-fail interactive defaults with override
Given operator runs `tau-unified.sh tui` without explicit timeout/retry flags,
when launch command is built,
then interactive command uses bounded defaults (shorter timeout, fewer retries)
and supports explicit override via flags/env.

### AC-4 Documentation reflects effective timeout/retry controls
Given operators configure interactive timeout/retry behavior,
when reading launcher/run docs,
then the new flags and env variables are documented with defaults and examples.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Integration/Conformance | CLI `request_timeout_ms=20` and slow client | run local runtime prompt | returns request-timeout with 20ms timeout semantics |
| C-02 | AC-2 | Unit/Functional | `tau-tui agent --request-timeout-ms 45000 --agent-request-max-retries 0` | build launch command | command forwards both flags/values |
| C-03 | AC-3 | Functional | `tau-unified.sh tui` default path | runner-mode launch | forwarded timeout/retry values match fast-fail defaults |
| C-04 | AC-3 | Functional | explicit timeout/retry overrides | runner-mode launch | forwarded values reflect override inputs |
| C-05 | AC-4 | Docs | runbook/README paths | inspect docs | includes flags/env and default behavior |

## Success Metrics / Observable Signals
- Interactive worst-case wait drops materially below previous multi-minute path
  for default launcher flow.
- Timeout/retry launch settings are visible in printed interactive command.
- Regression tests fail if timeout wiring reverts to hardcoded defaults.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `crates/tau-coding-agent/src/startup_local_runtime.rs` now maps `request_timeout_ms` from `cli.request_timeout_ms`. Verified by `cargo test -p tau-coding-agent regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent -- --nocapture`. |
| AC-2 | ✅ | `crates/tau-tui/src/main.rs` adds `--request-timeout-ms` and `--agent-request-max-retries` parse + command pass-through. Verified by `cargo test -p tau-tui -- --nocapture` (`spec_c07`, `functional_spec_c08`). |
| AC-3 | ✅ | `scripts/run/tau-unified.sh` adds fast-fail defaults (`45000`, `0`) and override flags/env; launcher forwards values to runner and `tau-tui agent`. Verified by `bash scripts/run/test-tau-unified.sh`. |
| AC-4 | ✅ | Updated docs in `README.md` and `docs/guides/operator-deployment-guide.md` with defaults, flags, and env overrides. |
