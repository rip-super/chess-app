use engine::*;
use std::io::{self, Write};

fn parse_uci(input: &str, gs: &GameState) -> Option<Move> {
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

    let (color, piece) = gs.position.piece_on(from)?;
    if color != gs.position.side_to_move {
        return None;
    }

    let target_piece = gs.position.piece_on(to);

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
            if (color == Color::White && rank_to == 7) || (color == Color::Black && rank_to == 0) {
                flag = if target_piece.is_some() {
                    MoveFlag::PromotionCapture
                } else {
                    MoveFlag::Promotion
                };
            } else if Some(to) == gs.position.en_passant {
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

    Some(Move {
        from,
        to,
        promotion,
        flag,
    })
}

pub fn print_position(gs: &GameState) {
    let pos = &gs.position;
    println!("  +------------------------+");
    for rank in (0..8).rev() {
        print!("{} |", rank + 1);
        for file in 0..8 {
            let sq = rank * 8 + file;
            let ch = if let Some((color, piece)) = pos.piece_on(sq) {
                match (color, piece) {
                    (Color::White, Piece::Pawn) => 'P',
                    (Color::White, Piece::Knight) => 'N',
                    (Color::White, Piece::Bishop) => 'B',
                    (Color::White, Piece::Rook) => 'R',
                    (Color::White, Piece::Queen) => 'Q',
                    (Color::White, Piece::King) => 'K',
                    (Color::Black, Piece::Pawn) => 'p',
                    (Color::Black, Piece::Knight) => 'n',
                    (Color::Black, Piece::Bishop) => 'b',
                    (Color::Black, Piece::Rook) => 'r',
                    (Color::Black, Piece::Queen) => 'q',
                    (Color::Black, Piece::King) => 'k',
                }
            } else {
                '.'
            };
            print!(" {} ", ch);
        }
        println!("|");
    }
    println!("  +------------------------+");
    println!("    a  b  c  d  e  f  g  h");
    println!("Side to move: {:?}", pos.side_to_move);

    println!(
        "Castling: {}{}{}{}",
        if pos.castling_rights & 1 != 0 {
            'K'
        } else {
            '-'
        },
        if pos.castling_rights & 2 != 0 {
            'Q'
        } else {
            '-'
        },
        if pos.castling_rights & 4 != 0 {
            'k'
        } else {
            '-'
        },
        if pos.castling_rights & 8 != 0 {
            'q'
        } else {
            '-'
        }
    );

    if let Some(ep) = pos.en_passant {
        let file = (ep % 8) + b'a';
        let rank = (ep / 8) + b'1';
        println!("En passant: {}{}", file as char, rank as char);
    } else {
        println!("En passant: -");
    }

    println!("Game result: {:?}", gs.result);
}

fn main() {
    let mut gs = GameState::new();

    loop {
        print_position(&gs);

        if gs.result != GameResult::Ongoing {
            match gs.result {
                GameResult::Checkmate(winner) => {
                    println!("Checkmate! {:?} wins.", winner);
                }
                GameResult::Stalemate => {
                    println!("Stalemate!");
                }
                GameResult::DrawFiftyMove => {
                    println!("Draw by fifty-move rule!");
                }
                GameResult::DrawRepetition => {
                    println!("Draw by threefold repetition!");
                }
                GameResult::DrawInsufficientMaterial => {
                    println!("Draw due to insufficient material!");
                }
                GameResult::Ongoing => {}
            }
            break;
        }

        print!("Enter move (uci, undo, quit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "quit" {
            break;
        }

        if input == "undo" {
            gs.undo_move();
            continue;
        }

        let Some(mv) = parse_uci(input, &gs) else {
            println!("Invalid input.");
            continue;
        };

        if let Err(msg) = gs.make_move(mv) {
            println!("{}", msg);
        }
    }
}
