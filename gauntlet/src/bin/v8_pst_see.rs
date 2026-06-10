use engine::*;
use std::time::{Duration, Instant};

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 10_000];
const CHECKMATE_SCORE: i32 = 1_000_000;
const TIMEOUT: i32 = i32::MAX;
const MAX_DEPTH: usize = 64;
const TT_SIZE: usize = 1 << 20;

const PHASE_WEIGHTS: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;

const PASSED_MG: [i32; 8] = [0, 0, 10, 20, 35, 60, 100, 0];
const PASSED_EG: [i32; 8] = [0, 0, 20, 40, 70, 120, 200, 0];

const DOUBLED_MG: i32 = 15;
const DOUBLED_EG: i32 = 25;
const ISOLATED_MG: i32 = 15;
const ISOLATED_EG: i32 = 20;
const KING_PROX: i32 = 5;

#[rustfmt::skip]
const MG_PST: [[i32; 64]; 6] = [
    [
         0,  0,  0,  0,  0,  0,  0,  0,
         5, 10, 10,-20,-20, 10, 10,  5,
         5, -5,-10,  0,  0,-10, -5,  5,
         0,  0,  0, 20, 20,  0,  0,  0,
         5,  5, 10, 25, 25, 10,  5,  5,
        10, 10, 20, 30, 30, 20, 10, 10,
        50, 50, 50, 50, 50, 50, 50, 50,
         0,  0,  0,  0,  0,  0,  0,  0,
    ],
    [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ],
    [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ],
    [
         0,  0,  0,  5,  5,  0,  0,  0,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
         5, 10, 10, 10, 10, 10, 10,  5,
         0,  0,  0,  0,  0,  0,  0,  0,
    ],
    [
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -10,  5,  5,  5,  5,  5,  0,-10,
          0,  0,  5,  5,  5,  5,  0, -5,
         -5,  0,  5,  5,  5,  5,  0, -5,
        -10,  0,  5,  5,  5,  5,  0,-10,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20,
    ],
    [
         20, 30, 10,  0,  0, 10, 30, 20,
         20, 20,  0,  0,  0,  0, 20, 20,
        -10,-20,-20,-20,-20,-20,-20,-10,
        -20,-30,-30,-40,-40,-30,-30,-20,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
    ],
];

#[rustfmt::skip]
const EG_PST: [[i32; 64]; 6] = [
    [
         0,  0,  0,  0,  0,  0,  0,  0,
         5,  5,  5,  5,  5,  5,  5,  5,
        10, 10, 10, 10, 10, 10, 10, 10,
        20, 20, 20, 20, 20, 20, 20, 20,
        30, 30, 30, 30, 30, 30, 30, 30,
        50, 50, 50, 50, 50, 50, 50, 50,
        80, 80, 80, 80, 80, 80, 80, 80,
         0,  0,  0,  0,  0,  0,  0,  0,
    ],
    [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ],
    [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ],
    [
         0,  0,  0,  5,  5,  0,  0,  0,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
         5, 10, 10, 10, 10, 10, 10,  5,
         0,  0,  0,  0,  0,  0,  0,  0,
    ],
    [
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -10,  5,  5,  5,  5,  5,  0,-10,
          0,  0,  5,  5,  5,  5,  0, -5,
         -5,  0,  5,  5,  5,  5,  0, -5,
        -10,  0,  5,  5,  5,  5,  0,-10,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20,
    ],
    [
        -50,-30,-30,-30,-30,-30,-30,-50,
        -30,-30,  0,  0,  0,  0,-30,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-20,-10,  0,  0,-10,-20,-30,
        -50,-40,-30,-20,-20,-30,-40,-50,
    ],
];

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

fn game_phase(pos: &Position) -> i32 {
    let mut phase = 0i32;
    for (piece, &weight) in PHASE_WEIGHTS.iter().enumerate() {
        let w = pos.bitboards.pieces[Color::White as usize][piece].count_ones() as i32;
        let b = pos.bitboards.pieces[Color::Black as usize][piece].count_ones() as i32;
        phase += (w + b) * weight;
    }
    phase.min(MAX_PHASE)
}

