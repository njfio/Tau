# Spec #2267

Status: Implemented
Milestone: specs/milestones/m46/index.md
Issue: https://github.com/njfio/Tau/issues/2267

## Problem Statement

Tau currently lacks repeatable fuzz coverage for high-risk untrusted input
surfaces. Parser/runtime paths accept raw JSON/NDJSON from external callers, but
there is no dedicated fuzz contract ensuring those paths remain panic-free and
error-classification stable under malformed/randomized payloads.

## Scope

In scope:

- Add deterministic fuzz-conformance tests for untrusted parser surfaces.
- Add a local fuzz contract runner script for repeatable execution.
- Cover two critical surfaces:
  - RPC raw/NDJSON parser flow (`tau-runtime`).
  - Gateway websocket request parser/classifier (`tau-gateway`).
- Document local fuzz contract usage.

Out of scope:

- Introducing `cargo-fuzz`/libFuzzer dependency stack in this slice.
- CI workflow topology changes.
- Fuzzing every crate/surface in one pass.

## Acceptance Criteria

- AC-1: Given deterministic fuzz input corpus generation, when executing RPC fuzz
  conformance tests, then parser/dispatcher paths do not panic and return
  structured outputs for all iterations.
- AC-2: Given deterministic fuzz input corpus generation, when executing gateway
  websocket fuzz conformance tests, then parse/classification helpers do not
  panic and always produce classified error/result outputs.
- AC-3: Given local operator/dev workflow, when running the fuzz contract script,
  then all fuzz-conformance tests execute in one command and emit pass/fail
  summary.
- AC-4: Given docs review, when developers need fuzz validation, then docs
  contain deterministic command usage and covered surfaces.

## Conformance Cases

- C-01 (AC-1, conformance): RPC raw-envelope fuzz test executes >=10,000
  deterministic malformed/variable payload iterations without panic.
- C-02 (AC-1, conformance): RPC NDJSON fuzz test executes >=10,000 deterministic
  line-shape variants without panic and returns report counters.
- C-03 (AC-2, conformance): Gateway WS parser fuzz test executes >=10,000
  deterministic payload iterations without panic; parse errors map through
  classifier contract.
- C-04 (AC-3, functional): local fuzz contract script runs targeted conformance
  tests for `tau-runtime` and `tau-gateway` in one invocation.
- C-05 (AC-4, documentation): docs include fuzz contract command and covered
  module list.

## Success Metrics / Observable Signals

- `spec_c0x_*` fuzz-conformance tests are present and passing.
- Local fuzz contract script passes on a clean workspace.
- Existing `tau-runtime` and `tau-gateway` full test suites stay green.
