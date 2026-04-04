const Phaser = window.Phaser;

const CELL_SIZE = 28;
const COLS = 10;
const ROWS = 18;
const BOARD_X = 28;
const BOARD_Y = 68;
const PANEL_X = BOARD_X + COLS * CELL_SIZE + 30;
const GAME_WIDTH = PANEL_X + 220;
const GAME_HEIGHT = BOARD_Y + ROWS * CELL_SIZE + 28;
const BG_COLOR = 0x05060c;
const PACMAN_STEP_MS = 150;
const GHOST_STEP_MS = 220;
const POWER_MODE_MS = 6000;
const GHOST_RESPAWN_MS = 3500;

// Arrow keys move Pac-Man. Tetromino controls stay on WASD + Space so both
// verbs can stay active in the same run instead of gating one system off.
const DIRECTIONS = [
  { name: "left", x: -1, y: 0, angle: Math.PI },
  { name: "right", x: 1, y: 0, angle: 0 },
  { name: "up", x: 0, y: -1, angle: -Math.PI / 2 },
  { name: "down", x: 0, y: 1, angle: Math.PI / 2 },
];

const TETROMINOES = [
  { name: "I", color: 0x46e0ff, matrix: [[1, 1, 1, 1]] },
  { name: "O", color: 0xffd449, matrix: [[1, 1], [1, 1]] },
  { name: "T", color: 0xc971ff, matrix: [[0, 1, 0], [1, 1, 1]] },
  { name: "S", color: 0x4ee26b, matrix: [[0, 1, 1], [1, 1, 0]] },
  { name: "Z", color: 0xff697d, matrix: [[1, 1, 0], [0, 1, 1]] },
  { name: "J", color: 0x6395ff, matrix: [[1, 0, 0], [1, 1, 1]] },
  { name: "L", color: 0xffad5a, matrix: [[0, 0, 1], [1, 1, 1]] },
];

function cloneMatrix(matrix) {
  return matrix.map((row) => [...row]);
}

function rotateMatrix(matrix) {
  return matrix[0].map((_, column) =>
    matrix.map((row) => row[column]).reverse(),
  );
}

function createBoard() {
  return Array.from({ length: ROWS }, () => Array(COLS).fill(null));
}

function cellKey(x, y) {
  return `${x},${y}`;
}

function wrapX(x) {
  return (x + COLS) % COLS;
}

function countOccupiedTopRows(board, rows = 4) {
  let occupied = 0;
  for (let y = 0; y < rows; y += 1) {
    if (board[y].some(Boolean)) {
      occupied += 1;
    }
  }
  return occupied;
}

class PacmanTetrisScene extends Phaser.Scene {
  constructor() {
    super("pacman-tetris");
  }