fn evaluate(pos: &Position) -> i32 {
    let phase = game_phase(pos);
    let mut mg = 0i32;
    let mut eg = 0i32;

    for piece in 0..6 {
        let value = PIECE_VALUES[piece];

        let mut wb = pos.bitboards.pieces[Color::White as usize][piece];
        while wb != 0 {
            let sq = wb.trailing_zeros() as usize;
            wb &= wb - 1;
            mg += value + MG_PST[piece][sq];
            eg += value + EG_PST[piece][sq];
        }

        let mut bb = pos.bitboards.pieces[Color::Black as usize][piece];
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            let mirrored = sq ^ 56;
            mg -= value + MG_PST[piece][mirrored];
            eg -= value + EG_PST[piece][mirrored];
        }
    }

    let wp = pos.bitboards.pieces[Color::White as usize][Piece::Pawn as usize];
    let bp = pos.bitboards.pieces[Color::Black as usize][Piece::Pawn as usize];

    let mut tmp = bp | ((bp & NOT_H_FILE) << 1) | ((bp & NOT_A_FILE) >> 1);

    tmp |= tmp >> 8;
    tmp |= tmp >> 16;
    tmp |= tmp >> 32;

    let w_passed = wp & !tmp;

    let mut tmp = wp | ((wp & NOT_H_FILE) << 1) | ((wp & NOT_A_FILE) >> 1);

    tmp |= tmp << 8;
    tmp |= tmp << 16;
    tmp |= tmp << 32;

    let b_passed = bp & !tmp;

    let mut wpp = w_passed;
    while wpp != 0 {
        let sq = wpp.trailing_zeros() as usize;
        wpp &= wpp - 1;
        let rank = sq / 8;
        mg += PASSED_MG[rank];
        eg += PASSED_EG[rank];
    }

    let mut bpp = b_passed;
    while bpp != 0 {
        let sq = bpp.trailing_zeros() as usize;
        bpp &= bpp - 1;
        let rank = (sq ^ 56) / 8;
        mg -= PASSED_MG[rank];
        eg -= PASSED_EG[rank];
    }

    let mut w_doubled = 0i32;
    let mut b_doubled = 0i32;

    for file in 0..8 {
        let mask = 0x0101010101010101u64 << file;

        let wn = ((wp & mask) >> file).count_ones() as i32;
        if wn > 1 {
            w_doubled += wn - 1;
        }

        let bn = ((bp & mask) >> file).count_ones() as i32;
        if bn > 1 {
            b_doubled += bn - 1;
        }
    }

    let mut w_isolated = 0i32;
    let mut b_isolated = 0i32;

    for file in 0..8usize {
        let file_mask = 0x0101010101010101u64 << file;

        let left = if file > 0 {
            0x0101010101010101u64 << (file - 1)
        } else {
            0
        };
        let right = if file < 7 {
            0x0101010101010101u64 << (file + 1)
        } else {
            0
        };

        if wp & file_mask != 0 && (wp & (left | right)) == 0 {
            w_isolated += (wp & file_mask).count_ones() as i32;
        }

        if bp & file_mask != 0 && (bp & (left | right)) == 0 {
            b_isolated += (bp & file_mask).count_ones() as i32;
        }
    }

    mg += (b_doubled - w_doubled) * DOUBLED_MG;
    eg += (b_doubled - w_doubled) * DOUBLED_EG;
    mg += (b_isolated - w_isolated) * ISOLATED_MG;
    eg += (b_isolated - w_isolated) * ISOLATED_EG;

    let wk =
        pos.bitboards.pieces[Color::White as usize][Piece::King as usize].trailing_zeros() as usize;
    let bk =
        pos.bitboards.pieces[Color::Black as usize][Piece::King as usize].trailing_zeros() as usize;

    let chebyshev = |a: usize, b: usize| -> i32 {
        let dr = ((a / 8) as i32 - (b / 8) as i32).abs();
        let df = ((a % 8) as i32 - (b % 8) as i32).abs();
        dr.max(df)
    };

    let mut wpp = w_passed;
    while wpp != 0 {
        let sq = wpp.trailing_zeros() as usize;
        wpp &= wpp - 1;
        let advancement = (sq / 8) as i32;
        eg += (chebyshev(bk, sq) - chebyshev(wk, sq)) * KING_PROX * advancement;
    }

    let mut bpp = b_passed;
    while bpp != 0 {
        let sq = bpp.trailing_zeros() as usize;
        bpp &= bpp - 1;
        let advancement = (7 - sq / 8) as i32;
        eg -= (chebyshev(wk, sq) - chebyshev(bk, sq)) * KING_PROX * advancement;
    }

    let score = (mg * phase + eg * (MAX_PHASE - phase)) / MAX_PHASE;

    if pos.side_to_move == Color::White {
        score
    } else {
        -score
    }
}

