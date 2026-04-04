# Pacman Tetris

`pacman-tetris/` is a self-contained Phaser 3 browser game example that mashes
up Pac-Man pellet chasing with falling Tetris pieces.

## Run locally

From the repository root:

```bash
python3 -m http.server 4173 --directory examples/pacman-tetris
```

Then open `http://127.0.0.1:4173`.

## Controls

- Arrow keys: move Pac-Man
- `A` / `D`: move the active tetromino
- `W`: rotate the active tetromino
- `S`: soft drop
- `Space`: hard drop
- `P`: pause
- `R`: restart after a game over

## Rules

- Eat pellets for points. The large power pellets let Pac-Man eat ghosts for a
  short burst.
- Falling tetrominoes become walls when they land. If a row fills, it clears
  and briefly freezes the ghosts.
- Getting touched by a ghost without power, or getting crushed by a falling
  piece, ends the run.
