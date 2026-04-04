# Context Snapshot: tau-application-improvement-audit

- Timestamp: `2026-04-03T04:00:49Z`
- Mode: `deep-interview`
- Profile: `standard`
- Context type: `brownfield`

## Task Statement
Assess how Tau needs to improve across functionality, user experience, developer experience, and agent experience, with special attention to whether the current loops are connected, live-tested, autonomous, and self-improving in a Ralph/Wiggum-like sense.

## Desired Outcome
Produce a clear, execution-ready understanding of:
- what is already integrated vs only specified
- what is not connected end-to-end
- whether the important loops have been live tested vs only deterministically verified
- what the highest-leverage improvement priorities are

## Stated Solution
Run a deep interview rather than jumping directly into planning or implementation.

## Probable Intent Hypothesis
The user likely wants a grounded product/system audit that separates marketing-level “integrated” claims from real operational depth, then converts that into a prioritized improvement direction for the platform as a whole.

## Known Facts / Evidence
- `README.md` states several integrated end-to-end paths exist today, including local operator loop, gateway auth/session loop, unified runtime lifecycle loop, prompt optimization loop, and a connected operator GA loop.
- `README.md` also explicitly marks several areas as partial or expansion tracks:
  - broader production policy-optimization operating loops still expanding
  - dashboard UX still expanding
  - live third-party credential/network validation remains environment-specific
  - live connector/provider uptime validation remains environment-specific
  - full PRD-wide E2E scenario-group completion remains an expansion track
  - richer TUI UX flows continue to evolve
- `specs/milestones/m334/index.md` defines the Ralph supervisor loop as an active milestone, with goals to compose session, memory, learning, orchestration, and operator visibility into one inspectable outer loop.
- `Cargo.toml` shows the workspace includes runtime, gateway, orchestrator, memory, skills, training, trainer, dashboard, browser automation, coding-agent, and TUI crates, which suggests platform breadth is present.
- Search results show training-related crates and deterministic RL/training harnesses exist:
  - `tau-training-runner`
  - `tau-training-tracer`
  - `tau-training-store`
  - `tau-training-proxy`
  - `tau-trainer`
- `docs/guides/training-ops.md` exists and includes live-run RL benchmark, significance, and crash-resume procedures, which is evidence of operational thinking, not just model-training code.
- `tasks/tau-gaps-issues-improvements.md` still exists as a current gaps/improvements inventory.

## Constraints
- This is a brownfield repo-wide question, not a single-file feature request.
- The user asked for a deep interview, so the immediate goal is clarification and prioritization, not direct implementation.
- The interview should distinguish deterministic verification from true live operational validation.

## Unknowns / Open Questions
- Whether the user wants a near-term product triage, a longer-term architecture roadmap, or a launch-readiness assessment.
- Which gap category matters most right now: functionality, UX, DX, agent experience, autonomy, or learning loops.
- Whether “live tested” means local deterministic ops drills, staging with real providers, or fully production-like autonomous execution.
- Whether the user wants to optimize for operator productivity, agent autonomy, or training/self-improvement first when tradeoffs conflict.

## Decision-Boundary Unknowns
- How much OMX may decide on prioritization without explicit user confirmation.
- Whether the desired output is an audit only, a ranked roadmap, or a handoff into planning (`$ralplan`) after interview completion.

## Likely Codebase Touchpoints
- `README.md`
- `specs/milestones/m334/index.md`
- `tasks/tau-gaps-issues-improvements.md`
- `docs/guides/training-ops.md`
- `crates/tau-orchestrator/`
- `crates/tau-gateway/`
- `crates/tau-runtime/`
- `crates/tau-tui/`
- `crates/tau-trainer/`
- `crates/tau-training-*`
