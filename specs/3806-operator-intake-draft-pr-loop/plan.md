# Plan: Operator Recovery, Hands-Off Intake, and Draft PR Loop

## Approach

Keep this slice on the existing autonomous coding runtime. Do not introduce a
parallel scheduler or dashboard. Add operator commands to `tau-unified` that
read the durable job status files and delegate actions to
`tau-autonomous-coding-job`. Extend issue intake with deterministic planning
metadata so no-authority runs tell the operator exactly what verifier and
authority are missing. Harden PR publication evidence so draft PR creation is
observable when GitHub auth exists and manual fallback is explicit when it does
not.

## Affected Modules

- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-agent-core/src/coding_mission.rs`
- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
- `docs/guides/autonomous-coding-jobs.md`
- `README.md`

## Risks and Mitigations

- Risk: operator commands accidentally mutate the wrong state directory.
  Mitigation: require explicit state-dir parsing with defaults matching existing
  `tau-unified` configuration and delegate to the existing CLI.
- Risk: "hands-off" is overclaimed.
  Mitigation: generated verifier plans are advisory contracts; mutation remains
  blocked until verifier and edit/provider authority exist.
- Risk: draft PR creation can fail due to local auth.
  Mitigation: record failure/manual command as evidence without failing the
  verified job.
- Risk: shell parsing in `tau-unified` grows brittle.
  Mitigation: keep job listing/inspection JSON parsing in small Python blocks
  and validate through `scripts/run/test-tau-unified.sh`.

## Interfaces

- `scripts/run/tau-unified.sh jobs [--autonomous-coding-state-dir <path>]`
- `scripts/run/tau-unified.sh job <job-id> [--autonomous-coding-state-dir <path>]`
- `scripts/run/tau-unified.sh recover [--jobs-state-dir <path>] [--autonomous-coding-state-dir <path>]`
- `scripts/run/tau-unified.sh replay <job-id> [--autonomous-coding-state-dir <path>]`
- `scripts/run/tau-unified.sh block <job-id> [--reason-code <code>] [--detail <text>]`
- Issue intake JSON gains verifier-plan and next-action metadata.
- PR-ready bundles retain manual command fields and record draft create evidence.

## Rollback

Remove the `tau-unified` job subcommands, the new intake planning fields, and
the PR publication evidence additions. Existing submit/run/replay/recover/status
CLI commands remain the lower-level fallback.
