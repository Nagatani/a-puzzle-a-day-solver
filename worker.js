// WASMモジュールをインポート
import init, { solve_for_date } from '../pkg/solver.js';

async function run() {
  await init();

  self.onmessage = (event) => {
    const { month, day } = event.data;

    try {
      const startTime = performance.now();
      const solutions = solve_for_date(month, day);
      const endTime = performance.now();

      self.postMessage({
        solutions: solutions,
        duration: (endTime - startTime) / 1000,
      });
    } catch (e) {
      console.error("WASMでエラーが発生しました:", e);
      self.postMessage({ solutions: null, duration: 0 });
    }
  };
}

run();