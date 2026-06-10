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

The `tau-unified` operator surface wraps the same durable state and action
commands:

```bash
scripts/run/tau-unified.sh jobs
scripts/run/tau-unified.sh job <job-id>
scripts/run/tau-unified.sh intakes
scripts/run/tau-unified.sh intake <intake-id>
scripts/run/tau-unified.sh recover
scripts/run/tau-unified.sh replay <job-id>
scripts/run/tau-unified.sh block <job-id> \
  --reason-code operator_marked_blocked \
  --detail "operator inspected job and marked it blocked"
```

`jobs` lists each durable job with status, operator state, replay safety,
recoverability, PR state, reason code, next command, and a plain resume
explanation. `job <job-id>` expands the evidence: verifier summary, provider
repair state, changed files, draft PR publication command/result, event log, and
why replay or recovery is safe or unsafe. It also exposes recovery evidence
paths, provider repair context, GitHub draft PR stdout/stderr/exit status,
auto-merge command state, heartbeat/lease, and the safe mark-blocked command.

`intakes` lists persisted issue-intake decisions with classification, decision,
question count, missing-input count, reason code, and next action. `intake
<intake-id>` expands the verifier plan, suggested verifier commands, required
authority, missing inputs, clarifying questions, and a safe rerun command when
one can be generated from concrete stored inputs.

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

For bounded docs/readme issues, `issue-to-merge` can derive a verifier only when
the issue includes an exact quoted or backticked single-token marker such as
`tau_docs_marker`. In that narrow case, Tau records the intake plan and uses
`git diff --check` plus a concrete `grep` command for the marker.

For bounded Rust or CLI issues, Tau can derive one focused verifier when the
repository has Cargo metadata, the issue text names an actual package, and the
issue also includes an exact quoted or backticked safe test filter such as
`spec_3810_repo_aware_verifier`. In that narrow case, the derived command is
`cargo test -p <package> <test-filter>` and it is persisted in the intake plan
before the job is created. Multi-word phrases, placeholders, broad requests,
unsafe requests, unresolved packages, and code issues without a concrete test
filter still require an explicit `--verifier-command`.

Run the same loop with built-in OpenRouter-compatible repair authority instead
of manual edits:

```bash
OPENROUTER_API_KEY=... cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- issue-to-merge \
  --state-dir .tau/autonomous-coding \
  --jobs-state-dir .tau/jobs \
  --repo-path /path/to/repo \
  --intake-id issue-123 \
  --mission-id issue-123 \
  --issue-url https://github.com/owner/repo/issues/123 \
  --issue-title "Issue title" \
  --issue-body "Issue body" \
  --verifier-command "cargo test -p some-crate spec_c01" \
  --provider-repair-openrouter \
  --provider-repair-attempts 3 \
  --provider-repair-model openrouter/qwen/qwen3-235b-a22b \
  --commit-message "Make verifier green"
```

The built-in adapter reads provider configuration from flags, process env, or
the nearest repo `.env`. API keys can be supplied as `OPENROUTER_API_KEY`,
`TAU_OPENROUTER_API_KEY`, `OPENAI_API_KEY`, or `TAU_API_KEY`. Model names can be
supplied with `--provider-repair-model`, `TAU_PROVIDER_REPAIR_MODEL`,
`TAU_AUTONOMOUS_CODING_REPAIR_MODEL`, `TAU_PROVIDER_PROOF_MODEL`, or
`TAU_OPENROUTER_MODEL`. Tau accepts `openrouter/<model>` as an operator-facing
model label and sends `<model>` to the OpenRouter-compatible API.

The adapter stores a sanitized metadata JSON file beside the durable repair
context. It includes provider, model, API base, auth source name, usage and
finish reason when returned, response hash, edit count, and failure summary when
relevant. It never writes the API key.

The command-adapter boundary is still available for other providers or local
experiments:

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

For custom adapters, Tau writes a sanitized repair context JSON file and invokes
the adapter with:

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
recovery counts, PR state, draft PR publication evidence, background job id and
last background reason, and auto-merge state.

The status JSON also classifies the operator action surface:

- `operator_state`: `running`, `stale_lease`, `needs_authority`,
  `safe_to_replay`, `blocked`, `failed`, or `complete`.
