# Plan: Issue #3626 - Route MCP skill catalog and install flows through tau-skills

Status: Implemented
Milestone: M329
Parent: #3623

## Compatibility Strategy
```yaml
implementation_strategy:
  task: "3626"
  change_surface:
    - symbol: "tau.skills_list / tau.skills_info naming and lookup behavior"
      location: "crates/tau-tools/src/mcp_server_runtime.rs"
      change_type: "modification"
      current: "Filesystem scan uses file stems, which misnames directory-backed SKILL.md skills"
      proposed: "Delegate to tau_skills::load_catalog so names/lookup match repo-standard skills loading"
      compatibility: "caution"
      reason: "Observable MCP metadata changes for nested skills, but the new behavior matches the canonical loader."
    - symbol: "tau.skills_install MCP install side effects"
      location: "crates/tau-tools/src/mcp_server_runtime.rs"
      change_type: "modification"
      current: "Copies markdown file directly and leaves no skills lockfile metadata"
      proposed: "Use tau-skills install flow and write/update skills.lock.json"
      compatibility: "caution"
      reason: "Adds lockfile side effects and richer install metadata; this is the intended repo-standard behavior."
  overall_compatibility: "caution"
  approach:
    strategy: "Direct implementation with regression coverage"
    steps:
      - "Add failing tests for nested SKILL.md naming drift and missing lockfile updates."
      - "Replace ad hoc catalog/list/info handling with tau_skills::load_catalog."
      - "Replace ad hoc install copy logic with tau_skills::install_skills plus lockfile helpers."
      - "Return structured MCP payloads that expose install counts and lockfile path."
    migration_guide: |
      MCP callers should treat tau.skills_list/info as canonical tau-skills views.
      Directory-backed skills may now resolve under their directory name rather
      than the literal `SKILL` file stem. tau.skills_install now writes
      skills.lock.json metadata alongside the installed skill.
    version_impact: "minor (pre-1.0) — parity correction to match canonical skill loading"
```

## Approach
1. Add RED coverage for the two concrete drift cases:
   - directory-backed `SKILL.md` naming/lookup
   - missing lockfile writes on install
2. Introduce small MCP adapter helpers that turn `tau-skills` catalog/install
   results into existing structured MCP payloads.
3. Keep search behavior aligned with the canonical catalog if it needs to move
   alongside list/info for consistency.
4. Verify with scoped `tau-tools` tests.

## Affected Modules
- `crates/tau-tools/src/mcp_server_runtime.rs`

## Risks / Mitigations
- Risk: list/info ordering or names change for some callers.
  - Mitigation: use regression tests that explicitly document the intended
    `tau-skills` behavior and keep payload shape stable.
- Risk: lockfile writes may fail on permissions or invalid sources.
  - Mitigation: convert install failures into structured MCP errors with clear
    messages, preserving the existing MCP error style.

## Verification Plan
- `CARGO_TARGET_DIR=/tmp/tau-target-3626 cargo test -p tau-tools skills_ -- --test-threads=1`
- `CARGO_TARGET_DIR=/tmp/tau-target-3626 cargo test -p tau-tools mcp_server_runtime -- --test-threads=1`

## Verification Result
- RED:
  - `CARGO_TARGET_DIR=/tmp/tau-target-3626 cargo test -p tau-tools regression_skills_ -- --nocapture`
- GREEN / VERIFY:
  - `rustfmt --check --edition 2021 crates/tau-tools/src/mcp_server_runtime.rs`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3626 cargo test -p tau-tools regression_skills_ -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3626 cargo test -p tau-tools mcp_server_runtime -- --test-threads=1`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3626 cargo test -p tau-skills install_skills -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3626 cargo test -p tau-skills skills_lockfile -- --nocapture`

## ADR
No ADR required. This is a source-of-truth alignment within the existing skills
subsystem.
