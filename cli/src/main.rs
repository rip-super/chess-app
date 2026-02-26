use engine::*;

pub fn print_position(pos: &Position) {
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
    println!("   a  b  c  d  e  f  g  h");
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

    println!("Halfmove clock: {}", pos.halfmove_clock);
}

fn main() {
    let mut pos = Position::from_fen("8/4P3/8/8/8/8/8/4kK3 w - - 0 1");
    print_position(&pos);

    let mv = Move {
        from: Square::E7 as u8,
        to: Square::E8 as u8,
        promotion: Some(Piece::Queen),
        flag: MoveFlag::Promotion,
    };

    let undo = pos.make_move(mv);

    print_position(&pos);

    pos.undo_move(mv, undo);
    print_position(&pos);
}
