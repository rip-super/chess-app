use engine::*;
use std::time::{Duration, Instant};

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 10_000];
const CHECKMATE_SCORE: i32 = 1_000_000;
const TIMEOUT: i32 = i32::MAX;
const MAX_DEPTH: usize = 64;
const TT_SIZE: usize = 1 << 20;

#[derive(Clone, Copy)]
enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
struct TTEntry {
    hash: u64,
    depth: u32,
    score: i32,
    flag: TTFlag,
    best_move: Option<Move>,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            hash: 0,
            depth: 0,
            score: 0,
            flag: TTFlag::Exact,
            best_move: None,
        }
    }
}

fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score > CHECKMATE_SCORE - MAX_DEPTH as i32 {
        score + ply as i32
    } else if score < -CHECKMATE_SCORE + MAX_DEPTH as i32 {
        score - ply as i32
    } else {
        score
    }
}

fn score_from_tt(score: i32, ply: usize) -> i32 {
    if score > CHECKMATE_SCORE - MAX_DEPTH as i32 {
        score - ply as i32
    } else if score < -CHECKMATE_SCORE + MAX_DEPTH as i32 {
        score + ply as i32
    } else {
        score
    }
}

fn evaluate(pos: &Position) -> i32 {
    let mut score = 0i32;
    for (piece, &value) in PIECE_VALUES.iter().enumerate() {
        let white = pos.bitboards.pieces[Color::White as usize][piece].count_ones() as i32;
        let black = pos.bitboards.pieces[Color::Black as usize][piece].count_ones() as i32;
        score += (white - black) * value;
    }
    if pos.side_to_move == Color::White {
        score
    } else {
        -score
    }
}

pub struct Bot {
    pub depth: u32,
    pub max_time: Option<Duration>,
    killers: [[Option<Move>; 2]; MAX_DEPTH],
    history: [[i32; 64]; 6],
    rep_history: [u64; MAX_DEPTH],
    tt: Box<[TTEntry]>,
}

