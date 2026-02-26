use engine::*;

fn print_bitboard(board: u64) {
    println!();

    for rank in 0..8 {
        for file in 0..8 {
            let square = rank * 8 + file;

            if file == 0 {
                print!(" {}  ", 8 - rank);
            }

            print!(" {}", get_bit(board, square));
        }

        println!();
    }

    print!("\n     a b c d e f g h\n\n");

    println!("     Bitboard: {board}");

    println!();
}

fn main() {
    print_bitboard(AttackTables::mask_king_attacks(Square::E4));
}
