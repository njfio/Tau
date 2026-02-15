# M23 Doc Density Gate Artifact

- Generated at: `2026-02-15T18:58:07Z`
- Repo root: `/Users/n/RustroverProjects/rust_pi_mainline`

## Command

```bash
python3 .github/scripts/rust_doc_density.py --repo-root /Users/n/RustroverProjects/rust_pi_mainline --targets-file docs/guides/doc-density-targets.json --json
```

## Versions

| Tool | Version |
| --- | --- |
| python3 | `Python 3.14.2` |
| rustc | `rustc 1.90.0 (1159e78c4 2025-09-14) (Homebrew)` |
| cargo | `cargo 1.90.0 (Homebrew)` |
| gh | `gh version 2.61.0 (2024-11-06)` |
| jq | `jq-1.7.1` |

## Context

| Field | Value |
| --- | --- |
| OS | `Darwin 25.0.0 arm64` |
| Git commit | `c2136ccb43ff4f4f6500021226eb5964323f444a` |
| Git branch | `codex/issue-1654-wave2-doc-uplift-v2` |
| Git dirty | `true` |

## Summary

- Overall documented/public: `1076/2193`
- Overall percent: `49.07%`
- Issues reported by density checker: `0`

## Crate Breakdown

| Crate | Documented | Public | Percent |
| --- | ---: | ---: | ---: |
| tau-github-issues | 26 | 114 | 22.81% |
| tau-session | 27 | 117 | 23.08% |
| tau-access | 17 | 69 | 24.64% |
| tau-training-store | 2 | 8 | 25.0% |
| tau-release-channel | 11 | 42 | 26.19% |
| tau-onboarding | 60 | 224 | 26.79% |
| tau-custom-command | 13 | 45 | 28.89% |
| tau-tui | 13 | 45 | 28.89% |
| tau-cli | 30 | 95 | 31.58% |
| tau-deployment | 18 | 55 | 32.73% |
| kamn-sdk | 2 | 6 | 33.33% |
| tau-skills | 39 | 117 | 33.33% |
| tau-diagnostics | 18 | 51 | 35.29% |
| tau-orchestrator | 19 | 50 | 38.0% |
| tau-safety | 7 | 18 | 38.89% |
| tau-browser-automation | 16 | 41 | 39.02% |
| tau-runtime | 71 | 171 | 41.52% |
| tau-algorithm | 7 | 16 | 43.75% |
| tau-ai | 22 | 50 | 44.0% |
| tau-tools | 40 | 90 | 44.44% |
| tau-extensions | 15 | 33 | 45.45% |
| kamn-core | 3 | 6 | 50.0% |
| tau-trainer | 3 | 6 | 50.0% |
| tau-training-types | 19 | 38 | 50.0% |
| tau-dashboard | 18 | 35 | 51.43% |
| tau-startup | 13 | 25 | 52.0% |
| tau-voice | 25 | 48 | 52.08% |
| tau-memory | 31 | 57 | 54.39% |
| tau-training-runner | 5 | 9 | 55.56% |
| tau-events | 22 | 35 | 62.86% |
| tau-training-tracer | 6 | 8 | 75.0% |
| tau-agent-core | 41 | 52 | 78.85% |
| tau-coding-agent | 0 | 0 | 100.0% |
| tau-contract | 5 | 5 | 100.0% |
| tau-core | 6 | 6 | 100.0% |
| tau-gateway | 70 | 70 | 100.0% |
| tau-github-issues-runtime | 5 | 5 | 100.0% |
| tau-multi-channel | 147 | 147 | 100.0% |
| tau-ops | 48 | 48 | 100.0% |
| tau-provider | 127 | 127 | 100.0% |
| tau-slack-runtime | 5 | 5 | 100.0% |
| tau-training-proxy | 4 | 4 | 100.0% |

## Troubleshooting

1. Compare the rendered command and `targets_file` field with CI to rule out mismatched thresholds.
2. Compare tool versions (`python3`, `rustc`, `cargo`) when count output changes unexpectedly.
3. Re-run on a clean worktree at the same commit when local dirty state is `true`.

## Reproduction Command

```bash
python3 .github/scripts/rust_doc_density.py --repo-root /Users/n/RustroverProjects/rust_pi_mainline --targets-file docs/guides/doc-density-targets.json --json
```