  create() {
    this.cameras.main.setBackgroundColor(BG_COLOR);

    this.boardGraphics = this.add.graphics();
    this.previewGraphics = this.add.graphics();
    this.overlayGraphics = this.add.graphics();

    this.titleText = this.add.text(PANEL_X, 26, "PACMAN\nTETRIS", {
      fontFamily: '"Trebuchet MS", "Avenir Next", sans-serif',
      fontSize: "28px",
      fontStyle: "bold",
      color: "#fff4b3",
      lineSpacing: 6,
    });

    this.statsText = this.add.text(PANEL_X, 118, "", {
      fontFamily: '"Trebuchet MS", "Avenir Next", sans-serif',
      fontSize: "18px",
      color: "#f6f2d8",
      lineSpacing: 10,
    });

    this.controlsText = this.add.text(
      PANEL_X,
      314,
      [
        "Arrow keys  move Pac-Man",
        "A / D       shift piece",
        "W           rotate piece",
        "S           soft drop",
        "Space       hard drop",
        "P           pause",
        "R           restart",
      ].join("\n"),
      {
        fontFamily: '"Courier New", monospace',
        fontSize: "15px",
        color: "#9fb1c7",
        lineSpacing: 7,
      },
    );

    this.bannerText = this.add
      .text(
        BOARD_X + (COLS * CELL_SIZE) / 2,
        BOARD_Y + (ROWS * CELL_SIZE) / 2,
        "",
        {
          fontFamily: '"Trebuchet MS", "Avenir Next", sans-serif',
          fontSize: "34px",
          fontStyle: "bold",
          color: "#fff4b3",
          align: "center",
          lineSpacing: 10,
        },
      )
      .setOrigin(0.5)
      .setDepth(20);

    this.flavorText = this.add.text(PANEL_X, 258, "", {
      fontFamily: '"Trebuchet MS", "Avenir Next", sans-serif',
      fontSize: "15px",
      color: "#ffcf8e",
      lineSpacing: 6,
    });

    this.nextLabel = this.add.text(PANEL_X, 206, "NEXT", {
      fontFamily: '"Trebuchet MS", "Avenir Next", sans-serif',
      fontSize: "14px",
      color: "#17d7c8",
      fontStyle: "bold",
      letterSpacing: 2,
    });

    this.input.keyboard.on("keydown-R", () => {
      if (this.gameOver) {
        this.scene.restart();
      }
    });

    this.keys = this.input.keyboard.addKeys({
      left: Phaser.Input.Keyboard.KeyCodes.LEFT,
      right: Phaser.Input.Keyboard.KeyCodes.RIGHT,
      up: Phaser.Input.Keyboard.KeyCodes.UP,
      down: Phaser.Input.Keyboard.KeyCodes.DOWN,
      pieceLeft: Phaser.Input.Keyboard.KeyCodes.A,
      pieceRight: Phaser.Input.Keyboard.KeyCodes.D,
      rotate: Phaser.Input.Keyboard.KeyCodes.W,
      softDrop: Phaser.Input.Keyboard.KeyCodes.S,
      hardDrop: Phaser.Input.Keyboard.KeyCodes.SPACE,
      pause: Phaser.Input.Keyboard.KeyCodes.P,
    });

    this.resetState();
  }

  resetState() {
    this.board = createBoard();
    this.powerPellets = new Set([
      cellKey(1, 3),
      cellKey(COLS - 2, 3),
      cellKey(1, ROWS - 3),
      cellKey(COLS - 2, ROWS - 3),
    ]);

    this.pellets = Array.from({ length: ROWS }, (_, y) =>
      Array.from({ length: COLS }, (_, x) => y >= 2 && y <= ROWS - 2),
    );

    this.pacman = {
      x: Math.floor(COLS / 2),
      y: ROWS - 2,
      dir: { ...DIRECTIONS[0] },
      desired: { ...DIRECTIONS[0] },
      poweredUntil: 0,
    };

    this.ghosts = [
      this.createGhost(1, 2, 0xff5d73),
      this.createGhost(COLS - 2, 2, 0x63b7ff),
    ];

    this.pellets[this.pacman.y][this.pacman.x] = false;
    for (const ghost of this.ghosts) {
      this.pellets[Math.round(ghost.y)][Math.round(ghost.x)] = false;
    }

    this.score = 0;
    this.lines = 0;
    this.level = 1;
    this.pelletsEaten = 0;
    this.ghostFreezeUntil = 0;
    this.pacmanAccumulator = 0;
    this.ghostAccumulator = 0;
    this.fallAccumulator = 0;
    this.gameOver = false;
    this.paused = false;
    this.overlayReason = "";
    this.nextPiece = this.randomPiece();
    this.activePiece = null;
    this.spawnPiece();
    this.refreshHud();
    this.draw(performance.now());
  }

  createGhost(x, y, color) {
    return {
      spawnX: x,
      spawnY: y,
      x,
      y,
      dir: { ...DIRECTIONS[1] },
      color,
      respawnUntil: 0,
    };
  }

  randomPiece() {
    const template =
      TETROMINOES[Math.floor(Math.random() * TETROMINOES.length)];
    return {
      name: template.name,
      color: template.color,
      matrix: cloneMatrix(template.matrix),
      x: Math.floor((COLS - template.matrix[0].length) / 2),
      y: 0,
    };
  }

