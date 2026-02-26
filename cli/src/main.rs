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
    let mut occupancy = 0;
    set_bit(&mut occupancy, Square::E4);
    set_bit(&mut occupancy, Square::F7);
    let attacks = AttackTables::new();
    print_bitboard(attacks.get_bishop_attacks(Square::D5, occupancy));
}
