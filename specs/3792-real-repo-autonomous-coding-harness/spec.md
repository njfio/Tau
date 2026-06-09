# Spec: Real-repo autonomous coding harness

Status: Reviewed

## Problem

Tau's autonomous coding loop can prove a single-file disposable fixture and can call a provider for that fixture, but that does not yet make it a general coding-agent harness over a real repository. The next product slice must let a coding mission apply a provider-supplied multi-file repair plan against a real worktree, run spec-derived verifiers, commit the verified diff, and prepare auditable PR output without claiming unattended production auto-merge.

## Scope

In:
- Accept multiple controlled edits in a single `CodingMissionRunner` run and resume path while preserving the legacy single-edit field.
- Reject any multi-file edit set that escapes the repository root before committing.
- Extend the live coding loop harness provider payload so providers can return the legacy single edit, a multi-edit `edits` array, or a compact `files` map, including JSON recovered from a markdown/prose wrapper.
- Add a real-repo/worktree integration mode that runs red verifier, provider/controlled multi-file fix, green verifier, commit, and PR-ready bundle.
- Add a direct runtime batch-write tool so prompt-mode agents can create a complete multi-file edit set in one tool call instead of relying on fragile one-file-at-a-time sequencing.
- Add a direct runtime batch-edit tool so prompt-mode agents can apply surgical exact-string patches across multiple files, including multiple ordered patches in the same file, without rewriting complete files.
- Record clear evidence for PR publication state: manual command or draft PR creation, depending on opt-in auth flags.

Out:
- Unattended production PR merge.
- Running arbitrary destructive commands outside the existing workspace command policy.
- A full long-running background scheduler UI.
- Broad dashboard polish unrelated to coding missions.

## Acceptance Criteria

AC-1: Given a coding mission request contains two valid controlled edits, when the runner executes after a red verifier, then both files are written, the green verifier passes, and the mission reaches `PrReady` with one commit containing both files.

AC-2: Given a multi-edit mission contains any path outside the repository root, when the runner executes, then the mission fail-closes without writing or committing the outside file.

AC-3: Given provider output contains an `edits` array, when the live coding loop harness resolves provider instructions, then all edits are applied through `CodingMissionRunner` and legacy single-edit provider output remains accepted.

AC-4: Given the harness is pointed at a real repository worktree with a spec-derived verifier, when it runs in real-repo mode, then it produces durable red/green verifier evidence, a commit, a PR-ready bundle, and a publication status that is honest about manual versus draft-PR creation.

AC-5: Given a multi-edit mission pauses after edit application, when it resumes, then resume restores the prepared branch, verifies the multi-file mutation fingerprint, completes the commit, and reaches `PrReady`.

AC-6: Given prompt-mode runtime receives a model tool call for `write_many`, when the tool call includes multiple in-root UTF-8 files, then Tau prevalidates the full batch, writes every file through the registered direct tool, and records the batch tool result in session history.

AC-7: Given prompt-mode runtime receives a model tool call for `edit_many`, when the tool call includes exact-string edits or a unified diff across one or more in-root existing files, then Tau prevalidates every path and match, applies edits in-memory in request order, writes only after the full batch succeeds, and records the batch edit result in session history.

## Conformance Cases

C-01 maps AC-1: `CodingMissionRunRequest` with `controlled_edits=[src/lib.rs, tests/spec.rs]` starts from a failing verifier and exits with both files changed and verifier status passed.

C-02 maps AC-2: `controlled_edits=[src/lib.rs, ../escape.txt]` returns a blocked outcome; `../escape.txt` is absent and no commit evidence is recorded.

C-03 maps AC-3: provider JSON with `{"edits":[...]}` or `{"files":{...}}` is accepted and converted into a multi-edit mission request; provider JSON with `{"relative_path":...,"contents":...}` remains accepted; fenced/prose-wrapped JSON is extracted before parsing.

C-04 maps AC-4: `tau_live_coding_loop_harness --mode real-repo` runs against a temporary worktree of this repository and writes an evidence report with red verifier failure, green verifier pass, commit hash, PR-ready body path, and publication status.

C-05 maps AC-5: a run stopped after `ApplyEdit` stores a checkpoint whose mutation fingerprint covers all edited files; resume completes from that checkpoint and records a verified commit.

C-06 maps AC-6: a prompt-mode model response containing one `write_many` tool call writes `index.html` and `src/main.js`, persists a `tool_name=write_many` session entry, and rejects escaped or duplicate paths before any file is written.

C-07 maps AC-7: a prompt-mode model response containing one `edit_many` tool call patches `index.html` and `src/main.js`, persists a `tool_name=edit_many` session entry, supports multiple ordered exact-string patches in one file, accepts unified-diff hunks for existing files, and rejects a missing later match or mismatched hunk before writing any earlier edit.

## Success Signals

- `cargo test -p tau-agent-core coding_mission -- --test-threads=1` covers multi-file run, escape rejection, and resume.
- `cargo test -p tau-coding-agent --bin tau_live_coding_loop_harness provider_backed -- --test-threads=1` covers provider multi-edit parsing.
- `cargo test -p tau-tools write_many -- --test-threads=1` covers direct batch-write registration, duplicate-path rejection, escape rejection, and multi-file writes.
- `cargo test -p tau-coding-agent integration_run_prompt_with_cancellation_promotes_textual_write_many_tool_call_and_persists_files -- --test-threads=1` covers prompt-loop execution of the direct batch tool.
- `cargo test -p tau-tools edit_many -- --test-threads=1` covers direct batch-edit registration, multi-file patching, same-file ordered patches, missing-match fail-closed behavior, and escape rejection.
- `cargo test -p tau-coding-agent integration_run_prompt_with_cancellation_promotes_textual_edit_many_tool_call_and_patches_files -- --test-threads=1` covers prompt-loop execution of exact-string batch patching.
- `cargo test -p tau-coding-agent integration_run_prompt_with_cancellation_promotes_textual_edit_many_unified_diff_tool_call_and_patches_files -- --test-threads=1` covers prompt-loop execution of unified-diff batch patching.
- A deterministic real-repo harness script passes without provider credentials.
- Provider-backed mode remains opt-in and can use OpenRouter/Qwen when `TAU_LIVE_PROVIDER_PROOF=1` and credentials are available.