  spawnPiece() {
    this.activePiece = this.nextPiece;
    this.activePiece.x = Math.floor(
      (COLS - this.activePiece.matrix[0].length) / 2,
    );
    this.activePiece.y = 0;
    this.nextPiece = this.randomPiece();

    if (!this.canPlace(this.activePiece.matrix, this.activePiece.x, this.activePiece.y)) {
      this.endGame("STACKED OUT");
    }
  }

  update(time, delta) {
    if (Phaser.Input.Keyboard.JustDown(this.keys.pause) && !this.gameOver) {
      this.paused = !this.paused;
    }

    if (this.paused) {
      this.overlayReason = "PAUSED";
      this.draw(time);
      return;
    }

    if (this.gameOver) {
      this.draw(time);
      return;
    }

    this.capturePacmanIntent();
    this.handleTetrominoInput();

    this.pacmanAccumulator += delta;
    while (this.pacmanAccumulator >= PACMAN_STEP_MS) {
      this.pacmanAccumulator -= PACMAN_STEP_MS;
      this.stepPacman(time);
    }

    this.ghostAccumulator += delta;
    if (time >= this.ghostFreezeUntil) {
      while (this.ghostAccumulator >= GHOST_STEP_MS) {
        this.ghostAccumulator -= GHOST_STEP_MS;
        this.stepGhosts(time);
      }
    } else {
      this.ghostAccumulator = 0;
    }

    this.fallAccumulator += delta;
    while (this.fallAccumulator >= this.getFallInterval()) {
      this.fallAccumulator -= this.getFallInterval();
      if (!this.tryMovePiece(0, 1)) {
        this.lockPiece(time);
      }
    }

    this.checkEncounters(time);
    this.refreshHud();
    this.draw(time);
  }

  capturePacmanIntent() {
    if (this.keys.left.isDown) {
      this.pacman.desired = { ...DIRECTIONS[0] };
    } else if (this.keys.right.isDown) {
      this.pacman.desired = { ...DIRECTIONS[1] };
    } else if (this.keys.up.isDown) {
      this.pacman.desired = { ...DIRECTIONS[2] };
    } else if (this.keys.down.isDown) {
      this.pacman.desired = { ...DIRECTIONS[3] };
    }
  }

  handleTetrominoInput() {
    if (Phaser.Input.Keyboard.JustDown(this.keys.pieceLeft)) {
      this.tryMovePiece(-1, 0);
    }

    if (Phaser.Input.Keyboard.JustDown(this.keys.pieceRight)) {
      this.tryMovePiece(1, 0);
    }

    if (Phaser.Input.Keyboard.JustDown(this.keys.rotate)) {
      this.tryRotatePiece();
    }

    if (Phaser.Input.Keyboard.JustDown(this.keys.softDrop)) {
      if (!this.tryMovePiece(0, 1)) {
        this.lockPiece(performance.now());
      }
    }

    if (Phaser.Input.Keyboard.JustDown(this.keys.hardDrop)) {
      while (this.tryMovePiece(0, 1)) {
        // Keep stepping until the piece settles.
      }
      this.lockPiece(performance.now());
    }
  }

  stepPacman(time) {
    const desired = this.pacman.desired;
    if (this.canEnterCell(this.pacman.x + desired.x, this.pacman.y + desired.y, true)) {
      this.pacman.dir = { ...desired };
    }

    const nextX = this.pacman.x + this.pacman.dir.x;
    const nextY = this.pacman.y + this.pacman.dir.y;

    if (this.canEnterCell(nextX, nextY, true)) {
      this.pacman.x = wrapX(nextX);
      this.pacman.y = nextY;
      this.collectPellet(time);
      this.checkEncounters(time);
    }
  }

  stepGhosts(time) {
    for (const ghost of this.ghosts) {
      if (time < ghost.respawnUntil) {
        continue;
      }

      const options = DIRECTIONS.filter((direction) =>
        this.canGhostEnter(ghost.x + direction.x, ghost.y + direction.y),
      );

      if (options.length === 0) {
        continue;
      }

      const reverseName = this.reverseDirectionName(ghost.dir.name);
      const filtered =
        options.length > 1
          ? options.filter((direction) => direction.name !== reverseName)
          : options;

      const candidates = filtered.length > 0 ? filtered : options;
      const frightened = this.isPowered(time);
      const target = { x: this.pacman.x, y: this.pacman.y };

      candidates.sort((a, b) => {
        const aDistance = this.gridDistance(
          wrapX(ghost.x + a.x),
          ghost.y + a.y,
          target.x,
          target.y,
        );
        const bDistance = this.gridDistance(
          wrapX(ghost.x + b.x),
          ghost.y + b.y,
          target.x,
          target.y,
        );
        return frightened ? bDistance - aDistance : aDistance - bDistance;
      });

      const chosen = candidates[0];
      ghost.dir = { ...chosen };
      ghost.x = wrapX(ghost.x + chosen.x);
      ghost.y += chosen.y;
    }
  }

