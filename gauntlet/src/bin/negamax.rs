use engine::*;

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 10_000];

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

const CHECKMATE_SCORE: i32 = 1_000_000;

fn negamax(pos: &mut Position, depth: u32) -> i32 {
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

    let mut best = -CHECKMATE_SCORE - 1;

    for mv in moves {
        let undo = pos.make_move(mv);
        let score = -negamax(pos, depth - 1);
        pos.undo_move(mv, undo);

        if score > best {
            best = score;
        }
    }

    best
}

pub struct Bot {
    pub depth: u32,
}

impl Bot {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Bot { depth: 4 }
    }

    pub fn with_depth(depth: u32) -> Self {
        Bot { depth }
    }

    pub fn best_move(&mut self, gs: &mut GameState) -> Option<Move> {
        let moves = gs.position.get_legal_moves();
        if moves.is_empty() {
            return None;
        }

        let mut best_move = None;
        let mut best_score = -CHECKMATE_SCORE - 1;

        for mv in moves {
            let undo = gs.position.make_move(mv);
            let score = -negamax(&mut gs.position, self.depth - 1);
            gs.position.undo_move(mv, undo);

            if score > best_score {
                best_score = score;
                best_move = Some(mv)
            }
        }

        best_move
    }
}

gauntlet::bot_main!();
