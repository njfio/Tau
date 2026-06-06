# Spec: Issue #3768 - Runtime reality gate for agentic-runtime claims

Status: Implemented
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture
Parent: #3654

## Problem Statement

Tau has real runtime, gateway, Agent Canvas, RL harness, auth, transport, and
validation proof surfaces. The remaining risk is claim drift: deterministic
proof, live proof, production-readiness, and aspiration can be described as if
they are the same thing. Operators need one executable gate that says what works,
what is partial, what is opt-in/live-env only, and what must not be claimed yet.

## Scope

In scope:

- Add a deterministic runtime reality gate script that emits JSON and markdown
  evidence.
- Run the fast proof surfaces by default: Tau product proof check, tau-unified
  launcher regression, Agent Canvas proof-loop regression, and roadmap status
  sync.
- Record opt-in evidence requirements for heavyweight/live gates without running
  them by default.
- Record maintainability hotspot line counts and classify them honestly.
- Fail closed if an unsupported claim is represented as fully complete.
- Update README guidance so product claims point to the executable gate.

Out of scope:

- Completing production-scale RL policy operations.
- Building a full dashboard command center UX.
- Running live third-party provider credentials by default.
- Claiming headed-browser pixel evidence from shell-only Agent Canvas proof.
- Refactoring large hotspot files in this slice.

## Acceptance Criteria

AC-1: Given the runtime reality gate runs, when evidence is written, then JSON
and markdown outputs separate integrated, partial, deterministic-only,
live-env-required, opt-in-heavy, and not-claimable surfaces.

AC-2: Given a not-claimable surface such as autonomous-forever or shell-only
browser-pixel proof, when the gate evaluates claims, then it fails if that
surface is represented as complete.

AC-3: Given current `master`, when the gate runs in default mode, then it
executes and records the fast proof surfaces: Tau product proof check,
tau-unified launcher test, Agent Canvas proof-loop test, and roadmap status sync.

AC-4: Given heavyweight or live-env proof is not requested, when the gate runs,
then RL productionization, full release validation, live provider validation, and
headed-browser pixel proof are recorded as opt-in or not-claimable rather than
silently passing.

AC-5: Given README capability language is inspected, when future readers look
for proof boundaries, then it links to the runtime reality gate and preserves the
deterministic/live/production distinction.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | fake fast proof commands | run gate | JSON and markdown contain all reality categories |
| C-02 | AC-2 | Regression | unsupported surfaces in default manifest | run gate | unsupported surfaces remain `not_claimable` and gate passes only because they are not overstated |
| C-03 | AC-3 | Conformance | current repo checkout | run default gate | four fast proof checks are recorded as passed |
| C-04 | AC-4 | Functional | heavy/live flags omitted | run gate | heavy/live checks are skipped/opt-in with explicit commands |
| C-05 | AC-5 | Regression | README claim section | inspect docs | runtime reality gate is linked as claim evidence |

## Success Metrics / Observable Signals

- `scripts/dev/test-runtime-reality-gate.sh`
- `scripts/dev/runtime-reality-gate.sh --output-json /tmp/tau-runtime-reality.json --output-md /tmp/tau-runtime-reality.md`
- `scripts/dev/prove-tau-product.sh --check`
- `scripts/run/test-tau-unified.sh`
- `scripts/dev/test-ops-chat-canvas-proof.sh`
- `scripts/dev/roadmap-status-sync.sh --check --quiet`
- `git diff --check`
