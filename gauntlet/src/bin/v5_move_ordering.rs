use engine::*;
use std::time::{Duration, Instant};

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 10_000];
const CHECKMATE_SCORE: i32 = 1_000_000;
const TIMEOUT: i32 = i32::MAX;
const MAX_DEPTH: usize = 64;

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
}

impl Bot {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Bot {
            depth: u32::MAX,
            max_time: Some(Duration::from_millis(500)),
            killers: [[None; 2]; MAX_DEPTH],
            history: [[0; 64]; 6],
        }
    }

    pub fn with_depth(depth: u32) -> Self {
        Bot {
            depth,
            max_time: Some(Duration::from_millis(500)),
            killers: [[None; 2]; MAX_DEPTH],
            history: [[0; 64]; 6],
        }
    }

    pub fn with_time(max_time: Duration) -> Self {
        Bot {
            depth: u32::MAX,
            max_time: Some(max_time),
            killers: [[None; 2]; MAX_DEPTH],
            history: [[0; 64]; 6],
        }
    }

    fn score_move(&self, pos: &Position, mv: Move, ply: usize) -> i32 {
        match mv.flag {
            MoveFlag::Capture | MoveFlag::PromotionCapture => {
                let victim = pos.bitboards.mailbox[mv.to as usize]
                    .map_or(0, |(_, p)| PIECE_VALUES[p as usize]);
                let attacker = pos.bitboards.mailbox[mv.from as usize]
                    .map_or(0, |(_, p)| PIECE_VALUES[p as usize]);
                10_000 + victim - attacker
            }
            MoveFlag::EnPassant => 10_000,
            MoveFlag::Promotion => mv.promotion.map_or(0, |p| PIECE_VALUES[p as usize]),
            _ => {
                let k = self.killers[ply];
                if k[0] == Some(mv) {
                    9_000
                } else if k[1] == Some(mv) {
                    8_000
                } else {
                    let piece =
                        pos.bitboards.mailbox[mv.from as usize].map_or(0, |(_, p)| p as usize);
                    self.history[piece][mv.to as usize].min(7_000)
                }
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

                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

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
