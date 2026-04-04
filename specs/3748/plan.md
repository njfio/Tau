# Plan: Issue #3748 - Add `examples/pacman-tetris-ws3` Phaser mashup example

Status: Reviewed

## Approach
Start from the established `pacman-tetris-ws2` structure so the new example
stays consistent with existing static-browser examples, then reshape it into a
distinct `ws3` variant with a new presentation layer and a few gameplay
mechanics that are easy to understand from the HUD: hold-piece swapping,
fruit-triggered tempo swings, and combo-driven scoring. Keep everything inside
the example directory so contributors can run it with a plain static file
server.

## Affected Modules
- `examples/pacman-tetris-ws3/index.html`
  - new browser shell, typography, framing copy, and serve instructions
- `examples/pacman-tetris-ws3/README.md`
  - local run instructions, controls, and rules
- `examples/pacman-tetris-ws3/src/main.js`
  - Phaser scene, board state, input handling, scoring, and rendering
- `examples/README.md`
  - index entry for the new example
- `specs/3748/`
  - local spec, plan, and tasks for this slice
- `specs/milestones/m337/index.md`
  - milestone container for browser example slices

## Contracts
- The example remains self-contained and browser-runnable with no build step
- Pac-Man and tetrominoes share one board and can interfere with each other
- WS3 introduces distinct mechanics instead of mirroring `ws`/`ws2`

## Risks
- Copying a sibling example too literally would make WS3 feel redundant
- Adding too many mechanics would make the example brittle and harder to verify
- Static examples have limited automated test infrastructure, so verification
  must rely on smoke checks and browser validation

## Verification Strategy
- Capture RED evidence that the `ws3` directory and docs entry do not exist yet
- Run `node --check examples/pacman-tetris-ws3/src/main.js`
- Serve `examples/pacman-tetris-ws3/` locally and load it in a browser
- Check browser console output and capture a screenshot
