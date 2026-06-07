use bot::*;
use engine::*;
use std::io::{self, Write};

enum Controller {
    Human,
    Bot(Bot),
}

impl Controller {
    fn is_bot(&self) -> bool {
        matches!(self, Controller::Bot(_))
    }
}

struct GameConfig {
    white: Controller,
    black: Controller,
}

impl GameConfig {
    fn for_color(&mut self, color: Color) -> &mut Controller {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    fn is_bot(&self, color: Color) -> bool {
        match color {
            Color::White => self.white.is_bot(),
            Color::Black => self.black.is_bot(),
        }
    }
}

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}

fn setup_game() -> GameConfig {
    println!("Select game mode:");
    println!("  1  Player vs Player");
    println!("  2  Player vs Bot");
    println!("  3  Bot vs Bot");

    let mode = loop {
        match read_line("> ").as_str() {
            "1" => break 1u8,
            "2" => break 2,
            "3" => break 3,
            _ => println!("Please enter 1, 2, or 3."),
        }
    };

    match mode {
        1 => GameConfig {
            white: Controller::Human,
            black: Controller::Human,
        },
        2 => {
            println!("\nPlay as:");
            println!("  1  White");
            println!("  2  Black");

            let human_color = loop {
                match read_line("> ").as_str() {
                    "1" => break Color::White,
                    "2" => break Color::Black,
                    _ => println!("Please enter 1 or 2."),
                }
            };

            match human_color {
                Color::White => GameConfig {
                    white: Controller::Human,
                    black: Controller::Bot(Bot::new()),
                },
                Color::Black => GameConfig {
                    white: Controller::Bot(Bot::new()),
                    black: Controller::Human,
                },
            }
        }
        _ => GameConfig {
            white: Controller::Bot(Bot::new()),
            black: Controller::Bot(Bot::new()),
        },
    }
}

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

fn print_position(gs: &GameState) {
    let pos = &gs.position;

    let white_in_check = pos.is_in_check(Color::White);
    let black_in_check = pos.is_in_check(Color::Black);

    let white_king_sq = {
        let bb = pos.bitboards.pieces[Color::White as usize][Piece::King as usize];
        if bb != 0 {
            Some(bb.trailing_zeros() as u8)
        } else {
            None
        }
    };

    let black_king_sq = {
        let bb = pos.bitboards.pieces[Color::Black as usize][Piece::King as usize];
        if bb != 0 {
            Some(bb.trailing_zeros() as u8)
        } else {
            None
        }
    };

    const ANSI_RESET: &str = "\x1b[0m";
    const ANSI_CHECK_BG: &str = "\x1b[41m";
    const ANSI_CHECK_FG: &str = "\x1b[97m";

    println!("  +------------------------+");
    for rank in (0..8).rev() {
        print!("{} |", rank + 1);
        for file in 0..8 {
            let sq = (rank * 8 + file) as u8;

            if let Some((color, piece)) = pos.piece_on(sq) {
                let ch = match (color, piece) {
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
                };

                let highlight = match color {
                    Color::White => white_in_check && white_king_sq == Some(sq),
                    Color::Black => black_in_check && black_king_sq == Some(sq),
                };

                if highlight {
                    print!(" {}{}{}{} ", ANSI_CHECK_BG, ANSI_CHECK_FG, ch, ANSI_RESET);
                } else {
                    print!(" {} ", ch);
                }
            } else {
                print!(" . ");
            }
        }
        println!("|");
    }
    println!("  +------------------------+");
    println!("    a  b  c  d  e  f  g  h");
    println!();
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
    let mut config = setup_game();
    let mut gs = GameState::new();
    println!();

    loop {
        print_position(&gs);

        if gs.result != GameResult::Ongoing {
            match gs.result {
                GameResult::Checkmate(winner) => println!("Checkmate! {:?} wins.", winner),
                GameResult::Stalemate => println!("Stalemate!"),
                GameResult::DrawFiftyMove => println!("Draw by fifty-move rule!"),
                GameResult::DrawRepetition => println!("Draw by threefold repetition!"),
                GameResult::DrawInsufficientMaterial => {
                    println!("Draw due to insufficient material!")
                }
                GameResult::Ongoing => {}
            }
            break;
        }

        let side = gs.position.side_to_move;

        if let Controller::Bot(bot) = config.for_color(side) {
            if let Some(mv) = bot.best_move(&mut gs) {
                gs.make_move(mv).unwrap();
            }
            continue;
        }

        print!("Enter move (uci, undo, quit): ");
        io::stdout().flush().unwrap();

        let input = read_line("Enter move (uci, undo, quit): ");
        match input.as_str() {
            "quit" => break,
            "undo" => {
                let just_played = side.opposite();
                gs.undo_move();
                if config.is_bot(just_played) {
                    gs.undo_move();
                }
            }
            mv_str => {
                let Some(mv) = parse_uci(mv_str, &gs) else {
                    println!("Invalid input.");
                    continue;
                };
                if let Err(msg) = gs.make_move(mv) {
                    println!("{}", msg);
                }
            }
        }
    }
}
