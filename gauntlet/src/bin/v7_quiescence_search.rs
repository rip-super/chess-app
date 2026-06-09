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
                let hist = self.history[piece][mv.to as usize].min(7_000);

                let penalty = if pos.is_attacked_by_pawn(mv.to, pos.side_to_move.opposite()) {
                    PIECE_VALUES[piece]
                } else {
                    0
                };

                hist - penalty
            }
        }
    }

    fn quiescence(
        &mut self,
        pos: &mut Position,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        deadline: Instant,
    ) -> i32 {
        if Instant::now() >= deadline {
            return TIMEOUT;
        }

        let in_check = pos.is_in_check(pos.side_to_move);

        if !in_check {
            let stand_pat = evaluate(pos);

            if stand_pat >= beta {
                return beta;
            }

            if stand_pat > alpha {
                alpha = stand_pat;
            }
        }

        let mut moves = pos.get_legal_moves();

        if moves.is_empty() {
            return if in_check { -CHECKMATE_SCORE } else { alpha };
        }

        if !in_check {
            moves.retain(|mv| {
                matches!(
                    mv.flag,
                    MoveFlag::Capture
                        | MoveFlag::PromotionCapture
                        | MoveFlag::EnPassant
                        | MoveFlag::Promotion
                )
            });
        }

        moves.sort_unstable_by_key(|&mv| -self.score_move(pos, mv, ply));

        for mv in moves {
            if !in_check {
                let victim_value = pos.bitboards.mailbox[mv.to as usize]
                    .map_or(0, |(_, p)| PIECE_VALUES[p as usize]);
                let promo_value = mv.promotion.map_or(0, |p| PIECE_VALUES[p as usize]);
                const DELTA_MARGIN: i32 = 200;
                if evaluate(pos) + victim_value + promo_value + DELTA_MARGIN < alpha {
                    continue;
                }
            }

            let undo = pos.make_move(mv);
            let raw = self.quiescence(pos, ply + 1, -beta, -alpha, deadline);
            pos.undo_move(mv, undo);

            if raw == TIMEOUT {
                return TIMEOUT;
            }

            let score = -raw;

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
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
                let eval = evaluate(pos);
                return if eval > 0 {
                    -50
                } else if eval < 0 {
                    50
                } else {
                    0
                };
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
                    TTFlag::Exact => return score,
                    TTFlag::LowerBound => {
                        if score >= beta {
                            return score;
                        }
                        alpha = alpha.max(score);
                    }
                    TTFlag::UpperBound => {
                        if score <= alpha {
                            return score;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            return self.quiescence(pos, ply, alpha, beta, deadline);
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

        for (i, mv) in moves.iter().enumerate() {
            let mv = *mv;
            let undo = pos.make_move(mv);

            let raw = if i == 0 {
                self.negamax(pos, depth - 1, ply + 1, -beta, -alpha, deadline)
            } else {
                let zw = self.negamax(pos, depth - 1, ply + 1, -alpha - 1, -alpha, deadline);

                if zw == TIMEOUT {
                    pos.undo_move(mv, undo);
                    return TIMEOUT;
                }

                if -zw > alpha && -zw < beta {
                    self.negamax(pos, depth - 1, ply + 1, -beta, -alpha, deadline)
                } else {
                    zw
                }
            };

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
                    score: beta,
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
            score: alpha,
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
        let mut prev_score = 0i32;

        'ids: for depth in 1..=self.depth {
            if Instant::now() >= deadline {
                break;
            }

            if let Some(bm) = best_move
                && let Some(idx) = moves.iter().position(|&m| m == bm)
            {
                moves.swap(0, idx);
            }

            self.killers = [[None; 2]; MAX_DEPTH];

            const INIT_DELTA: i32 = 50;

            let (mut alpha, mut beta) = if depth < 3 {
                (-CHECKMATE_SCORE - 1, CHECKMATE_SCORE + 1)
            } else {
                (prev_score - INIT_DELTA, prev_score + INIT_DELTA)
            };
            let mut delta = INIT_DELTA;

            let (completed_move, completed_score) = 'asp: loop {
                let mut best_this_iter = None;
                let mut score_this_iter = alpha;

                for &mv in &moves {
                    let undo = gs.position.make_move(mv);
                    let raw = self.negamax(
                        &mut gs.position,
                        depth - 1,
                        1,
                        -beta,
                        -score_this_iter,
                        deadline,
                    );
                    gs.position.undo_move(mv, undo);

                    if raw == TIMEOUT {
                        break 'ids;
                    }

                    let score = -raw;

                    if score >= beta {
                        if let Some(idx) = moves.iter().position(|&m| m == mv) {
                            moves.swap(0, idx);
                        }
                        beta = (beta + delta).min(CHECKMATE_SCORE + 1);
                        delta *= 2;
                        continue 'asp;
                    }

                    if score > score_this_iter {
                        score_this_iter = score;
                        best_this_iter = Some(mv);
                    }
                }

                if score_this_iter <= alpha {
                    alpha = (alpha - delta).max(-CHECKMATE_SCORE - 1);
                    delta *= 2;
                    continue 'asp;
                }

                break 'asp (best_this_iter, score_this_iter);
            };

            if let Some(mv) = completed_move {
                best_move = Some(mv);
                prev_score = completed_score;
            }
        }

        best_move
    }
}

gauntlet::bot_main!();
