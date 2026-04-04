# Spec: Issue #3748 - Add `examples/pacman-tetris-ws3` Phaser mashup example

Status: Reviewed

## Problem Statement
The repository already carries checked-in browser examples, including multiple
Pac-Man + Tetris mashups, but there is no third widened variant under
`examples/pacman-tetris-ws3/`. That leaves the examples set missing the new
standalone Phaser slice the user requested and prevents contributors from
serving or iterating on a fresh self-contained mashup without touching the
existing examples.

## Scope
In scope:
- add a new self-contained `examples/pacman-tetris-ws3/` directory
- implement a Phaser 3 browser game that mixes Pac-Man movement and Tetris
  piece pressure in one shared board
- give the `ws3` slice distinct mechanics and presentation from the earlier
  examples
- add local run instructions and controls documentation
- update `examples/README.md`

Out of scope:
- changes to Rust crates or application runtime behavior
- asset pipelines, bundlers, or npm-based build steps
- introducing third-party dependencies beyond the Phaser CDN already used by
  sibling examples
- automated gameplay tests beyond smoke verification for this static example

## Acceptance Criteria
### AC-1 `pacman-tetris-ws3` exists as a self-contained browser example
Given a developer opens `examples/pacman-tetris-ws3/`,
when they inspect the example files,
then the directory contains an `index.html`, `README.md`, and `src/main.js`
that can be served statically and load Phaser from a CDN.

### AC-2 `pacman-tetris-ws3` delivers a real Pac-Man + Tetris mashup
Given the example is loaded in a browser,
when the player uses the keyboard,
then Pac-Man movement and tetromino controls operate on the same board and the
game exposes distinct WS3 mechanics beyond a straight copy, including hold and
bonus/combo play.

### AC-3 Example documentation points at the new slice
Given a contributor scans the examples index or the example README,
when they look for the new browser game,
then they can see what `pacman-tetris-ws3` is and how to serve it locally.

## Conformance Cases
- C-01 `examples/pacman-tetris-ws3/index.html`,
  `examples/pacman-tetris-ws3/README.md`, and
  `examples/pacman-tetris-ws3/src/main.js` exist and reference Phaser 3 via
  CDN. Maps to AC-1. Tier: Functional.
- C-02 The WS3 game source includes both Pac-Man movement controls and
  tetromino controls on one board plus WS3-specific mechanics for hold and
  fruit/combo play. Maps to AC-2. Tier: Functional.
- C-03 `examples/README.md` and the example README describe the new slice and
  include a local serve command. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- A contributor can start the example with `python3 -m http.server`
- The page loads without browser console errors
- The game reads as a distinct third variant rather than a renamed duplicate

## Key Decisions
- Keep the example self-contained and static so it matches the existing
  `examples/` workflow
- Reuse the Phaser CDN pattern from sibling examples instead of adding tooling
- Differentiate WS3 with a new visual shell, hold support, and fruit/combo
  scoring to justify a separate example directory