  collectPellet(time) {
    if (!this.pellets[this.pacman.y]?.[this.pacman.x]) {
      return;
    }

    this.pellets[this.pacman.y][this.pacman.x] = false;
    this.pelletsEaten += 1;

    if (this.powerPellets.has(cellKey(this.pacman.x, this.pacman.y))) {
      this.score += 50;
      this.pacman.poweredUntil = time + POWER_MODE_MS;
      this.cameras.main.flash(140, 255, 246, 120);
    } else {
      this.score += 10;
    }
  }

  tryMovePiece(dx, dy) {
    const nextX = this.activePiece.x + dx;
    const nextY = this.activePiece.y + dy;

    if (!this.canPlace(this.activePiece.matrix, nextX, nextY)) {
      return false;
    }

    this.activePiece.x = nextX;
    this.activePiece.y = nextY;
    this.checkPieceCrush();
    return true;
  }

  tryRotatePiece() {
    const rotated = rotateMatrix(this.activePiece.matrix);
    for (const kick of [0, -1, 1, -2, 2]) {
      if (this.canPlace(rotated, this.activePiece.x + kick, this.activePiece.y)) {
        this.activePiece.matrix = rotated;
        this.activePiece.x += kick;
        this.checkPieceCrush();
        return true;
      }
    }
    return false;
  }

  canPlace(matrix, baseX, baseY) {
    for (let y = 0; y < matrix.length; y += 1) {
      for (let x = 0; x < matrix[y].length; x += 1) {
        if (!matrix[y][x]) {
          continue;
        }

        const boardX = baseX + x;
        const boardY = baseY + y;
        if (boardX < 0 || boardX >= COLS || boardY < 0 || boardY >= ROWS) {
          return false;
        }
        if (this.board[boardY][boardX]) {
          return false;
        }
      }
    }
    return true;
  }

  lockPiece(time) {
    for (const cell of this.activeCells()) {
      this.board[cell.y][cell.x] = this.activePiece.color;
      if (cell.x === this.pacman.x && cell.y === this.pacman.y) {
        this.endGame("CRUSHED");
      }
      for (const ghost of this.ghosts) {
        if (
          time >= ghost.respawnUntil &&
          cell.x === ghost.x &&
          cell.y === ghost.y
        ) {
          ghost.respawnUntil = time + GHOST_RESPAWN_MS;
          ghost.x = ghost.spawnX;
          ghost.y = ghost.spawnY;
        }
      }
    }

    const cleared = [];
    for (let y = 0; y < ROWS; y += 1) {
      if (this.board[y].every(Boolean)) {
        cleared.push(y);
      }
    }

    if (cleared.length > 0) {
      for (const row of [...cleared].sort((a, b) => b - a)) {
        this.board.splice(row, 1);
        this.board.unshift(Array(COLS).fill(null));
      }
      this.lines += cleared.length;
      this.level = 1 + Math.floor(this.lines / 4);
      this.score += [0, 120, 320, 560, 900][cleared.length] ?? 1200;
      this.ghostFreezeUntil = time + 900 + cleared.length * 250;
      this.cameras.main.flash(120, 255, 184, 77);
      this.cameras.main.shake(120, 0.005 + cleared.length * 0.001);
    }

    if (countOccupiedTopRows(this.board) >= 3) {
      this.cameras.main.flash(80, 255, 92, 92);
    }

    if (this.board[this.pacman.y][this.pacman.x]) {
      this.endGame("BOXED IN");
    }

    this.spawnPiece();
  }

