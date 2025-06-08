const monthSelect = document.getElementById('month');
const daySelect = document.getElementById('day');
const solveButton = document.getElementById('solve-button');
const statusDiv = document.getElementById('status');
const solutionsContainer = document.getElementById('solutions-container');

// 日付選択肢を生成
for (let i = 1; i <= 12; i++) {
  const option = document.createElement('option');
  option.value = i;
  option.textContent = i;
  monthSelect.appendChild(option);
}
for (let i = 1; i <= 31; i++) {
  const option = document.createElement('option');
  option.value = i;
  option.textContent = i;
  daySelect.appendChild(option);
}

// 今日の日付をセット
const today = new Date();
monthSelect.value = today.getMonth() + 1;
daySelect.value = today.getDate();

// Web Workerを生成
const worker = new Worker('worker.js', { type: 'module' });

solveButton.addEventListener('click', () => {
  const month = parseInt(monthSelect.value);
  const day = parseInt(daySelect.value);

  solutionsContainer.innerHTML = '';
  statusDiv.textContent = '⏳ 探索を開始しています...';
  solveButton.disabled = true;

  worker.postMessage({ month, day });
});

worker.onmessage = (event) => {
  const { solutions, duration } = event.data;

  if (solutions && solutions.length > 0) {
    statusDiv.textContent = `🎉 ${solutions.length}個の解が見つかりました。(探索時間: ${duration.toFixed(2)}秒)`;
    displaySolutions(solutions);
  } else {
    statusDiv.textContent = '⚠️ この日付の解は見つかりませんでした。';
  }
  solveButton.disabled = false;
};

function displaySolutions(solutions) {
  const pieceColors = [
    '#E6194B', '#3CB44B', '#FFE119', '#4363D8', '#F58231', '#911EB4', '#46F0F0', '#F032E6'
  ];

  solutions.forEach(solution => {
    const grid = document.createElement('div');
    grid.className = 'solution-grid';
    solution.board.forEach((row, r_idx) => {
      row.forEach((cellId, c_idx) => {
        const cell = document.createElement('div');
        cell.className = 'cell';
        if (cellId > 0) {
          cell.style.backgroundColor = pieceColors[cellId - 1];
          // cell.textContent = cellId; // ピース番号を表示したい場合
        } else if (cellId === -1) {
          cell.className += ' date-hole';
        }
        grid.appendChild(cell);
      });
    });
    solutionsContainer.appendChild(grid);
  });
}