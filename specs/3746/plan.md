# Plan: Issue #3746 - Add a `pacman-tetris-ws` Phaser example under `examples/`

## Goal
Ship a second self-contained Phaser 3 browser game example inside `examples/`
with a distinct Pac-Man + Tetris mashup and local run instructions.

## Approach
1. Reserve local issue/spec scaffolding under the existing browser-game
   milestone because remote issue creation is blocked by invalid `gh` auth.
2. Create a new `examples/pacman-tetris-ws/` folder with:
   - `index.html`
   - `src/main.js`
   - `README.md`
3. Build a single-scene Phaser prototype with:
   - Pac-Man movement on arrow keys
   - separate WASD + Space tetromino controls
   - pellets, power pellets, and simple ghost pursuit
   - line clears that freeze ghost pressure and affect scoring
   - a distinct arcade HUD and overlay treatment so the sibling example does
     not feel like a duplicate shell
4. Update `examples/README.md`, `README.md`, and the milestone index so the new
   example is discoverable.
5. Validate with JavaScript syntax checks plus a local browser smoke test.

## Affected Modules
- `examples/pacman-tetris-ws/index.html`
- `examples/pacman-tetris-ws/src/main.js`
- `examples/pacman-tetris-ws/README.md`
- `examples/README.md`
- `README.md`
- `specs/milestones/m336/index.md`
- `specs/3746/spec.md`
- `specs/3746/plan.md`
- `specs/3746/tasks.md`

## Risks / Mitigations
- Risk: the new example feels too close to `pacman-tetris/`.
  Mitigation: change the board size, HUD treatment, ghost count, and event
  feedback so the sibling example has its own identity.
- Risk: browser-only code can hide runtime defects.
  Mitigation: pair `node --check` with a local static server and automated
  browser smoke validation.
- Risk: doc updates collide with unrelated in-flight README edits.
  Mitigation: keep the README diff limited to one additional example bullet.

## Verification
- `node --check examples/pacman-tetris-ws/src/main.js`
- `python3 -m http.server 4174 --directory examples/pacman-tetris-ws`
- open `http://127.0.0.1:4174` in a browser automation pass and confirm the
  game renders plus responds to Pac-Man and tetromino controls
- `rg -n "pacman-tetris-ws" README.md examples/README.md`
