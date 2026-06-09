# Plan 3807: Intake Clarifying Contract

## Approach

Add a small intake decision layer to
`crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`. The existing
classification and verifier-plan fields remain intact. New fields are additive:
`decision`, `clarifying_questions`, and optional blocking reason details. The
decision is derived from the existing classification and authority context, so
there is no provider call or model dependency.

## Affected Modules

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3807-intake-clarifying-contract/*`

## Risks And Mitigations

- Risk: changing persisted JSON breaks old records.
  Mitigation: additive fields use `#[serde(default)]`; legacy load is tested.
- Risk: questions become vague prose again.
  Mitigation: each question has a reason code and required input string.
- Risk: this is mistaken for mutation authority.
  Mitigation: docs explicitly state the clarifying contract blocks until
  verifier and edit/provider authority are present.

## Interfaces

New serialized intake fields:

- `decision`: `ready_to_run`, `needs_authority`, `needs_clarification`,
  `split_required`, `blocked_unsafe`, `missing_credentials`, or `unknown`.
- `clarifying_questions`: list of `{reason_code, question, required_input}`.

No CLI flag changes are required because the intake commands already print the
full JSON outcome.