  activeCells() {
    const cells = [];
    for (let y = 0; y < this.activePiece.matrix.length; y += 1) {
      for (let x = 0; x < this.activePiece.matrix[y].length; x += 1) {
        if (this.activePiece.matrix[y][x]) {
          cells.push({
            x: this.activePiece.x + x,
            y: this.activePiece.y + y,
          });
        }
      }
    }
    return cells;
  }

  canEnterCell(nextX, nextY, allowWrap = false) {
    if (allowWrap) {
      nextX = wrapX(nextX);
    }
    if (nextX < 0 || nextX >= COLS || nextY < 0 || nextY >= ROWS) {
      return false;
    }
    if (this.board[nextY][nextX]) {
      return false;
    }
    return !this.isActiveCell(nextX, nextY);
  }

  canGhostEnter(nextX, nextY) {
    nextX = wrapX(nextX);
    if (nextY < 0 || nextY >= ROWS) {
      return false;
    }
    if (this.board[nextY][nextX]) {
      return false;
    }
    return !this.isActiveCell(nextX, nextY);
  }

  isActiveCell(x, y) {
    return this.activeCells().some((cell) => cell.x === x && cell.y === y);
  }

  checkPieceCrush() {
    if (this.isActiveCell(this.pacman.x, this.pacman.y)) {
      this.endGame("CRUSHED");
    }
  }

  checkEncounters(time) {
    for (const ghost of this.ghosts) {
      if (time < ghost.respawnUntil) {
        continue;
      }
      if (ghost.x !== this.pacman.x || ghost.y !== this.pacman.y) {
        continue;
      }

      if (this.isPowered(time)) {
        ghost.respawnUntil = time + GHOST_RESPAWN_MS;
        ghost.x = ghost.spawnX;
        ghost.y = ghost.spawnY;
        this.score += 250;
        this.cameras.main.flash(90, 126, 196, 255);
      } else {
        this.endGame("GHOSTED");
      }
    }
  }

  refreshHud() {
    const remainingPellets = this.pellets.flat().filter(Boolean).length;
    const danger = countOccupiedTopRows(this.board);
    const powerLabel = this.isPowered(performance.now()) ? "HOT" : "OFF";
    const freezeLabel =
      performance.now() < this.ghostFreezeUntil ? "STUNNED" : "HUNTING";

    this.statsText.setText(
      [
        `Score   ${this.score.toString().padStart(5, " ")}`,
        `Lines   ${this.lines.toString().padStart(5, " ")}`,
        `Level   ${this.level.toString().padStart(5, " ")}`,
        `Pellets ${this.pelletsEaten.toString().padStart(5, " ")}`,
        `Left    ${remainingPellets.toString().padStart(5, " ")}`,
        `Power   ${powerLabel}`,
        `Ghosts  ${freezeLabel}`,
        `Danger  ${danger}/4`,
      ].join("\n"),
    );

    this.flavorText.setText(
      this.gameOver
        ? "Tetrominoes won.\nPress R to run it back."
        : this.isPowered(performance.now())
          ? "Power mode live.\nGhosts are edible."
          : "Line clears freeze\nghost pressure briefly.",
    );
  }

  getFallInterval() {
    return Math.max(170, 760 - (this.level - 1) * 55);
  }

  isPowered(time) {
    return time < this.pacman.poweredUntil;
  }

  reverseDirectionName(name) {
    return (
      {
        left: "right",
        right: "left",
        up: "down",
        down: "up",
      }[name] ?? "left"
    );
  }

  gridDistance(ax, ay, bx, by) {
    const direct = Math.abs(ax - bx);
    const wrapped = COLS - direct;
    return Math.min(direct, wrapped) + Math.abs(ay - by);
  }

  endGame(reason) {
    this.gameOver = true;
    this.overlayReason = reason;
  }

