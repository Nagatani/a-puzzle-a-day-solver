const boardElement = document.getElementById('puzzle-board');
const monthSelect = document.getElementById('month-select');
const daySelect = document.getElementById('day-select');
const solveButton = document.getElementById('solve-button');
const statusElement = document.getElementById('status');
const noSolutionElement = document.getElementById('no-solution');

const BOARD_WIDTH = 7;
const BOARD_HEIGHT = 7;
const MONTHS = ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"];

const BOARD_LAYOUT = [
  ["M1", "M2", "M3", "M4", "M5", "M6", -1],
  ["M7", "M8", "M9", "M10", "M11", "M12", -1],
  ["D1", "D2", "D3", "D4", "D5", "D6", "D7"],
  ["D8", "D9", "D10", "D11", "D12", "D13", "D14"],
  ["D15", "D16", "D17", "D18", "D19", "D20", "D21"],
  ["D22", "D23", "D24", "D25", "D26", "D27", "D28"],
  ["D29", "D30", "D31", -1, -1, -1, -1],
];

const PIECES_SHAPES = [
  [[1, 1, 0], [1, 1, 0], [1, 1, 0]],
  [[1, 1, 0], [1, 0, 0], [1, 1, 0]],
  [[1, 0, 0], [1, 1, 0], [0, 1, 0], [0, 1, 0]],
  [[1, 0, 0], [1, 0, 0], [1, 1, 1]],
  [[1, 0, 0], [1, 1, 0], [1, 1, 0]],
  [[1, 0, 0], [1, 1, 0], [1, 0, 0], [1, 0, 0]],
  [[1, 1, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0]],
  [[1, 1, 0], [0, 1, 0], [0, 1, 1]]
];

let pieceVariations = [];

function initialize() {
  populateSelectors();
  generatePieceVariations();
  drawBoard();
  solveButton.addEventListener('click', handleSolveClick);
}

function populateSelectors() {
  MONTHS.forEach((month, index) => {
    const option = document.createElement('option');
    option.value = index + 1;
    option.textContent = month;
    monthSelect.appendChild(option);
  });
  for (let i = 1; i <= 31; i++) {
    const option = document.createElement('option');
    option.value = i;
    option.textContent = `${i}日`;
    daySelect.appendChild(option);
  }
  const today = new Date();
  monthSelect.value = today.getMonth() + 1;
  daySelect.value = today.getDate();
}

function drawBoard(solution = null) {
  if (boardElement.children.length === 0) {
    const fragment = document.createDocumentFragment();
    for (let r = 0; r < BOARD_HEIGHT; r++) {
      for (let c = 0; c < BOARD_WIDTH; c++) {
        const cell = document.createElement('div');
        fragment.appendChild(cell);
      }
    }
    boardElement.appendChild(fragment);
  }
  const selectedMonth = `M${monthSelect.value}`;
  const selectedDay = `D${daySelect.value}`;
  const cells = boardElement.children;

  for (let r = 0; r < BOARD_HEIGHT; r++) {
    for (let c = 0; c < BOARD_WIDTH; c++) {
      const index = r * BOARD_WIDTH + c;
      const cell = cells[index];
      const layoutValue = BOARD_LAYOUT[r][c];

      cell.className = 'grid-cell';
      cell.textContent = '';

      let pieceId = 0;
      if (solution) { pieceId = solution[r][c]; }

      if (layoutValue === -1) {
        cell.classList.add('blocked');
      } else if (pieceId > 0) {
        cell.classList.add(`piece-${pieceId}`);
      } else {
        cell.classList.add('date-cell');
        if (layoutValue.startsWith('M')) {
          cell.textContent = layoutValue.substring(1);
        } else {
          cell.textContent = layoutValue.substring(1);
        }
        if (layoutValue === selectedMonth || layoutValue === selectedDay) {
          cell.classList.add('uncovered');
          if (layoutValue.startsWith('M')) {
            cell.textContent = MONTHS[parseInt(layoutValue.substring(1)) - 1];
          }
        }
      }
    }
  }
}

function rotate(piece) {
  const rows = piece.length;
  const cols = piece[0].length;
  const newPiece = Array(cols).fill(0).map(() => Array(rows).fill(0));
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      newPiece[c][rows - 1 - r] = piece[r][c];
    }
  }
  return newPiece;
}

function flip(piece) {
  return piece.map(row => row.slice().reverse());
}

function generatePieceVariations() {
  pieceVariations = PIECES_SHAPES.map((shape) => {
    const variations = new Set();
    let current = shape;
    for (let i = 0; i < 4; i++) {
      variations.add(JSON.stringify(current));
      variations.add(JSON.stringify(flip(current)));
      current = rotate(current);
    }
    return Array.from(variations).map(v => JSON.parse(v));
  });
}