impl Bot {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Bot {
            depth: u32::MAX,
            max_time: Some(Duration::from_millis(500)),
            killers: [[None; 2]; MAX_DEPTH],
            history: [[0; 64]; 6],
            rep_history: [0; MAX_DEPTH],
            tt: vec![TTEntry::default(); TT_SIZE].into_boxed_slice(),
        }
    }

    pub fn with_depth(depth: u32) -> Self {
        Bot {
            depth,
            max_time: Some(Duration::from_millis(500)),
            killers: [[None; 2]; MAX_DEPTH],
            history: [[0; 64]; 6],
            rep_history: [0; MAX_DEPTH],
            tt: vec![TTEntry::default(); TT_SIZE].into_boxed_slice(),
        }
    }

    pub fn with_time(max_time: Duration) -> Self {
        Bot {
            depth: u32::MAX,
            max_time: Some(max_time),
            killers: [[None; 2]; MAX_DEPTH],
            history: [[0; 64]; 6],
            rep_history: [0; MAX_DEPTH],
            tt: vec![TTEntry::default(); TT_SIZE].into_boxed_slice(),
        }
    }

    fn score_move(&self, pos: &Position, mv: Move, ply: usize) -> i32 {
        match mv.flag {
            MoveFlag::Capture | MoveFlag::PromotionCapture => {
                let victim = pos.bitboards.mailbox[mv.to as usize]
                    .map_or(0, |(_, p)| PIECE_VALUES[p as usize]);
                let attacker = pos.bitboards.mailbox[mv.from as usize]
                    .map_or(0, |(_, p)| PIECE_VALUES[p as usize]);
                let promo_bonus = mv.promotion.map_or(0, |p| PIECE_VALUES[p as usize]);
                let mvv_lva = victim - attacker;

                if mvv_lva >= 0 {
                    10_000 + mvv_lva + promo_bonus
                } else {
                    mvv_lva + promo_bonus
                }
            }

            MoveFlag::EnPassant => 10_000,

            MoveFlag::Promotion => 9_500 + mv.promotion.map_or(0, |p| PIECE_VALUES[p as usize]),

            _ => {
                let k = self.killers[ply];
                if k[0] == Some(mv) {
                    return 9_000;
                } else if k[1] == Some(mv) {
                    return 8_000;
                }

                let piece = pos.bitboards.mailbox[mv.from as usize].map_or(0, |(_, p)| p as usize);
                self.history[piece][mv.to as usize].min(7_000)
            }
        }
    }

    fn negamax(
        &mut self,
        pos: &mut Position,
        depth: u32,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        deadline: Instant,
    ) -> i32 {
        if Instant::now() >= deadline {
            return TIMEOUT;
        }

        let hash = pos.zobrist_hash();

        self.rep_history[ply] = hash;
        for i in (0..ply).step_by(2) {
            if self.rep_history[i] == hash {
                return 0;
            }
        }

        let tt_idx = (hash as usize) & (TT_SIZE - 1);
        let tt_entry = self.tt[tt_idx];
        let mut tt_move = None;

        if tt_entry.hash == hash {
            tt_move = tt_entry.best_move;

            if tt_entry.depth >= depth {
                let score = tt_entry.score;
                match tt_entry.flag {
                    TTFlag::Exact => return score_from_tt(score, ply),
                    TTFlag::LowerBound => {
                        let adjusted = score_from_tt(score, ply);
                        if adjusted >= beta {
                            return beta;
                        }
                        alpha = alpha.max(adjusted);
                    }
                    TTFlag::UpperBound => {
                        let adjusted = score_from_tt(score, ply);
                        if adjusted <= alpha {
                            return alpha;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            return evaluate(pos);
        }

        let mut moves = pos.get_legal_moves();
        if moves.is_empty() {
            return if pos.is_in_check(pos.side_to_move) {
                -CHECKMATE_SCORE
            } else {
                0
            };
        }

        moves.sort_unstable_by_key(|&mv| -self.score_move(pos, mv, ply));

        if let Some(tm) = tt_move
            && let Some(idx) = moves.iter().position(|&m| m == tm)
        {
            moves.swap(0, idx);
        }

        let original_alpha = alpha;
        let mut best_move_found = None;

        for mv in moves {
            let undo = pos.make_move(mv);
            let raw = self.negamax(pos, depth - 1, ply + 1, -beta, -alpha, deadline);
            pos.undo_move(mv, undo);

            if raw == TIMEOUT {
                return TIMEOUT;
            }

            let score = -raw;

            if score >= beta {
                if matches!(
                    mv.flag,
                    MoveFlag::Quiet
                        | MoveFlag::DoublePawnPush
                        | MoveFlag::KingCastle
                        | MoveFlag::QueenCastle
                ) {
                    let k = &mut self.killers[ply];
                    if k[0] != Some(mv) {
                        k[1] = k[0];
                        k[0] = Some(mv);
                    }

                    let piece =
                        pos.bitboards.mailbox[mv.from as usize].map_or(0, |(_, p)| p as usize);
                    self.history[piece][mv.to as usize] += (depth * depth) as i32;
                }

                self.tt[tt_idx] = TTEntry {
                    hash,
                    depth,
                    score: score_to_tt(beta, ply),
                    flag: TTFlag::LowerBound,
                    best_move: Some(mv),
                };

                return beta;
            }

            if score > alpha {
                alpha = score;
                best_move_found = Some(mv);
            }
        }

        self.tt[tt_idx] = TTEntry {
            hash,
            depth,
            score: score_to_tt(alpha, ply),
            flag: if alpha > original_alpha {
                TTFlag::Exact
            } else {
                TTFlag::UpperBound
            },
            best_move: best_move_found,
        };

        alpha
    }

    pub fn best_move(&mut self, gs: &mut GameState) -> Option<Move> {
        self.history = [[0; 64]; 6];
        self.killers = [[None; 2]; MAX_DEPTH];

        let mut moves = gs.position.get_legal_moves();
        if moves.is_empty() {
            return None;
        }

        moves.sort_unstable_by_key(|&mv| -self.score_move(&gs.position, mv, 0));

        let deadline = Instant::now() + self.max_time.unwrap_or(Duration::from_millis(500));
        let mut best_move = moves.first().copied();

        for depth in 1..=self.depth {
            if Instant::now() >= deadline {
                break;
            }

            if let Some(bm) = best_move
                && let Some(idx) = moves.iter().position(|&m| m == bm)
            {
                moves.swap(0, idx);
            }

            self.killers = [[None; 2]; MAX_DEPTH];

            let mut current_best = None;
            let mut alpha = -CHECKMATE_SCORE - 1;
            let mut completed = true;

            for &mv in &moves {
                let undo = gs.position.make_move(mv);
                let raw = self.negamax(
                    &mut gs.position,
                    depth - 1,
                    1,
                    -CHECKMATE_SCORE - 1,
                    -alpha,
                    deadline,
                );
                gs.position.undo_move(mv, undo);

                if raw == TIMEOUT {
                    completed = false;
                    break;
                }

                let score = -raw;
                if score > alpha {
                    alpha = score;
                    current_best = Some(mv);
                }
            }

            if completed && let Some(mv) = current_best {
                best_move = Some(mv);
            }
        }

        best_move
    }
}

gauntlet::bot_main!();