  draw(time) {
    this.boardGraphics.clear();
    this.previewGraphics.clear();
    this.overlayGraphics.clear();

    const pulse = 0.58 + (Math.sin(time / 170) + 1) * 0.16;
    const danger = countOccupiedTopRows(this.board);

    this.boardGraphics.fillStyle(0x091128, 0.96);
    this.boardGraphics.fillRoundedRect(
      BOARD_X - 10,
      BOARD_Y - 12,
      COLS * CELL_SIZE + 20,
      ROWS * CELL_SIZE + 24,
      18,
    );
    this.boardGraphics.lineStyle(4, danger >= 3 ? 0xff5d73 : 0x17d7c8, 0.85);
    this.boardGraphics.strokeRoundedRect(
      BOARD_X - 10,
      BOARD_Y - 12,
      COLS * CELL_SIZE + 20,
      ROWS * CELL_SIZE + 24,
      18,
    );

    for (let y = 0; y < ROWS; y += 1) {
      for (let x = 0; x < COLS; x += 1) {
        const pixelX = BOARD_X + x * CELL_SIZE;
        const pixelY = BOARD_Y + y * CELL_SIZE;
        this.boardGraphics.fillStyle(0x10182f, 0.9);
        this.boardGraphics.fillRoundedRect(
          pixelX + 2,
          pixelY + 2,
          CELL_SIZE - 4,
          CELL_SIZE - 4,
          8,
        );

        if (
          this.pellets[y][x] &&
          !this.board[y][x] &&
          !this.isActiveCell(x, y)
        ) {
          const powered = this.powerPellets.has(cellKey(x, y));
          this.boardGraphics.fillStyle(powered ? 0xffe07a : 0xf6f2d8, powered ? pulse : 0.82);
          this.boardGraphics.fillCircle(
            pixelX + CELL_SIZE / 2,
            pixelY + CELL_SIZE / 2,
            powered ? 5.4 + Math.sin(time / 120) * 0.9 : 2.7,
          );
        }

        if (this.board[y][x]) {
          this.drawBlock(this.boardGraphics, pixelX, pixelY, this.board[y][x], 1);
        }
      }
    }

    for (const cell of this.activeCells()) {
      const pixelX = BOARD_X + cell.x * CELL_SIZE;
      const pixelY = BOARD_Y + cell.y * CELL_SIZE;
      this.drawBlock(this.boardGraphics, pixelX, pixelY, this.activePiece.color, 0.94);
    }

    this.drawPacman(time);
    this.drawGhosts(time);
    this.drawNextPiece();
    this.drawOverlay();
  }

  drawBlock(graphics, pixelX, pixelY, color, alpha) {
    graphics.fillStyle(0x000000, 0.28 * alpha);
    graphics.fillRoundedRect(pixelX + 4, pixelY + 6, CELL_SIZE - 8, CELL_SIZE - 8, 8);
    graphics.fillStyle(color, alpha);
    graphics.fillRoundedRect(pixelX + 3, pixelY + 3, CELL_SIZE - 6, CELL_SIZE - 6, 8);
    graphics.fillStyle(0xffffff, 0.16 * alpha);
    graphics.fillRoundedRect(pixelX + 7, pixelY + 6, CELL_SIZE - 14, 6, 4);
  }

  drawPacman(time) {
    const centerX = BOARD_X + this.pacman.x * CELL_SIZE + CELL_SIZE / 2;
    const centerY = BOARD_Y + this.pacman.y * CELL_SIZE + CELL_SIZE / 2;
    const radius = CELL_SIZE * 0.39;
    const moving = this.pacman.dir.x !== 0 || this.pacman.dir.y !== 0;
    const mouth = moving ? 0.18 + (Math.sin(time / 90) + 1) * 0.09 : 0.16;
    const angle = this.pacman.dir.angle ?? 0;

    if (this.isPowered(time)) {
      this.boardGraphics.fillStyle(0xfff6bd, 0.12);
      this.boardGraphics.fillCircle(centerX, centerY, radius + 7 + Math.sin(time / 110) * 2);
    }

    this.boardGraphics.fillStyle(0xffd447, 1);
    this.boardGraphics.fillCircle(centerX, centerY, radius);

    const x1 = centerX + Math.cos(angle + mouth) * radius;
    const y1 = centerY + Math.sin(angle + mouth) * radius;
    const x2 = centerX + Math.cos(angle - mouth) * radius;
    const y2 = centerY + Math.sin(angle - mouth) * radius;

    this.boardGraphics.fillStyle(BG_COLOR, 1);
    this.boardGraphics.fillTriangle(centerX, centerY, x1, y1, x2, y2);
  }

