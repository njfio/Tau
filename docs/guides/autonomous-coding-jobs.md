# Autonomous Coding Jobs

Tau autonomous coding jobs link a durable job record, a `CodingMissionRunner`
state file, provider-repair evidence, and the background-job recovery runtime.
The product loop is bounded: operators provide the repo, issue/spec context,
verifier commands, and either controlled edits or provider repair authority; Tau
runs/replays the mission until it is blocked or PR-ready.

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

Run the authorized issue-to-merge loop in one command with supplied edits:

```bash
GH_TOKEN=... cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- issue-to-merge \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --repo-path /path/to/repo \
  --intake-id issue-123 \
  --mission-id issue-123 \
  --issue-url https://github.com/owner/repo/issues/123 \
  --issue-title "Issue title" \
  --issue-body "Issue body" \
  --verifier-command "cargo test -p some-crate spec_c01" \
  --edit "src/example.rs=updated contents" \
  --commit-message "Make verifier green" \
  --pr-mode draft \
  --allow-auto-merge \
  --merge-method squash
```

This command ties together issue intake, durable job creation, verifier-gated
mission execution, PR-ready or draft-PR publication, and optional GitHub
auto-merge request. If verifier commands or edit authority are missing, it
falls back to a blocked issue-intake authority plan and does not mutate the
repository.

Run the same loop with provider repair authority instead of manual edits:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- issue-to-merge \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --repo-path /path/to/repo \
  --intake-id issue-123 \
  --mission-id issue-123 \
  --issue-url https://github.com/owner/repo/issues/123 \
  --issue-title "Issue title" \
  --issue-body "Issue body" \
  --verifier-command "cargo test -p some-crate spec_c01" \
  --provider-repair-command ./scripts/local-provider-repair-adapter.sh \
  --provider-repair-attempts 3 \
  --provider-repair-provider openrouter \
  --provider-repair-model qwen/qwen3-235b-a22b \
  --commit-message "Make verifier green"
```

The repair command is an adapter boundary. Tau writes a sanitized repair context
JSON file and invokes the adapter with:

- `TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT`: path to the context JSON,
- `TAU_AUTONOMOUS_CODING_REPAIR_ATTEMPT`: one-based attempt index,
- `TAU_AUTONOMOUS_CODING_REPAIR_PROVIDER`: configured provider label,
- `TAU_AUTONOMOUS_CODING_REPAIR_MODEL`: configured model label.

The adapter must print one JSON payload to stdout. Supported payloads are:

```json
{"relative_path":"src/example.rs","contents":"updated contents\n"}
```

```json
{"edits":[{"relative_path":"src/example.rs","contents":"updated contents\n"}]}
```

```json
{"files":{"src/example.rs":"updated contents\n"}}
```

```json
{"diff":"--- a/src/example.rs\n+++ b/src/example.rs\n@@ -1 +1 @@\n-old\n+new\n"}
```

Unified diff support is intentionally narrow: existing-file modifications only,
no `/dev/null`, no renames, no path escapes, and hunks must match the current
file. Accepted provider output is converted into checked mission edits and is
applied by the existing mission runner, so checkpoints, verifier reruns, commit
evidence, and PR-ready bundles stay in the durable job path.

`tau-unified status` also reads
`.tau/autonomous-coding/autonomous-coding-jobs/*.status.json` and emits
`control_plane.autonomous_coding.*` markers for the latest job: status,
verifier summary, provider repair result, event log, heartbeat/lease, replay and
recovery counts, PR state, and auto-merge state.

## Boundaries

This loop prepares PR-ready evidence, can create a draft PR when draft mode and
GitHub auth are available, and can request GitHub auto-merge when explicit
policy/auth/PR URL gates pass. It does not bypass protected branches, and it
does not claim arbitrary issue solving without verifier commands plus either
operator-supplied edits or bounded provider repair authority. Crash-resume and
stuck-job recovery state is persisted and visible; the fully polished operator
replay/recovery UX is still product work.
