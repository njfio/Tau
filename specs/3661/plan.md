# Plan: Issue #3661 - Recycle stale tau-unified runtime after repo/runtime fingerprint changes

## Approach
1. Introduce a lightweight launcher fingerprint file in `.tau/unified` that is
   derived from the current repo/runtime launch inputs.
2. Reuse that fingerprint in both `up` and `tui` bootstrap so stale runtime
   detection happens before the launcher decides to short-circuit on an existing
   pid.
3. Keep the contract additive and shell-local: no gateway changes, no new
   long-lived daemons, and no behavioral change for same-fingerprint reuse.

## Proposed Design
### Fingerprint source
- Create `tau-unified.runtime-fingerprint` beside the existing pid/log/command
  files.
- Build the fingerprint from the current launch command plus current Git HEAD
  when available.
- Treat missing fingerprint files as stale so old runtime generations are
  recycled automatically after the launcher upgrade lands.

### Stale runtime handling
- Add a helper that evaluates:
  - pid file exists
  - process is alive
  - persisted fingerprint matches current fingerprint
- For mismatches:
  - emit a clear `tau-unified: recycling stale runtime ...` message
  - call `cmd_down`
  - continue into the normal `up` path

### TUI bootstrap path
- Reuse the same stale-runtime detection inside `bootstrap_runtime_for_tui`
  instead of returning early on any alive pid.
- Preserve the existing no-bootstrap runner-mode default and same-fingerprint
  fast path.

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3661"
  change_surface:
    - symbol: "tau-unified runtime bookkeeping"
      location: "scripts/run/tau-unified.sh"
      change_type: "addition"
      current: "pid/log/command files only"
      proposed: "pid/log/command files plus launcher fingerprint"
      compatibility: "safe"
      reason: "launcher-local metadata only"
    - symbol: "tau-unified runtime reuse behavior"
      location: "scripts/run/tau-unified.sh"
      change_type: "modification"
      current: "any alive pid is reused"
      proposed: "alive pid is reused only when fingerprint matches"
      compatibility: "safe"
      reason: "prevents stale runtime reuse and preserves same-fingerprint reuse"
  overall_compatibility: "safe"
  approach:
    strategy: "Add runtime freshness detection before launcher reuse"
    steps:
      - "persist fingerprint on successful up"
      - "recycle mismatched runtime on up"
      - "recycle mismatched runtime on tui bootstrap"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: false-positive mismatch causes unnecessary restarts.
  Mitigation: fingerprint only uses stable launcher inputs plus Git HEAD.
- Risk: old runtimes created before this change lack a fingerprint file.
  Mitigation: treat missing fingerprint as stale by design.
- Risk: runner-mode tests become brittle.
  Mitigation: keep all new behavior observable through runner log assertions.

## Verification
- `bash scripts/run/test-tau-unified.sh`
- `bash scripts/dev/test-root-just-launcher.sh`
- ad hoc launcher smoke: stale fingerprint triggers recycle message
