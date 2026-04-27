# Plan: Issue #3691 - Add placeholder-jump editing ergonomics to tau-tui command palette

## Goal
Make tau-tui command palette editing fast enough for argument-heavy commands by
adding cursor-aware input and a deterministic placeholder jump operation.

## Approach
1. Inspect the existing command palette flow in `crates/tau-tui/src/interactive/`
   and keep the implementation local to the command-palette path.
2. Add a small command-palette editor model, or reuse the existing input editor
   pattern if it fits without broad refactoring.
3. Implement cursor-aware insert, backspace, delete, left, right, home, and end
   handling for focused command-palette input.
4. Implement placeholder jump for spans written as `<placeholder>` or
   `{placeholder}` so existing usage text can opt into the behavior without a
   separate template registry.
5. Add focused tau-tui tests with names containing `placeholder_jump` so the
   Gyre acceptance gate can run a narrow test filter.
6. Run tau-tui test, fmt, clippy, and root Cargo drift gates.

## Affected Modules
- `specs/3691/spec.md`
- `specs/3691/plan.md`
- `specs/3691/tasks.md`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- tau-tui interactive tests, likely near existing command/input tests

## Risks / Mitigations
- Risk: placeholder syntax is underspecified.
  Mitigation: support two common explicit forms, `<name>` and `{name}`, and
  keep the scanner pure and unit-tested.
- Risk: command execution changes accidentally.
  Mitigation: submit commands through the same `execute_command` path and add a
  regression test for an existing command.
- Risk: broad TUI input-editor refactor causes review churn.
  Mitigation: keep this slice command-palette-local unless reuse is trivial.
- Risk: keyboard choice conflicts with existing controls.
  Mitigation: use Tab for placeholder jump while the command palette is focused,
  where focused palette key handling already intercepts keys before global focus
  navigation.

## Verification
- `cargo test -p tau-tui placeholder_jump --lib`
- `cargo fmt --check`
- `cargo clippy -p tau-tui --tests --no-deps -- -D warnings`
- `git diff --quiet -- Cargo.toml Cargo.lock`