- `operator_next_command`: the next safe command Tau recommends, such as
  `tau-autonomous-coding-job recover` or `tau-autonomous-coding-job replay`.
- `replay_safe`, `recoverable`, `needs_authority`, and `stale_lease`: booleans
  for command-center displays.
- `mark_blocked_command`: a safe command for an inspected job that should not
  continue automatically.

To stop a stale or inspected job without hand-editing state:

```bash
cargo run -p tau-coding-agent --bin tau_autonomous_coding_job -- mark-blocked \
  --state-dir .tau/autonomous-coding \
  --job-id <job-id> \
  --reason-code operator_marked_blocked \
  --detail "operator inspected job and marked it blocked"
```

Issue intake uses deterministic local classification. Blocked intake records can
now report `unsafe`, `too_broad`, `underspecified`, `missing_verifier`,
`missing_edit_or_provider_authority`, `missing_credentials`, or `solvable`,
along with the exact verifier, edit/provider, or credential input still needed.
They also persist `verifier_plan`, `missing_inputs`, and `next_action_summary`
so a vague or no-authority issue becomes a concrete contract instead of a generic
failure. These plans are intentionally not mutation authority; Tau still blocks
until a verifier and edit/provider authority are supplied.

For repo-aware Rust or CLI verifier planning, blocked records distinguish
between a missing real Cargo package token and a missing exact safe test filter.
That means an operator can supply the one missing ingredient instead of
rewriting the whole verifier plan.
Those repo-aware gaps are also exposed as stable clarifying question reason
codes: `repo_aware_cargo_package` asks for an actual Cargo package present in
`cargo metadata`, and `repo_aware_test_filter` asks for an exact quoted or
backticked safe test filter token.

Intake records also expose a queue-friendly clarifying contract:

- `decision`: `ready_to_run`, `needs_authority`, `needs_clarification`,
  `split_required`, `blocked_unsafe`, `missing_credentials`, or `unknown`.
- `clarifying_questions`: structured rows with `reason_code`, `question`, and
  `required_input`.

This lets an operator or scheduler ask the exact missing question instead of
parsing prose. For example, an underspecified issue records questions for
`expected_behavior`, `current_behavior`, `affected_surface`, and
`verifier_command`; an over-broad issue asks for `bounded_surface` and
`single_acceptance_criterion`. A `ready_to_run` decision only appears when the
intake path already has verifier, edit/provider authority, and required
credentials.

For `needs_authority` records that already have the original issue body and a
concrete verifier command, `tau-unified intake <intake-id>` also prints
`rerun_command`. The command is shell-quoted, uses
`tau-autonomous-coding-job issue-to-merge`, supplies the persisted issue fields
and verifier command, and adds built-in OpenRouter provider-repair authority in
draft PR mode. Vague, unsafe, broad, missing-verifier, legacy, or placeholder
verifier records print `rerun_command=none`.

Draft PR behavior is evidence-backed. In draft mode Tau checks for an existing
PR for the job branch, updates it when found, creates a draft PR when GitHub auth
and `gh` are available, and records stdout/stderr/exit status in the mission
bundle. If auth is missing, the verified job remains PR-ready with
`draft_pr_missing_github_auth` and the exact manual `gh pr create --draft`
command.

The broadest deterministic benchmark for this path is:

```bash
CARGO_TARGET_DIR=/tmp/rust_pi-3805-target scripts/dev/test-real-repo-autonomous-coding-gauntlet.sh
```

It runs a temporary Tau worktree through docs-only, single-file, multi-file,
failing-test repair, CLI flag, flaky verifier, malformed provider rejection, and
stale-lease operator-status cases. The provider in this gauntlet is a bounded
fake adapter so the benchmark stays deterministic; live-provider proof remains
in the opt-in provider scripts.

## Boundaries

This loop prepares PR-ready evidence, can create or update a draft PR when draft
mode and GitHub auth are available, and can request GitHub auto-merge when
explicit policy/auth/PR URL gates pass. It does not bypass protected branches,
and it does not claim arbitrary issue solving without verifier commands plus
either operator-supplied edits or bounded provider repair authority. Crash-resume
and stuck-job recovery state is persisted, classified, and operable from
`tau-unified`; graphical command-center polish is still product work.
