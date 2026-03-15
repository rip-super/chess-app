use engine::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ChessMove(Move);

#[wasm_bindgen]
impl ChessMove {
    pub fn from_sq(&self) -> u8 {
        self.0.from
    }

    pub fn to_sq(&self) -> u8 {
        self.0.to
    }

    pub fn is_capture(&self) -> bool {
        matches!(
            self.0.flag,
            MoveFlag::Capture | MoveFlag::PromotionCapture | MoveFlag::EnPassant
        )
    }

    pub fn is_en_passant(&self) -> bool {
        self.0.flag == MoveFlag::EnPassant
    }

    pub fn is_double_pawn_push(&self) -> bool {
        self.0.flag == MoveFlag::DoublePawnPush
    }

    pub fn is_kingside_castle(&self) -> bool {
        self.0.flag == MoveFlag::KingCastle
    }

    pub fn is_queenside_castle(&self) -> bool {
        self.0.flag == MoveFlag::QueenCastle
    }

    pub fn is_castle(&self) -> bool {
        matches!(self.0.flag, MoveFlag::KingCastle | MoveFlag::QueenCastle)
    }

    pub fn is_promotion(&self) -> bool {
        self.0.promotion.is_some()
    }

    pub fn promotion_piece(&self) -> Option<String> {
        self.0.promotion.map(|p| match p {
            Piece::Queen => "q".to_string(),
            Piece::Rook => "r".to_string(),
            Piece::Bishop => "b".to_string(),
            Piece::Knight => "n".to_string(),
            _ => unreachable!(),
        })
    }

    pub fn to_uci(&self) -> String {
        let from_file = (self.0.from % 8 + b'a') as char;
        let from_rank = (self.0.from / 8 + b'1') as char;
        let to_file = (self.0.to % 8 + b'a') as char;
        let to_rank = (self.0.to / 8 + b'1') as char;
        let mut s = format!("{}{}{}{}", from_file, from_rank, to_file, to_rank);
        if let Some(p) = self.0.promotion {
            s.push(match p {
                Piece::Queen => 'q',
                Piece::Rook => 'r',
                Piece::Bishop => 'b',
                Piece::Knight => 'n',
                _ => unreachable!(),
            });
        }
        s
    }
}

#[wasm_bindgen]
pub struct ChessEngine {
    gs: GameState,
}

#[wasm_bindgen]
impl ChessEngine {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            gs: GameState::new(),
        }
    }

    pub fn from_fen(fen: &str) -> ChessEngine {
        ChessEngine {
            gs: GameState::from_fen(fen),
        }
    }

    pub fn parse_uci(&self, input: &str) -> Option<ChessMove> {
        if input.len() < 4 {
            return None;
        }

        let bytes = input.as_bytes();
        let file_from = bytes[0].wrapping_sub(b'a');
        let rank_from = bytes[1].wrapping_sub(b'1');
        let file_to = bytes[2].wrapping_sub(b'a');
        let rank_to = bytes[3].wrapping_sub(b'1');

        if file_from > 7 || file_to > 7 || rank_from > 7 || rank_to > 7 {
            return None;
        }

        let from = rank_from * 8 + file_from;
        let to = rank_to * 8 + file_to;

        let (color, piece) = self.gs.position.piece_on(from)?;
        if color != self.gs.position.side_to_move {
            return None;
        }

        let target_piece = self.gs.position.piece_on(to);

        let promotion = if input.len() == 5 {
            match bytes[4] as char {
                'q' => Some(Piece::Queen),
                'r' => Some(Piece::Rook),
                'b' => Some(Piece::Bishop),
                'n' => Some(Piece::Knight),
                _ => return None,
            }
        } else {
            None
        };

        let mut flag = MoveFlag::Quiet;
        match piece {
            Piece::Pawn => {
                let rank_diff = if color == Color::White {
                    rank_to as i8 - rank_from as i8
                } else {
                    rank_from as i8 - rank_to as i8
                };
                if (color == Color::White && rank_to == 7)
                    || (color == Color::Black && rank_to == 0)
                {
                    flag = if target_piece.is_some() {
                        MoveFlag::PromotionCapture
                    } else {
                        MoveFlag::Promotion
                    };
                } else if Some(to) == self.gs.position.en_passant {
                    flag = MoveFlag::EnPassant;
                } else if rank_diff == 2 {
                    flag = MoveFlag::DoublePawnPush;
                } else if target_piece.is_some() {
                    flag = MoveFlag::Capture;
                }
            }
            Piece::King => {
                if (from as i8 - to as i8).abs() == 2 {
                    flag = if file_to == 6 {
                        MoveFlag::KingCastle
                    } else {
                        MoveFlag::QueenCastle
                    };
                } else if target_piece.is_some() {
                    flag = MoveFlag::Capture;
                }
            }
            _ => {
                if target_piece.is_some() {
                    flag = MoveFlag::Capture;
                }
            }
        }

        Some(ChessMove(Move {
            from,
            to,
            promotion,
            flag,
        }))
    }

    pub fn is_in_check(&self) -> bool {
        self.gs.position.is_in_check(self.gs.position.side_to_move)
    }

    pub fn game_result(&self) -> String {
        match self.gs.result {
            GameResult::Ongoing => "ongoing".to_string(),
            GameResult::Checkmate(Color::White) => "checkmate_white".to_string(),
            GameResult::Checkmate(Color::Black) => "checkmate_black".to_string(),
            GameResult::Stalemate => "stalemate".to_string(),
            GameResult::DrawRepetition => "draw_repetition".to_string(),
            GameResult::DrawFiftyMove => "draw_fifty_move".to_string(),
            GameResult::DrawInsufficientMaterial => "draw_insufficient".to_string(),
        }
    }

    pub fn piece_on(&self, square: u8) -> Option<String> {
        self.gs.position.piece_on(square).map(|(color, piece)| {
            let c = match color {
                Color::White => 'w',
                Color::Black => 'b',
            };
            let p = match piece {
                Piece::Pawn => 'P',
                Piece::Knight => 'N',
                Piece::Bishop => 'B',
                Piece::Rook => 'R',
                Piece::Queen => 'Q',
                Piece::King => 'K',
            };
            format!("{}{}", c, p)
        })
    }

    pub fn side_to_move(&self) -> String {
        match self.gs.position.side_to_move {
            Color::White => "w".to_string(),
            Color::Black => "b".to_string(),
        }
    }

    pub fn king_square(&self, color: &str) -> Option<u8> {
        let target = match color {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None,
        };
        for sq in 0..64 {
            if let Some((c, Piece::King)) = self.gs.position.piece_on(sq)
                && c == target
            {
                return Some(sq);
            }
        }
        None
    }

    #[cfg(feature = "server")]
    pub fn make_move(&mut self, mv: &ChessMove) -> Result<(), JsValue> {
        self.gs.make_move(mv.0).map_err(JsValue::from_str)
    }

    #[cfg(feature = "server")]
    pub fn undo_move(&mut self) {
        self.gs.undo_move();
    }

    pub fn legal_moves(&mut self) -> Box<[JsValue]> {
        self.gs
            .position
            .get_legal_moves()
            .into_iter()
            .map(|mv| JsValue::from(ChessMove(mv)))
            .collect()
    }
}
