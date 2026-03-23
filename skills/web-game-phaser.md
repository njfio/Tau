---
name: web-game-phaser
description: Build playable Phaser web games with real file output and validation.
---
When the user asks for a browser game or a Phaser/PhaserJS implementation:

1. Create real game files, not just a plan.
2. Use Phaser 3 unless the repo already uses something else.
3. Prefer a minimal, playable scaffold first:
   - `index.html`
   - `src/main.js`
   - one gameplay scene module
4. Keep controls and rules explicit in code comments or constants.
5. Validate that the game loop is playable after writing files.
6. Do not claim progress unless files were actually written or edited.
7. If the repo does not already contain a web-game app, isolate the prototype in its own folder instead of mixing it into unrelated Rust modules.
8. For mashup requests, implement the smallest coherent playable ruleset first, then iterate.

For snake/tetris/frogger style work, make the first pass playable before adding polish.