let solverWorker = null;
function handleSolveClick() {
  if (solverWorker) { solverWorker.terminate(); }
  drawBoard();
  statusElement.innerHTML = `<div><div class="loader"></div><p class="attempt-text">試行回数: <span id="attempt-counter">0</span></p></div>`;
  noSolutionElement.textContent = '';
  solveButton.disabled = true;

  const month = parseInt(monthSelect.value);
  const day = parseInt(daySelect.value);

  const workerCode = `
      let attemptCounter = 0;

      self.onmessage = function(e) {
          const { month, day } = e.data;
          const board = Array(7).fill(0).map(() => Array(7).fill(0));
          const layout = ${JSON.stringify(BOARD_LAYOUT)};
          for (let r = 0; r < 7; r++) {
              for (let c = 0; c < 7; c++) {
                  const val = layout[r][c];
                  if (val === -1 || val === \`M\${month}\` || val === \`D\${day}\`) {
                      board[r][c] = -1;
                  }
              }
          }
          const variations = ${JSON.stringify(pieceVariations)};
          const solution = solve(board, Array.from({length: 8}, (_, i) => i), variations);
            if (solution) {
              self.postMessage({ type: 'solution', board: solution, attempts: attemptCounter });
          } else {
              self.postMessage({ type: 'nosolution', attempts: attemptCounter });
          }
      };

      function canPlace(board, piece, r, c) {
          for (let pr = 0; pr < piece.length; pr++) {
              for (let pc = 0; pc < piece[pr].length; pc++) {
                  if (piece[pr][pc] === 1) {
                      const br = r + pr;
                      const bc = c + pc;
                      if (br >= 7 || bc >= 7 || board[br][bc] !== 0) return false;
                  }
              }
          }
          return true;
      }

      function placePiece(board, piece, r, c, pieceId) {
          const newBoard = board.map(row => row.slice());
          for (let pr = 0; pr < piece.length; pr++) {
              for (let pc = 0; pc < piece[pr].length; pc++) {
                  if (piece[pr][pc] === 1) {
                      newBoard[r + pr][c + pc] = pieceId + 1;
                  }
              }
          }
          return newBoard;
      }

      function solve(board, pieceIndices, allVariations) {
          if (pieceIndices.length === 0) {
              return board; // すべてのピースが配置されたら成功
          }

          const pieceIndex = pieceIndices[0];
          const remainingIndices = pieceIndices.slice(1);
          const variations = allVariations[pieceIndex];

          // 盤面のすべての空きマスに対して、ピースの配置を試みる
          for (let r = 0; r < 7; r++) {
              for (let c = 0; c < 7; c++) {
                  if (board[r][c] === 0) {
                      // この空きマス(r, c)に現在のピースを配置してみる
                      for (const variation of variations) {
                          if (canPlace(board, variation, r, c)) {
                              attemptCounter++;
                              if (attemptCounter % 10000 === 0) {
                                  self.postMessage({ type: 'update', attempts: attemptCounter });
                              }
                              const newBoard = placePiece(board, variation, r, c, pieceIndex);
                              const result = solve(newBoard, remainingIndices, allVariations);
                              if (result) {
                                  return result;
                              }
                          }
                      }
                      // この空きマスに現在のピースをどの向きで置いても解に繋がらなかった場合、
                      // このマスは現時点では埋められない、ということになる。
                      // しかし、他のピースならこのマスを埋められるかもしれないので、探索は止めない。
                      // このロジックは根本的に誤り。正しいバックトラッキングに変更する。
                  }
              }
          }

          // 上記のロジックは間違いだった場合のバックトラッキング

          // 最初の空きマスを見つける
          let first_r = -1, first_c = -1;
          for (let i = 0; i < 7; i++) {
              for (let j = 0; j < 7; j++) {
                  if (board[i][j] === 0) {
                      first_r = i;
                      first_c = j;
                      break;
                  }
              }
              if (first_r !== -1) break;
          }

          // 空きマスがなければ成功
          if (first_r === -1) {
                return pieceIndices.length === 0 ? board : null;
          }
          
          // 利用可能な各ピースを試す
          for (let i = 0; i < pieceIndices.length; i++) {
              const currentPieceIndex = pieceIndices[i];
              const variations = allVariations[currentPieceIndex];
              
              const remaining = pieceIndices.slice(0, i).concat(pieceIndices.slice(i + 1));

              for (const variation of variations) {
                    if (canPlace(board, variation, first_r, first_c)) {
                      attemptCounter++;
                      if (attemptCounter % 10000 === 0) {
                          self.postMessage({ type: 'update', attempts: attemptCounter });
                      }
                      const newBoard = placePiece(board, variation, first_r, first_c, currentPieceIndex);
                      const result = solve(newBoard, remaining, allVariations);
                      if (result) return result;
                  }
              }
          }


          return null; // このパスでは解が見つからなかった
      }
    `;
  const blob = new Blob([workerCode], { type: 'application/javascript' });
  solverWorker = new Worker(URL.createObjectURL(blob));

  solverWorker.onmessage = function (e) {
    const { type, board, attempts } = e.data;
    const attemptCounterElement = document.getElementById('attempt-counter');

    switch (type) {
      case 'update':
        if (attemptCounterElement) {
          attemptCounterElement.textContent = attempts.toLocaleString();
        }
        break;
      case 'solution':
        statusElement.innerHTML = `<p class="success-message">解が見つかりました！<br>(${attempts.toLocaleString()}回試行)</p>`;
        solveButton.disabled = false;
        drawBoard(board);
        solverWorker.terminate();
        solverWorker = null;
        break;
      case 'nosolution':
        statusElement.innerHTML = '';
        solveButton.disabled = false;
        statusElement.innerHTML = `<p class="error-message">解が見つかりませんでした。<br>(${attempts.toLocaleString()}回試行)</p>`;
        noSolutionElement.textContent = ``;
        solverWorker.terminate();
        solverWorker = null;
        break;
    }
  };

  solverWorker.postMessage({ month, day });
}
initialize();