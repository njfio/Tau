# M320 - TUI agent interaction integration

Status: Active

## Context
M319 delivered one-command runtime lifecycle control, but `tau-unified tui`
still routes to a read-only dashboard watch surface. Operators cannot run agent
turns directly from that TUI entrypoint, so the program still feels fragmented.

M320 closes that gap by making the TUI command surface agent-capable while
retaining explicit access to live dashboard observability mode.

Primary sources:
- `scripts/run/tau-unified.sh`
- `crates/tau-tui/src/main.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `README.md`

## Issue Hierarchy
- Epic: #3537
- Story: #3538
- Task: #3539

## Scope
- Add an interactive `tau-tui` mode that launches the coding-agent runtime with
  inherited terminal IO.
- Route `scripts/run/tau-unified.sh tui` to the new interactive mode by
  default.
- Preserve access to the existing live dashboard watch shell mode as an
  explicit option.
- Add deterministic tests for arg parsing and launcher routing behavior.
- Update operator-facing docs for the unified interactive flow.

## Exit Criteria
- `specs/3539/spec.md` is `Implemented` with AC evidence.
- `scripts/run/tau-unified.sh tui` allows immediate operator prompts to the
  agent.
- Existing live dashboard watch flow remains available via explicit flag.
- TUI and launcher tests pass with deterministic contract coverage.
