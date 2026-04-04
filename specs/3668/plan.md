# Plan: Issue #3668 - Isolate CLI provider backends from repo context bleed

## Approach
1. Create a small helper in each CLI adapter that allocates an ephemeral temp
   directory for one request and sets the subprocess `current_dir` to it.
2. Keep the prompt rendering, output parsing, and textual tool-call promotion
   logic unchanged so the behavior shift is limited to cwd isolation.
3. Add adapter-level regressions that prove the subprocess cwd is isolated and
   that existing tool-call flows still pass.

## Proposed Design
### Ephemeral execution root
- Before spawning the CLI subprocess, create a `tempfile::TempDir`.
- Set `Command::current_dir(tempdir.path())` for the provider subprocess.
- Keep the tempdir alive until the subprocess has completed and outputs are
  consumed.

### Adapter coverage
- Add a codex regression whose mock script writes `pwd` into the output file so
  the test can assert the subprocess cwd is not the repo root.
- Keep the existing textual tool-call write-through regression intact and let it
  prove compatibility under the isolated cwd.
- Add equivalent cwd-isolation regressions for claude and gemini adapters.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3668"
  change_surface:
    - symbol: "CLI provider subprocess cwd"
      location: "crates/tau-provider/src/{codex_cli_client,claude_cli_client,gemini_cli_client}.rs"
      change_type: "modification"
      current: "provider subprocess inherits caller repository cwd"
      proposed: "provider subprocess runs from an isolated ephemeral temp directory"
      compatibility: "safe"
      reason: "provider shims are stateless backends and should not inherit workspace-local prompt context"
  overall_compatibility: "safe"
  approach:
    strategy: "isolate CLI provider subprocess cwd without changing prompt or parsing contracts"
    steps:
      - "add ephemeral tempdir-backed current_dir setup per adapter"
      - "add cwd-isolation regressions for codex/claude/gemini adapters"
      - "rerun provider adapter test suites"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: a CLI backend implicitly expects to run from the repo cwd.
  Mitigation: provider shims already receive the full conversation and tool
  contract explicitly; adapter tests verify normal text and tool-call flows stay
  intact.
- Risk: tempdir lifecycle bugs remove the execution root before the subprocess
  exits.
  Mitigation: keep the tempdir handle alive until after `wait_with_output`.

## Verification
- `cargo test -p tau-provider codex_cli_client -- --test-threads=1`
- `cargo test -p tau-provider claude_cli_client -- --test-threads=1`
- `cargo test -p tau-provider gemini_cli_client -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-provider/src/codex_cli_client.rs crates/tau-provider/src/claude_cli_client.rs crates/tau-provider/src/gemini_cli_client.rs`