  drawGhosts(time) {
    for (const ghost of this.ghosts) {
      if (time < ghost.respawnUntil) {
        continue;
      }

      const pixelX = BOARD_X + ghost.x * CELL_SIZE;
      const pixelY = BOARD_Y + ghost.y * CELL_SIZE;
      const color = this.isPowered(time) ? 0x4b7bff : ghost.color;

      this.boardGraphics.fillStyle(color, 0.96);
      this.boardGraphics.fillCircle(pixelX + CELL_SIZE * 0.35, pixelY + CELL_SIZE * 0.42, CELL_SIZE * 0.22);
      this.boardGraphics.fillCircle(pixelX + CELL_SIZE * 0.65, pixelY + CELL_SIZE * 0.42, CELL_SIZE * 0.22);
      this.boardGraphics.fillRoundedRect(
        pixelX + 5,
        pixelY + CELL_SIZE * 0.38,
        CELL_SIZE - 10,
        CELL_SIZE - 9,
        6,
      );

      this.boardGraphics.fillStyle(0xffffff, 1);
      this.boardGraphics.fillCircle(pixelX + CELL_SIZE * 0.4, pixelY + CELL_SIZE * 0.47, 4.4);
      this.boardGraphics.fillCircle(pixelX + CELL_SIZE * 0.6, pixelY + CELL_SIZE * 0.47, 4.4);
      this.boardGraphics.fillStyle(0x10182f, 1);
      this.boardGraphics.fillCircle(pixelX + CELL_SIZE * 0.42, pixelY + CELL_SIZE * 0.5, 1.8);
      this.boardGraphics.fillCircle(pixelX + CELL_SIZE * 0.62, pixelY + CELL_SIZE * 0.5, 1.8);
    }
  }

  drawNextPiece() {
    this.previewGraphics.fillStyle(0x091128, 0.96);
    this.previewGraphics.fillRoundedRect(PANEL_X, 226, 164, 70, 14);
    this.previewGraphics.lineStyle(2, 0x2d446d, 0.85);
    this.previewGraphics.strokeRoundedRect(PANEL_X, 226, 164, 70, 14);

    const matrix = this.nextPiece.matrix;
    const offsetX = PANEL_X + 20;
    const offsetY = 246 + Math.max(0, (2 - matrix.length) * 8);

    for (let y = 0; y < matrix.length; y += 1) {
      for (let x = 0; x < matrix[y].length; x += 1) {
        if (!matrix[y][x]) {
          continue;
        }
        this.drawPreviewBlock(
          offsetX + x * 22,
          offsetY + y * 22,
          this.nextPiece.color,
        );
      }
    }
  }

  drawPreviewBlock(x, y, color) {
    this.previewGraphics.fillStyle(color, 0.96);
    this.previewGraphics.fillRoundedRect(x, y, 18, 18, 5);
    this.previewGraphics.fillStyle(0xffffff, 0.16);
    this.previewGraphics.fillRoundedRect(x + 3, y + 3, 12, 4, 3);
  }

  drawOverlay() {
    if (!this.gameOver && !this.paused) {
      this.bannerText.setText("");
      return;
    }

    this.overlayGraphics.fillStyle(0x05060c, 0.62);
    this.overlayGraphics.fillRoundedRect(
      BOARD_X + 18,
      BOARD_Y + 160,
      COLS * CELL_SIZE - 36,
      126,
      22,
    );

    if (this.gameOver) {
      this.bannerText.setText(`${this.overlayReason}\nPress R to restart`);
    } else {
      this.bannerText.setText("PAUSED\nPress P to resume");
    }
  }
}

const game = new Phaser.Game({
  type: Phaser.AUTO,
  width: GAME_WIDTH,
  height: GAME_HEIGHT,
  parent: "game",
  backgroundColor: "#05060c",
  scene: [PacmanTetrisScene],
  render: {
    pixelArt: false,
    antialias: true,
  },
});

window.__PACMAN_TETRIS__ = game;
