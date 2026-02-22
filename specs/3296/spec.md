# Spec: Issue #3296 - wire APO scheduler and guarded prompt adoption into live RL runtime

Status: Implemented

## Problem Statement
`tau-algorithm` ships APO, but `tau-coding-agent` live runtime only executes PPO updates. This leaves prompt self-optimization disconnected from real session rewards and prevents autonomous prompt evolution from live traces.

## Scope
In scope:
- Add live-runtime APO scheduling path triggered during optimizer updates.
- Build deterministic APO datasets from live rollout spans (`prompt`, `assistant_text`, `reward`).
- Run significance gating before writing adopted prompts to training resources.
- Persist APO execution/adoption diagnostics in live optimizer reports.

Out of scope:
- Immediate in-session hot-swapping of `Agent` system prompt.
- New external API endpoints.
- Changes to PPO/GAE math.

## Acceptance Criteria
### AC-1 APO executes from live rollout traces when enabled
Given live RL runtime has succeeded rollouts with decision spans,
when optimizer update runs,
then APO executes with datasets derived from those spans and records APO report diagnostics.

### AC-2 prompt adoption is guarded by significance policy
Given APO returns a candidate prompt,
when candidate improvement is not statistically significant,
then runtime does not persist a new `system_prompt` resources update and reports non-adoption reason.

### AC-3 significant improvements persist prompt resources deterministically
Given APO candidate improvement passes significance gating,
when optimizer update completes,
then runtime persists a resources update containing prompt/version metadata and reports adoption.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | APO enabled and live succeeded rollouts with decision spans | optimizer update runs | APO report exists with `executed = true` |
| C-02 | AC-2 | Regression/Conformance | APO candidate with non-significant delta | optimizer update runs | no new adopted prompt resource; report reason reflects non-adoption |
| C-03 | AC-3 | Functional/Conformance | APO candidate with significant positive delta | optimizer update runs | resources update persists adopted `system_prompt` + version metadata |

## Success Metrics / Observable Signals
- `cargo test -p tau-coding-agent spec_c07_functional_live_optimizer_runs_apo_and_persists_prompt_resources`
- `cargo test -p tau-coding-agent spec_c08_regression_live_apo_skips_adoption_without_significant_improvement`
- `cargo test -p tau-coding-agent spec_c02_functional_optimizer_runs_on_update_interval`
- `cargo fmt --check`
- `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`
