// Wasm-bindgenは、RustとJavaScript間の通信を容易にするための公式ツール。
// preludeをインポートすることで、#[wasm_bindgen]などの主要な機能が使えるようになる。
use wasm_bindgen::prelude::*;

// SerdeはRustの強力なシリアライズ/デシリアライズフレームワーク。
// ここでは、Rustのデータ構造（struct）をJavaScriptが理解できる形式（JSONのようなJsValue）に変換するために使う。
use serde::Serialize;

// Rustのコードがパニック（実行時エラー）を起こした際に、
// ブラウザの開発者コンソールに詳細なエラーメッセージを出力してくれる便利なデバッグツール。
use console_error_panic_hook;

use std::collections::HashSet;


// --- データ構造 ---

/// JavaScript側に渡すための、 một つの解を表す構造体。
/// #[derive(Serialize)] を付けることで、この構造体をSerdeが自動的に
/// シリアライズ（JsValueに変換）できるようになる。
#[derive(Serialize)]
struct Solution {
    /// ピースのID(1-8)で埋められた7x7の盤面。日付の穴は-1で表現。
    board: Vec<Vec<i8>>,
}

/// Union-Find木（素集合データ構造）。
/// 盤面上に残った空きマスが、連結したいくつかのグループ（島）を形成しているかを管理する。
/// これは「枝刈り」と呼ばれる探索アルゴリズムの重要な最適化のために使用される。
/// 例えば、空きマスが3マスだけの孤立した島になった場合、どのピース（最小5マス）でも
/// 埋めることは不可能。このような手詰まりの盤面を早期に検出し、無駄な探索を打ち切ることで
/// 計算量を劇的に削減する。
struct UnionFind {
    parents: Vec<i32>,
    n: usize,
}
impl UnionFind {
    /// n個の要素を持つUnion-Find木を初期化する。最初は全要素が別々のグループ。
    fn new(n: usize) -> Self { UnionFind { parents: vec![-1; n], n } }

    /// 要素xが属するグループの根（代表元）を見つける。
    /// 途中で見つかった要素を直接根に繋ぎ直す「経路圧縮」という最適化を行っている。
    fn find(&mut self, x: usize) -> usize {
        if self.parents[x] < 0 { x } else {
            let root = self.find(self.parents[x] as usize);
            self.parents[x] = root as i32;
            root
        }
    }

    /// 要素xとyが属するグループを統合する。
    fn union(&mut self, x: usize, y: usize) {
        let mut root_x = self.find(x);
        let mut root_y = self.find(y);
        if root_x != root_y {
            // サイズが小さい方を大きい方に繋ぐことで、木の高さが偏らないようにする最適化。
            if self.parents[root_x] > self.parents[root_y] { std::mem::swap(&mut root_x, &mut root_y); }
            self.parents[root_x] += self.parents[root_y];
            self.parents[root_y] = root_x as i32;
        }
    }
    
    /// 要素xが属するグループのサイズ（要素数）を返す。
    fn size(&mut self, x: usize) -> i32 { let root = self.find(x); -self.parents[root] }
    
    /// 全ての根をリストで返す。
    fn roots(&self) -> Vec<usize> { (0..self.n).filter(|&i| self.parents[i] < 0).collect() }

    /// 要素xが属するグループの全メンバーをリストで返す。
    fn members(&mut self, x: usize) -> Vec<usize> {
        let root = self.find(x);
        (0..self.n).filter(|&i| self.find(i) == root).collect()
    }
}

// --- ピース操作 ---

/// 全8ピースの基本形状を定義する。
fn get_initial_pieces() -> Vec<Vec<Vec<u8>>> {
    vec![
        vec![vec![1, 1], vec![1, 1], vec![1, 1]],
        vec![vec![1, 1, 0], vec![1, 0, 0], vec![1, 1, 0]],
        vec![vec![1, 0, 0], vec![1, 1, 0], vec![0, 1, 0], vec![0, 1, 0]],
        vec![vec![1, 0, 0], vec![1, 0, 0], vec![1, 1, 1]],
        vec![vec![1, 0, 0], vec![1, 1, 0], vec![1, 1, 0]],
        vec![vec![1, 0, 0], vec![1, 1, 0], vec![1, 0, 0], vec![1, 0, 0]],
        vec![vec![1, 1, 0], vec![1, 0, 0], vec![1, 0, 0], vec![1, 0, 0]],
        vec![vec![1, 1, 0], vec![0, 1, 0], vec![0, 1, 1]],
    ]
}

