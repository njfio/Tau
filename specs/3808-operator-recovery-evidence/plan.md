# Plan 3808: Operator Recovery Evidence Surface

## Approach

Extend the existing shell-level `tau-unified` operator surface. The runtime
already writes the evidence fields into job status snapshots and issue-intake
records, so this slice should project those fields rather than changing Rust
state schemas.

Implementation steps:

1. Add failing assertions to `scripts/run/test-tau-unified.sh status_contract`
   for the missing status/job evidence and intake views.
2. Extend `scripts/run/tau-unified.sh` to preserve additional status snapshot
   fields in the control-plane snapshot and logs.
3. Extend `jobs` and `job` output with event-log, provider, PR publication,
   auto-merge, background, heartbeat, lease, and mark-blocked evidence.
4. Add `intakes` and `intake` commands that read persisted
   `.tau/autonomous-coding/issue-intake/*.json` records.
5. Update the operator guide and run the focused shell verification.

## Affected Files

- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3808-operator-recovery-evidence/*`

## Risks

- Risk: line-oriented output becomes too noisy.
  Mitigation: keep list output compact and reserve detailed evidence for
  `job`/`intake`.
- Risk: malformed intake/status JSON breaks the operator command.
  Mitigation: list commands skip unreadable JSON, and detail commands fail with
  a deterministic not-found marker.
- Risk: shell quoting regresses commands with spaces.
  Mitigation: reuse the existing `clean` output sanitizer and focused shell
  contract tests.

## Interfaces

No runtime API changes. This is an additive operator CLI contract:

- `tau-unified intakes [--autonomous-coding-state-dir <path>]`
- `tau-unified intake <intake-id> [--autonomous-coding-state-dir <path>]`
- Additional `control_plane.autonomous_coding.*`,
  `autonomous_coding.job.*`, and `autonomous_coding.intake.*` output lines.
