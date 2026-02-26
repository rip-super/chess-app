use engine::*;

fn print_bitboard(board: u64) {
    println!();

    for rank in 0..8 {
        for file in 0..8 {
            let square = Square::new(rank * 8 + file).unwrap();

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
    let mut board = 0;

    set_bit(&mut board, Square::E4);
    set_bit(&mut board, Square::C3);
    set_bit(&mut board, Square::F2);
    print_bitboard(board);

    pop_bit(&mut board, Square::E4);
    print_bitboard(board);
}
