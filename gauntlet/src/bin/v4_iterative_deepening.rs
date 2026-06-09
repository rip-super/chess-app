use engine::*;
use std::time::{Duration, Instant};

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 10_000];
const CHECKMATE_SCORE: i32 = 1_000_000;
const TIMEOUT: i32 = i32::MAX;

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

fn negamax(pos: &mut Position, depth: u32, mut alpha: i32, beta: i32, deadline: Instant) -> i32 {
    if Instant::now() >= deadline {
        return TIMEOUT;
    }

    if depth == 0 {
        return evaluate(pos);
    }

    let moves = pos.get_legal_moves();
    if moves.is_empty() {
        return if pos.is_in_check(pos.side_to_move) {
            -CHECKMATE_SCORE
        } else {
            0
        };
    }

    for mv in moves {
        let undo = pos.make_move(mv);
        let raw = negamax(pos, depth - 1, -beta, -alpha, deadline);
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

pub struct Bot {
    pub depth: u32,
    pub max_time: Option<Duration>,
}

impl Bot {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Bot {
            depth: u32::MAX,
            max_time: Some(Duration::from_millis(500)),
        }
    }

    pub fn with_depth(depth: u32) -> Self {
        Bot {
            depth,
            max_time: Some(Duration::from_millis(500)),
        }
    }

    pub fn with_time(max_time: Duration) -> Self {
        Bot {
            depth: u32::MAX,
            max_time: Some(max_time),
        }
    }

    pub fn best_move(&mut self, gs: &mut GameState) -> Option<Move> {
        let moves = gs.position.get_legal_moves();
        if moves.is_empty() {
            return None;
        }

        let deadline = Instant::now() + self.max_time.unwrap_or(Duration::from_millis(500));

        let mut best_move = moves.first().copied();

        for depth in 1..=self.depth {
            if Instant::now() >= deadline {
                break;
            }

            let mut current_best = None;
            let mut alpha = -CHECKMATE_SCORE - 1;
            let mut completed = true;

            for &mv in &moves {
                let undo = gs.position.make_move(mv);
                let raw = negamax(
                    &mut gs.position,
                    depth - 1,
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
