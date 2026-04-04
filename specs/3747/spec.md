# Spec: Issue #3746 - Add a `pacman-tetris-ws` Phaser example under `examples/`

Status: Implemented

## Problem Statement
`examples/` now has one browser-game demo, but it does not yet show a second
variant in the same design space. A sibling `pacman-tetris-ws/` example gives
the repo another self-contained Phaser 3 artifact and broadens the checked-in
game prompt surface without touching the Rust runtime.

## Scope
In scope:
- a new `examples/pacman-tetris-ws/` folder with a runnable Phaser 3 scaffold
- a playable mashup loop that keeps Pac-Man movement, ghosts, pellets, falling
  tetrominoes, and line clears active in one scene
- checked-in run instructions for the new folder
- examples index updates so the new folder is discoverable

Out of scope:
- changes to Rust crates, build pipelines, or npm workflows
- multiplayer or networked play
- asset pipelines beyond inline/CSS/canvas rendering

## Acceptance Criteria
### AC-1 The new example is self-contained inside `examples/pacman-tetris-ws/`
Given a maintainer inspects the examples tree,
when they open `examples/pacman-tetris-ws/`,
then they find `index.html`, `src/main.js`, and `README.md` with enough
instructions to run the example locally.

### AC-2 The gameplay loop combines Pac-Man and Tetris mechanics in one scene
Given the example is served in a browser,
when an operator starts a run,
then they can move Pac-Man with the arrow keys, control a falling tetromino
with separate keys, collect pellets, avoid or eat ghosts, and trigger
Tetris-style row clears that change the pressure of the run.

### AC-3 The new folder is discoverable from the checked-in example docs
Given the repository docs,
when a maintainer scans the example references,
then `examples/pacman-tetris-ws/` is listed alongside the other checked-in
example assets.

## Conformance Cases
- C-01 `examples/pacman-tetris-ws/index.html`,
  `examples/pacman-tetris-ws/src/main.js`, and
  `examples/pacman-tetris-ws/README.md` exist. Maps to AC-1. Tier:
  Functional.
- C-02 Serving `examples/pacman-tetris-ws/` locally yields a playable scene
  with Pac-Man movement, tetromino controls, pellets, ghosts, and line clears.
  Maps to AC-2. Tier: Conformance.
- C-03 `examples/README.md` and `README.md` mention
  `examples/pacman-tetris-ws/`. Maps to AC-3. Tier: Functional.

## Success Metrics / Observable Signals
- the repo includes a second browser-game example under `examples/`
- the new example remains runnable without a Rust build step
- maintainers can discover the new folder from the checked-in examples docs

## Key Decisions
- keep the example dependency-light by loading Phaser from a CDN
- ship a sibling variant instead of modifying the existing `pacman-tetris/`
  folder so both examples remain available
- keep the work isolated to `examples/` plus the supporting spec/docs slice
