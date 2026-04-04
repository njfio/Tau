# Pacman Tetris WS

`pacman-tetris-ws/` is a self-contained Phaser 3 browser game example that
keeps Pac-Man pellet chasing and falling tetromino pressure active in the same
arena.

## Run locally

From the repository root:

```bash
python3 -m http.server 4174 --directory examples/pacman-tetris-ws
```

Then open `http://127.0.0.1:4174`.

## Controls

- Arrow keys: move Pac-Man
- `A` / `D`: move the active tetromino
- `W`: rotate the active tetromino
- `S`: soft drop
- `Space`: hard drop
- `P`: pause
- `R`: restart after a game over

## Rules

- Eat pellets for points. Power pellets briefly flip the hunt and let Pac-Man
  eat ghosts.
- Falling tetrominoes become walls when they land. Clearing rows freezes ghost
  pressure and boosts scoring.
- If a ghost tags Pac-Man outside power mode, or a piece lands on Pac-Man, the
  run ends.
