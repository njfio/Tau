# Issue 3793 Plan

## Approach

Patch the unified launcher command assembly to include Cargo's explicit binary
selector for the long-running Tau app. Add a shell regression assertion against
the recorded command file so the fake-runner contract catches this before a
real browser test has to discover it. For non-runner starts, detach the runtime
into its own session before returning from `up` so the pid remains valid in
non-interactive launch contexts.

## Affected Modules

- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `crates/tau-gateway/src/gateway_openresponses/server_bootstrap.rs`

## Risks

- Existing tests may assert exact command substrings around the package name.
- The real gateway may still fail later on runtime configuration after the
  Cargo binary selection issue is fixed.
- Background signal handling can differ between interactive terminals and
  non-interactive shells.

## Mitigations

- Preserve all existing arguments and only insert `--bin tau-coding-agent`
  before the Cargo argument separator.
- Run the launcher contract suite before live runtime validation.
- Validate actual loopback HTTP readiness and Browser Use after the patch.
- Treat Ctrl-C signal registration failure as non-fatal so inherited ignored
  signal dispositions do not immediately resolve graceful shutdown.

## Interfaces

No API, route, schema, or wire-format changes.
