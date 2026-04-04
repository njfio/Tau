# Plan: Issue #3745 - Add a Pac-Man + Tetris Phaser example under `examples/`

## Goal
Ship a self-contained Phaser 3 browser game example inside `examples/` that is
playable locally and clearly combines Pac-Man and Tetris mechanics.

## Approach
1. Create a new `examples/pacman-tetris/` folder with:
   - `index.html`
   - `src/main.js`
   - `README.md`
2. Build a single-scene Phaser prototype with:
   - pellet collection and power pellets
   - simple ghost pursuit
   - falling tetrominoes with rotation and line clears
   - score, instructions, and restart/pause overlays
3. Update `examples/README.md` and the root `README.md` to list the new example.
4. Validate the implementation with:
   - `node --check` for JavaScript syntax
   - a local static server + browser smoke test

## Affected Modules
- `examples/pacman-tetris/index.html`
- `examples/pacman-tetris/src/main.js`
- `examples/pacman-tetris/README.md`
- `examples/README.md`
- `README.md`
- `specs/milestones/m336/index.md`
- `specs/3745/spec.md`
- `specs/3745/plan.md`
- `specs/3745/tasks.md`

## Risks / Mitigations
- Risk: the mashup feels like two disconnected systems.
  Mitigation: keep both verbs live in one scene and let line clears and falling
  blocks directly change Pac-Man movement pressure.
- Risk: the example depends on new repo tooling.
  Mitigation: use a static HTML + JS scaffold with CDN-loaded Phaser.
- Risk: browser-only validation leaves syntax/runtime defects undiscovered.
  Mitigation: pair `node --check` with a local serve-and-open smoke test.

## Verification
- `node --check examples/pacman-tetris/src/main.js`
- `python3 -m http.server 4173 --directory examples/pacman-tetris`
- open `http://127.0.0.1:4173` in a browser automation pass and confirm the
  game renders and accepts controls
- `rg -n "pacman-tetris" README.md examples/README.md`
