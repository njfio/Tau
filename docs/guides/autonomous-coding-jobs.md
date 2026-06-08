# Autonomous Coding Jobs

Tau autonomous coding jobs link a durable job record, a `CodingMissionRunner`
state file, and the background-job recovery runtime. The first product loop is
bounded: operators provide the repo, issue/spec context, verifier commands, and
controlled edits; Tau runs/replays the mission until it is blocked or PR-ready.

## State Layout

- Autonomous job records:
  `.tau/autonomous-coding/autonomous-coding-jobs/<job-id>.json`
- Operator status snapshots:
  `.tau/autonomous-coding/autonomous-coding-jobs/<job-id>.status.json`
- Mission state:
  `.tau/autonomous-coding/coding-missions/<mission-id>.json`
- Background job state:
  `.tau/jobs`

## CLI

Submit a job:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- submit \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --repo-path /path/to/repo \
  --mission-id issue-123 \
  --issue-url https://github.com/owner/repo/issues/123 \
  --goal "Make the spec verifier pass" \
  --verifier-command "cargo test -p some-crate spec_c01" \
  --edit "src/example.rs=updated contents" \
  --commit-message "Make verifier green"
```

Run or resume the job:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- run \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --job-id <job-id>
```

Replay a checkpoint with updated edits:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- replay \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --job-id <job-id> \
  --edit "src/example.rs=updated contents"
```

Inspect operator status:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- status \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --job-id <job-id>
```

Run stuck background-job recovery and refresh linked autonomous jobs:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- recover \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs
```

Request protected-branch-safe GitHub auto-merge for a PR-ready job:

```bash
GH_TOKEN=... cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- auto-merge \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --job-id <job-id> \
  --allow-auto-merge \
  --merge-method squash \
  --delete-branch
```

This invokes normal GitHub auto-merge behavior. It honors branch protections and
required checks; it does not use admin bypass flags.

Ingest an arbitrary issue without verifier/edit authority:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- intake-issue \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --intake-id issue-123 \
  --issue-url https://github.com/owner/repo/issues/123 \
  --issue-title "Issue title" \
  --issue-body "Issue body" \
  --repo-path /path/to/repo
```

This creates a blocked authority plan that lists the verifier and edit authority
needed before Tau can mutate the repository.

## Boundaries

This loop prepares PR-ready evidence, can create a draft PR when the mission
state is configured for draft PR mode and GitHub auth exists in the environment,
and can request GitHub auto-merge when explicit policy/auth/PR URL gates pass.
It does not bypass protected branches, and it does not claim arbitrary issue
solving without verifier commands and bounded edit authority.
