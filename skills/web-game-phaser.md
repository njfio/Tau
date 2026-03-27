---
name: web-game-phaser
description: Build playable browser games using Phaser 3. Use when the user asks to create a game, build a web game, or mentions Phaser/PhaserJS.
---

# Web Game — Phaser 3

## When to use
- User asks to "create a game", "build a game", "make a game"
- User mentions Phaser, PhaserJS, or browser-based games
- User asks for tetris, snake, pacman, platformer, or similar arcade games

## Workflow

### 1. Scaffold
Create a self-contained game in an isolated folder (e.g., `examples/<game-name>/`):
- `index.html` — single entry point, loads Phaser 3 from CDN

Prefer a single `index.html` file with inline JS for simple games. Only split files for complex projects.

### 2. Phaser CDN
```html
<script src="https://cdn.jsdelivr.net/npm/phaser@3.80.1/dist/phaser.min.js"></script>
```

### 3. Game Structure
```javascript
const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    physics: { default: 'arcade', arcade: { gravity: { y: 0 } } },
    scene: { preload, create, update }
};
new Phaser.Game(config);
```

### 4. Implementation Rules
- Make the game playable on first creation. No placeholder code.
- Implement keyboard controls (arrow keys or WASD) immediately.
- Add a game loop with win/lose conditions.
- Use Phaser's built-in graphics for prototyping (no external assets needed).
- For mashup games: implement the smallest coherent ruleset first, then iterate.
- Add a visible score display.

### 5. Verify
After creating files:
- Read the HTML file back to confirm it exists and has content.
- Check that the Phaser CDN script tag is included.
- Check that `new Phaser.Game(config)` is present.
- Verify the file has keyboard input handling.

### 6. Tell the user
- Which file(s) were created and their paths
- How to open it (`open examples/<game-name>/index.html`)
- What controls to use
