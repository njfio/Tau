# Plan: Real-repo autonomous coding harness

## Approach

Extend the existing coding mission lifecycle instead of adding a parallel agent path. `CodingMissionRunner` already owns branch preparation, verifier evidence, controlled mutation, commit evidence, resume checkpoints, and PR-ready bundle generation. The product gap is that mutation input is single-file and fixture-shaped.

Add a backward-compatible `controlled_edits` vector to run/resume requests. Normalize the legacy `controlled_edit` field and the vector into one validated edit list, apply all edits before recording the checkpoint, and compute the mutation fingerprint after the full edit set. If any edit is invalid, the runner blocks before commit. This keeps the existing safety policy and state machine intact.

Update `tau_live_coding_loop_harness` so provider output may return either a legacy single edit or an `edits` array. Add a deterministic real-repo mode that runs against a repository worktree with a spec-derived verifier and emits PR-ready evidence. Real PR publication remains behind the existing draft-PR opt-in and GitHub auth check.

## Affected Modules

- `crates/tau-agent-core/src/coding_mission.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-coding-agent/src/bin/tau_live_coding_loop_harness.rs`
- `scripts/dev/test-full-autonomous-coding-loop.sh`
- `scripts/dev/test-provider-backed-autonomous-coding-loop.sh`
- `specs/3792-real-repo-autonomous-coding-harness/*`

## Risks

- Risk: multi-file edit support could partially write files before rejecting a later invalid path.
  - Mitigation: pre-validate every target path before writing any file in the edit set.

- Risk: resume fingerprints could miss one file and allow drift.
  - Mitigation: fingerprint after applying the whole edit set and add resume coverage for two edited files.

- Risk: provider schema expansion could break existing provider-proof responses.
  - Mitigation: keep the legacy single-edit shape accepted and test both payload forms.

- Risk: "real-repo" mode could accidentally mutate the developer's active checkout.
  - Mitigation: require an explicit worktree/repo path and keep validation scripts on temporary worktrees.

## Interfaces

`CodingMissionRunRequest` and `CodingMissionResumeRequest` gain:

- `controlled_edits: Vec<CodingMissionControlledEdit>`

The existing `controlled_edit: Option<CodingMissionControlledEdit>` field remains accepted for compatibility.

Provider edit JSON accepts either:

```json
{"relative_path":"path","contents":"...","reason_code":"..."}
```

or:

```json
{"edits":[{"relative_path":"path","contents":"...","reason_code":"..."}]}
```

The real-repo harness output must include verifier evidence, commit hash, PR-ready bundle path, and publication status. Draft PR creation remains opt-in through existing `allow_draft_pr` plus GitHub auth.