/// ピースの形状データを回転・反転させる。
fn rotate_and_flip(shape: &Vec<Vec<u8>>, rot_type: u8) -> Vec<Vec<u8>> {
    let mut current_shape = shape.clone();
    // 4以上ならまず左右反転
    if rot_type >= 4 { for row in &mut current_shape { row.reverse(); } }
    let k = rot_type % 4; // 0, 90, 180, 270度の回転
    if k == 0 { return current_shape; }
    let (h, w) = (current_shape.len(), current_shape[0].len());
    let mut rotated = if k % 2 == 1 { vec![vec![0; h]; w] } else { vec![vec![0; w]; h] };
    for i in 0..h { for j in 0..w { match k {
        1 => rotated[j][h - 1 - i] = current_shape[i][j],
        2 => rotated[h - 1 - i][w - 1 - j] = current_shape[i][j],
        3 => rotated[w - 1 - j][i] = current_shape[i][j],
        _ => {}
    }}}
    rotated
}

/// ピースの回転・反転から、重複しない形状パターンをすべて生成する。
fn get_unique_rotations(shape: &Vec<Vec<u8>>) -> Vec<Vec<Vec<u8>>> {
    let mut unique_shapes = Vec::new(); let mut seen = HashSet::new();
    for i in 0..8 {
        let rotated = rotate_and_flip(shape, i);
        if seen.insert(rotated.clone()) { unique_shapes.push(rotated); }
    }
    unique_shapes
}

/// 2次元の盤面を64ビット整数（ビットマスク）に変換する。
/// この変換により、非常に高速なビット演算が可能になる。
/// - ピースが重なっているかの判定 → ビットごとのAND演算 (`&`)
/// - ピースを盤面に置く操作 → ビットごとのOR演算 (`|`)
/// これらは2次元配列をループで操作するより桁違いに速い。
fn board_to_bitmask(board: &Vec<Vec<u8>>) -> u64 {
    let mut mask = 0u64;
    for (i, row) in board.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell == 1 { mask |= 1 << (i * 7 + j); }
        }
    }
    mask
}

// --- コアアルゴリズム ---

/// 枝刈り（Pruning）判定関数。盤面が手詰まりかどうかを調べる。
fn judge_connected_component(board_mask: u64, is_size_6_piece_used: bool) -> bool {
    let mut uf = UnionFind::new(49);
    // 空きマス(`0`)をUnion-Findでグループ分けする
    for i in 0..49 {
        if (board_mask >> i) & 1 == 0 { // マスが空いているか
            if (i + 1) % 7 != 0 && (board_mask >> (i + 1)) & 1 == 0 { uf.union(i, i + 1); }
            if i < 42 && (board_mask >> (i + 7)) & 1 == 0 { uf.union(i, i + 7); }
        }
    }
    // 各空きマスグループ（島）のサイズをチェック
    for root in uf.roots() {
        if (board_mask >> root) & 1 == 0 {
            let size = uf.size(root);
            if size < 5 { return false; } // どのピースよりも小さい島は手詰まり
            
            // サイズ6のピースが未使用の場合: 島は5の倍数か、(6 + 5の倍数)で構成可能
            if !is_size_6_piece_used {
                if size % 5 != 0 && (size < 6 || (size - 6) % 5 != 0) { return false; }
            } else { // サイズ6のピースが使用済みの場合: 島は5の倍数でなければならない
                if size % 5 != 0 { return false; }
            }
        }
    }
    true // 手詰まりではない
}

/// バックトラッキング（深さ優先探索）で全解法を探索する再帰関数。
/// 1. ピースを1つ置く
/// 2. 盤面が妥当かチェック（枝刈り）
/// 3. 妥当なら、次のピースのために自分自身を呼び出す（深く潜る）
/// 4. 探索が終わったら、置いたピースを元に戻し（バックトラック）、別の置き方を試す
fn find_solutions_recursive(
    piece_idx: usize, current_board_mask: u64, used_placements: &mut Vec<u64>,
    is_size_6_piece_used: bool, all_piece_placements: &Vec<Vec<u64>>,
    size_6_piece_index: usize, solutions: &mut Vec<Vec<u64>>,
) {
    // ベースケース: 全8ピースを配置できたら解として保存
    if piece_idx == 8 { solutions.push(used_placements.clone()); return; }

    // 現在のピースの全ての配置パターンを試す
    for &placement_mask in &all_piece_placements[piece_idx] {
        // 高速なビット演算で、ピースが既存の盤面と重ならないかチェック
        if (current_board_mask & placement_mask) == 0 {
            let new_board_mask = current_board_mask | placement_mask;
            let new_is_size_6_used = is_size_6_piece_used || (piece_idx == size_6_piece_index);
            
            // 枝刈り: この配置で手詰まりにならないかチェック
            if judge_connected_component(new_board_mask, new_is_size_6_used) {
                used_placements.push(placement_mask); // ピースを配置
                // 再帰呼び出しで次のピースの探索へ
                find_solutions_recursive(
                    piece_idx + 1, new_board_mask, used_placements, new_is_size_6_used,
                    all_piece_placements, size_6_piece_index, solutions,
                );
                used_placements.pop(); // バックトラック: 最後のピース配置を取り消す
            }
        }
    }
}


