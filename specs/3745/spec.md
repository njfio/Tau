# Spec: Issue #3745 - Add a Pac-Man + Tetris Phaser example under `examples/`

Status: Implemented

## Problem Statement
`examples/` currently demonstrates package, extension, and event assets, but it
does not show a self-contained browser build. A checked-in Phaser example would
give the repo a concrete greenfield artifact for game-style prompts and make the
examples area more representative of the project's bundled web-game guidance.

## Scope
In scope:
- a new `examples/pacman-tetris/` folder with a playable Phaser 3 prototype
- a coherent mashup loop that combines Pac-Man style pellet-chasing with
  falling Tetris pieces and line clears
- checked-in instructions or docs that explain how to run the example locally
- examples index updates so the new folder is discoverable

Out of scope:
- integrating the example into Rust crates or build pipelines
- asset pipelines, bundlers, or npm install flows
- production game polish beyond a playable demo loop

## Acceptance Criteria
### AC-1 The new example is self-contained inside `examples/pacman-tetris/`
Given a maintainer inspects the examples tree,
when they open `examples/pacman-tetris/`,
then they find a runnable Phaser 3 scaffold with `index.html`, a `src/main.js`
 game implementation, and local run instructions.

### AC-2 The gameplay loop mixes Pac-Man and Tetris mechanics in one playable scene
Given the example is served in a browser,
when the operator starts a run,
then they can steer Pac-Man with the arrow keys, manipulate falling tetrominoes
with separate controls, collect pellets, avoid ghosts, and trigger Tetris-style
line clears that affect the run.

### AC-3 The new example is documented in the repo's examples indexes
Given the checked-in docs,
when a maintainer scans the top-level examples references,
then the new `examples/pacman-tetris/` folder is mentioned alongside the other
example assets.

## Conformance Cases
- C-01 `examples/pacman-tetris/index.html`,
  `examples/pacman-tetris/src/main.js`, and
  `examples/pacman-tetris/README.md` exist. Maps to AC-1. Tier: Functional.
- C-02 Serving `examples/pacman-tetris/` locally yields a playable scene with
  separate Pac-Man and tetromino controls, pellets, ghosts, and line clears.
  Maps to AC-2. Tier: Conformance.
- C-03 `examples/README.md` and `README.md` mention
  `examples/pacman-tetris/`. Maps to AC-3. Tier: Functional.

## Success Metrics / Observable Signals
- the repo has a concrete browser-game example instead of only config-style
  assets under `examples/`
- the game is playable without any Rust build step
- the new folder is easy to discover from the existing examples docs

## Key Decisions
- keep the example dependency-light by loading Phaser from a CDN instead of
  adding a package manager workflow to the repo
- prefer a single-scene playable prototype over a larger asset-heavy game
- isolate the example in its own folder so it does not couple to unrelated
  product modules
