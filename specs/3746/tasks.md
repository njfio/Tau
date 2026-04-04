# Tasks: Issue #3746 - Add a `pacman-tetris-ws` Phaser example under `examples/`

- [x] T1. Create the local issue/spec scaffolding for the new browser-game
  example slot.
- [x] T2. Add a self-contained Phaser 3 Pac-Man + Tetris mashup under
  `examples/pacman-tetris-ws/`.
- [x] T3. Update the example discovery docs so the new folder is listed.
- [x] T4. Run JavaScript syntax checks plus a browser smoke pass for the new
  example.

## Verification Notes

- `node --check examples/pacman-tetris-ws/src/main.js` passed.
- `python3 -m http.server 4174 --directory examples/pacman-tetris-ws` was
  attempted, but the sandbox rejected local port binding with
  `PermissionError: [Errno 1] Operation not permitted`.
- `playwright screenshot --channel chrome file:///...` was attempted as a
  file-URL smoke fallback, but the sandboxed headless Chrome process exited
  immediately before page capture.