/// WASMとしてJavaScriptに公開されるメイン関数。
/// `#[wasm_bindgen]` アトリビュートにより、このRust関数がJavaScriptから直接呼び出せるようになる。
#[wasm_bindgen]
pub fn solve_for_date(month: u32, day: u32) -> Result<JsValue, JsValue> {
    // Rustがパニックした際に、ブラウザのコンソールにエラーを出力する設定
    console_error_panic_hook::set_once();

    // --- 事前計算フェーズ ---
    // 探索を始める前に、全ピースの全配置パターンを計算しておく。
    // これにより、探索中の回転や重複チェックのコストをなくし、大幅に高速化する。
    let initial_pieces = get_initial_pieces();
    let piece_sizes: Vec<usize> = initial_pieces.iter().map(|p| p.iter().map(|r| r.iter().sum::<u8>() as usize).sum()).collect();
    let size_6_piece_index = piece_sizes.iter().position(|&s| s == 6).unwrap();

    let all_piece_placements: Vec<Vec<u64>> = initial_pieces.iter().map(|p_shape| {
        let unique_shapes = get_unique_rotations(p_shape);
        let mut placements = HashSet::new();
        for shape in unique_shapes {
            let (h, w) = (shape.len(), shape[0].len());
            for r in 0..(8 - h) {
                for c in 0..(8 - w) {
                    let mut board = vec![vec![0; 7]; 7];
                    let mut is_valid = true;
                    for (i, row) in shape.iter().enumerate() {
                        for (j, &cell) in row.iter().enumerate() {
                            if r + i >= 7 || c + j >= 7 {
                                if cell == 1 { is_valid = false; }
                                continue;
                            };
                            if cell == 1 { board[r + i][c + j] = 1; }
                        }
                    }
                    if is_valid { placements.insert(board_to_bitmask(&board)); }
                }
            }
        }
        placements.into_iter().collect()
    }).collect();

    // --- 盤面初期化フェーズ ---
    // 1. まず空の盤面(0)を用意
    let mut initial_board = vec![vec![0u8; 7]; 7];

    // 2. 常に固定の穴(1)をマーク
    initial_board[0][6] = 1;
    initial_board[1][6] = 1;
    initial_board[6][3] = 1;
    initial_board[6][4] = 1;
    initial_board[6][5] = 1;
    initial_board[6][6] = 1;

    // 3. 指定された月日の穴(1)をマーク
    let month_r = ((month - 1) / 6) as usize;
    let month_c = ((month - 1) % 6) as usize;
    initial_board[month_r][month_c] = 1;

    let day_r = ((day - 1) / 7 + 2) as usize;
    let day_c = ((day - 1) % 7) as usize;
    initial_board[day_r][day_c] = 1;
    
    // 4. 正しい盤面のビットマスクを生成
    let initial_board_mask = board_to_bitmask(&initial_board);
    
    // --- 探索実行フェーズ ---
    let mut found_raw_solutions = Vec::new();
    find_solutions_recursive(0, initial_board_mask, &mut Vec::new(), false, &all_piece_placements, size_6_piece_index, &mut found_raw_solutions);

    // --- 結果の変換・返却フェーズ ---
    // 探索結果（ビットマスクのリスト）を、JavaScriptが扱いやすい`Solution`構造体のリストに変換する。
    let final_solutions: Vec<Solution> = found_raw_solutions.iter().map(|masks| {
        let mut board = vec![vec![0i8; 7]; 7];
        for (piece_id, &mask) in masks.iter().enumerate() {
            for i in 0..49 {
                if (mask >> i) & 1 == 1 {
                    board[i / 7][i % 7] = (piece_id + 1) as i8;
                }
            }
        }
        // 日付の穴を-1でマーク
        board[month_r][month_c] = -1;
        board[day_r][day_c] = -1;

        Solution { board }
    }).collect();

    // `serde_wasm_bindgen`を使って、Rustの`Vec<Solution>`をJavaScriptの`JsValue`
    // （実質的にはJavaScriptのオブジェクトの配列）にシリアライズして返す。
    // `?`はシリアライズ中にエラーが発生した場合に、そのエラーをJavaScript側に送るための糖衣構文。
    Ok(serde_wasm_bindgen::to_value(&final_solutions)?)
}