# Cortex Automation Scope

## Decision

Cortex remains **advisory-only** for this milestone.

It may summarize status, answer operator questions, enrich prompts with memory
and observer context, and report readiness through `/cortex/status`. It must not
route supervisor actions, start or stop processes, mutate credentials, approve
mission transitions, or dispatch deployment changes directly.

## Rationale

The current implementation records observer events, serves `/cortex/chat`, and
computes readiness gates. Those paths are inspectable and reversible because
they produce guidance and status only. Supervisor-routing actions would cross a
different trust boundary: they would need action authorization, durable audit
records, rollback semantics, operator confirmation, and conformance tests for
every action class.

Keeping Cortex advisory-only prevents an LLM-facing surface from becoming a
hidden control plane while the deploy/stop lifecycle work moves process control
into an explicit gateway supervisor boundary.

## Escalation Gate

Cortex can be reconsidered for supervisor-routing only after a future spec adds
all of the following:

- A typed action envelope with explicit allow/deny policy.
- Operator-visible audit entries for requested, accepted, rejected, executed,
  failed, and rolled-back actions.
- Route-level authorization and CSRF/session checks matching the target action.
- Dry-run preview support with deterministic diff/evidence output.
- Rollback or compensating-action semantics for each mutable action.
- Conformance tests proving advisory responses cannot execute actions without
  the typed action envelope.

Until that gate exists, Cortex output is guidance. The gateway, mission
supervisor, deploy process supervisor, and operator action endpoints remain the
only action-executing surfaces.
