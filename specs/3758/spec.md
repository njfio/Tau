# Spec: Issue #3758 - Supervised deploy/stop process lifecycle control

Status: Accepted

## Problem Statement

`POST /gateway/deploy` and `POST /gateway/agents/{agent_id}/stop` currently
persist deterministic deploy state and transition gateway service status, but
they do not supervise an agent process lifecycle. Operators need deploy/stop to
prove spawn and terminate behavior with durable, inspectable process evidence.

This is a P0 slice. The user explicitly requested implementation in the active
goal, so the P0 acceptance gate is satisfied for this branch.

## Acceptance Criteria

### AC-1 Deploy starts a supervised process
Given a gateway configured with a deploy process supervisor and a valid
authenticated deploy request,
When `POST /gateway/deploy` is called,
Then the gateway starts a supervised process, persists process lifecycle
metadata, and returns process evidence in the deploy response.

### AC-2 Stop terminates a supervised process
Given a previously deployed agent with a live supervised process,
When `POST /gateway/agents/{agent_id}/stop` is called,
Then the gateway terminates the supervised process, persists stopped lifecycle
metadata, and returns deterministic stop evidence.

### AC-3 Deploy remains fail-closed when process spawn fails
Given a gateway configured with a failing deploy process supervisor,
When `POST /gateway/deploy` is called,
Then the gateway returns a deterministic process-spawn error and does not report
the agent as deploying.

### AC-4 Existing endpoint contracts stay stable
Given unauthorized, invalid, or unknown-agent deploy/stop requests,
When existing deploy/stop regression cases run,
Then existing deterministic auth, validation, discovery, and not-found behavior
remains unchanged.

### AC-5 Request schema stays backward compatible
Given existing deploy clients,
When they send the prior `{agent_id, profile, model}` body,
Then no new request field is required and arbitrary command strings are not read
from request bodies.

### AC-6 Scoped verification passes
Given this implementation slice,
When scoped validation runs,
Then formatting, targeted gateway tests, and package-scoped checks pass or any
remaining unrelated workspace baseline blocker is reported explicitly.

### AC-7 Cortex automation scope is explicit
Given the prioritized action list asks to decide cortex automation scope,
When architecture documentation is updated,
Then Cortex is documented as advisory-only for this milestone unless a future
typed supervisor-routing action contract is accepted.

### AC-8 Quality and operator documentation closures are durable
Given the P1/P2 action items require quality, onboarding, security,
key-rotation, and dashboard-stack decisions,
When this slice is complete,
Then the repo contains explicit test coverage targets, a regenerated dependency
graph artifact, release freshness cadence for contributor/security docs,
encrypted credential key-rotation runbook coverage, and a current dashboard
stack ADR.

### AC-9 Deploy evidence is visible in the operator shell
Given deploy state contains supervised process records,
When an operator opens `/ops/deploy`,
Then the deploy panel renders process lifecycle evidence including process
status, pid, and stop metadata.

### AC-10 Supervisor configuration supports safe lifecycle operations
Given a command-backed deploy supervisor is configured by environment,
When it receives static args and stop requests,
Then JSON args preserve quoted argument boundaries and stop attempts graceful
termination before falling back to a hard kill.

### AC-11 Operator shell deploy/stop controls execute lifecycle operations
Given an operator opens `/ops/deploy` in the browser,
When they submit the deploy form and then submit the row-level stop form,
Then the shell posts to gateway-owned handlers, starts the configured process,
renders running process evidence, terminates the process, and renders stopped
process evidence.

## Scope

### In Scope
- Gateway-local deploy process supervisor abstraction.
- A command-backed supervisor implementation for configured/test runtimes.
- Persisted deploy process fields: process id, pid when available, lifecycle
  status, started/stopped timestamps, stop reason, and exit status.
- Operator-shell deploy process evidence table.
- Operator-shell deploy and stop form handlers backed by the same lifecycle
  operations as the JSON gateway API.
- Environment-driven command args via legacy whitespace args or JSON array args.
- Graceful terminate-then-kill fallback for command-backed stop.
- Backward-compatible deploy/stop response fields.
- Conformance/regression tests for live spawn/terminate and spawn failure.
- Milestone/index and spec task updates.
- Cortex automation scope decision documentation.
- Quality target, dependency graph, security cadence, key rotation, and
  dashboard stack direction documentation.

### Out of Scope
- Accepting command strings from operator request bodies.
- Replacing the mission/Ralph supervisor model.
- External process pools, containers, or production deployment orchestration.
- Dashboard redesign or route migration.

## Conformance Cases

| Case | AC | Tier | Scenario | Expected signal |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Integration | configured supervisor deploy | response and state include `process_status=running` and pid/process id |
| C-02 | AC-2 | Functional/Integration | stop known live process | response and state include `process_status=stopped` and stop reason |
| C-03 | AC-3 | Regression | failing supervisor deploy | deterministic `deploy_process_start_failed` error and no deploying state |
| C-04 | AC-4 | Regression | existing invalid/auth/not-found tests | prior status/error contracts remain stable |
| C-05 | AC-5 | Contract | existing deploy body | no command field required; request body command is ignored if present |
| C-06 | AC-6 | Verify | scoped commands | targeted checks pass or unrelated blockers are named |
| C-07 | AC-7 | Docs/Architecture | cortex scope decision | architecture note states advisory-only and names supervisor-routing gate |
| C-08 | AC-8 | Docs/Governance | P1/P2 closure docs | docs and generated dependency graph artifacts exist |
| C-09 | AC-9 | Functional | `/ops/deploy` after deployment | process lifecycle table renders persisted process status and pid |
| C-10 | AC-10 | Unit | configured supervisor args/stop | JSON args preserve spaces; stop attempts graceful termination before kill fallback |
| C-11 | AC-11 | Functional/Integration | submit `/ops/deploy` form then row stop form | redirects preserve shell context; table moves from running process pid to stopped stop reason |

## Success Metrics / Observable Signals

- Deploy/stop is no longer state-only when the gateway is configured with a
  process supervisor.
- Process lifecycle evidence is durable enough for operator inspection after
  the HTTP response.
- Operators can inspect deploy process evidence in `/ops/deploy` without
  reading the state file directly.
- Operators can start and stop the supervised deploy process from `/ops/deploy`
  without leaving the shell.
- Existing deploy/stop clients remain compatible.
- Remaining listed action items are either closed with durable artifacts or
  explicitly deferred behind future specs.
