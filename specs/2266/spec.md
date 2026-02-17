# Spec #2266

Status: Accepted
Milestone: specs/milestones/m46/index.md
Issue: https://github.com/njfio/Tau/issues/2266

## Problem Statement

Tau daemon supports systemd-user profile operations, but the systemd unit contract
is not tracked with explicit conformance cases and misses edge-case safety for
`ExecStart` when executable paths include spaces. This risks broken managed runtime
deployments on hosts with non-trivial install paths.

## Scope

In scope:

- Define explicit conformance coverage for systemd unit rendering and operational
  daemon lifecycle behavior.
- Ensure rendered systemd `ExecStart` safely handles executable paths with spaces.
- Keep launchd and existing daemon lifecycle semantics unchanged.

Out of scope:

- Root/system-wide systemd units (`/etc/systemd/system`).
- External service manager integration (`systemctl --user` invocation automation).
- Daemon protocol/runtime behavior unrelated to service unit generation.

## Acceptance Criteria

- AC-1: Given `render_systemd_user_unit`, when called for standard executable/state
  paths, then output includes required `[Unit]`, `[Service]`, and `[Install]` sections
  with gateway startup flags.
- AC-2: Given an executable path containing spaces, when rendering systemd unit,
  then `ExecStart` encodes executable path safely so systemd tokenization does not
  split the binary path.
- AC-3: Given systemd-user daemon lifecycle (`install/start/stop/uninstall`), when
  operations run, then service files/pid/state transitions are consistent and
  inspectable.
- AC-4: Given repeated status inspection after lifecycle changes, when inspecting
  daemon state, then reports remain deterministic and regression-safe.

## Conformance Cases

- C-01 (AC-1, unit): rendered systemd unit includes expected command and gateway
  paths.
- C-02 (AC-2, regression): rendered systemd unit escapes/quotes executable path with
  spaces in `ExecStart`.
- C-03 (AC-3, functional): lifecycle install/start/stop/uninstall roundtrip passes
  for `CliDaemonProfile::SystemdUser`.
- C-04 (AC-4, integration): status report fields are stable across repeated
  inspection.

## Success Metrics / Observable Signals

- `tau-ops` tests include explicit `spec_c0x_*` coverage for systemd template and
  lifecycle behavior.
- Service unit output remains backward compatible for standard paths while covering
  path-with-space safety.