fn see(pos: &Position, mv: Move) -> i32 {
    fn find_lva(pos: &Position, sq: usize, side: Color, occ: u64) -> Option<(usize, i32)> {
        let idx = side as usize;

        let pawns = PAWN_ATTACKS[side.opposite() as usize][sq]
            & pos.bitboards.pieces[idx][Piece::Pawn as usize]
            & occ;
        if pawns != 0 {
            return Some((pawns.trailing_zeros() as usize, PIECE_VALUES[0]));
        }

        let knights = KNIGHT_ATTACKS[sq] & pos.bitboards.pieces[idx][Piece::Knight as usize] & occ;
        if knights != 0 {
            return Some((knights.trailing_zeros() as usize, PIECE_VALUES[1]));
        }

        let diag = get_bishop_attacks(sq, occ);
        let bishops = diag & pos.bitboards.pieces[idx][Piece::Bishop as usize];
        if bishops != 0 {
            return Some((bishops.trailing_zeros() as usize, PIECE_VALUES[2]));
        }

        let straight = get_rook_attacks(sq, occ);
        let rooks = straight & pos.bitboards.pieces[idx][Piece::Rook as usize];
        if rooks != 0 {
            return Some((rooks.trailing_zeros() as usize, PIECE_VALUES[3]));
        }

        let queens = (diag | straight) & pos.bitboards.pieces[idx][Piece::Queen as usize];
        if queens != 0 {
            return Some((queens.trailing_zeros() as usize, PIECE_VALUES[4]));
        }

        let kings = KING_ATTACKS[sq] & pos.bitboards.pieces[idx][Piece::King as usize] & occ;
        if kings != 0 {
            return Some((kings.trailing_zeros() as usize, PIECE_VALUES[5]));
        }
        None
    }

    if mv.flag == MoveFlag::EnPassant {
        return 0;
    }

    let to = mv.to as usize;
    let from = mv.from as usize;

    let Some((_, cap_p)) = pos.bitboards.mailbox[to] else {
        return 0;
    };

    let Some((_, mov_p)) = pos.bitboards.mailbox[from] else {
        return 0;
    };

    let mut targets = [0i32; 32];
    targets[0] = PIECE_VALUES[cap_p as usize];
    let mut n = 0;

    let mut piece_on_sq = mv
        .promotion
        .map_or(PIECE_VALUES[mov_p as usize], |p| PIECE_VALUES[p as usize]);

    let mut occ = {
        let mut o = 0u64;
        for c in 0..2 {
            for p in 0..6 {
                o |= pos.bitboards.pieces[c][p];
            }
        }
        o ^ (1u64 << from)
    };

    let mut side = pos.side_to_move.opposite();

    while n < 31 {
        let Some((lva_sq, lva_val)) = find_lva(pos, to, side, occ) else {
            break;
        };
        n += 1;
        targets[n] = piece_on_sq;
        piece_on_sq = lva_val;
        occ ^= 1u64 << lva_sq;
        side = side.opposite();
    }

    let mut gain = targets[n];
    for i in (0..n).rev() {
        gain = targets[i] - gain.max(0);
    }

    gain
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
                let see_val = see(pos, mv);
                let promo_bonus = mv.promotion.map_or(0, |p| PIECE_VALUES[p as usize]);
                if see_val >= 0 {
                    10_000 + see_val + promo_bonus
                } else {
                    see_val + promo_bonus
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
            if !in_check && see(pos, mv) < 0 {
                continue;
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
