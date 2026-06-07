# Issue 3788: Provider-Backed Coding Loop Proof

Status: Implemented
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture

## Problem

The M334 live coding loop harness proves Tau's mission lifecycle with
controlled edits. It does not prove that a configured model provider can
produce the edit that drives a disposable coding task from a failing verifier to
a passing verifier.

## Scope

In scope:
- Add an opt-in provider-backed mode to the existing live coding loop harness.
- Require provider output to parse into the edit passed through
  `CodingMissionRunner`.
- Record sanitized provider/model evidence in the JSON report.
- Fail closed when credentials are missing, the provider call fails, or the
  provider output is malformed.

Out of scope:
- Publishing a live GitHub PR from the disposable fixture.
- Replacing the controlled local success/resume/blocked harness cases.
- Broad agent policy, tool-use, or scheduler changes.

## Acceptance Criteria

AC-1: Given provider-backed mode is enabled with a usable provider credential,
when the harness runs the disposable task, then it calls the provider and uses
the provider response as the controlled edit for `CodingMissionRunner`.

AC-2: Given the provider-backed run succeeds, when the report is written, then
it includes provider name, model name, parse status, verifier red/green
evidence, commit hash, and PR-ready bundle without exposing secret values.

AC-3: Given credentials are missing, a provider call fails, or provider output
is malformed, when the provider-backed mode runs, then the harness exits
non-zero and records a fail-closed reason instead of committing or producing a
PR-ready bundle.

## Conformance Cases

C-01 maps to AC-1 and AC-2: A real provider-backed run in a disposable repo
receives JSON edit instructions, applies `status.txt = pass`, records provider
metadata, observes failing then passing verifier evidence, commits, and prepares
the manual PR-ready bundle.

C-02 maps to AC-3: A deterministic malformed-provider fixture returns non-JSON
content and the harness exits with a provider-output failure report, no commit,
and no PR-ready bundle.

C-03 maps to AC-3: A missing-credential provider-backed invocation exits with a
credential failure report before provider dispatch.

## Success Signals

- `cargo test -p tau-coding-agent --bin tau_live_coding_loop_harness provider_backed`
- `./scripts/dev/test-provider-backed-autonomous-coding-loop.sh --mock-malformed`
- `TAU_LIVE_PROVIDER_PROOF=1 ./scripts/dev/test-provider-backed-autonomous-coding-loop.sh`
