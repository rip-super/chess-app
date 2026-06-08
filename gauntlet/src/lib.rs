#[macro_export]
macro_rules! bot_main {
    () => {
        fn move_to_uci(mv: Move) -> String {
            let file = |sq: u8| (b'a' + sq % 8) as char;
            let rank = |sq: u8| (b'1' + sq / 8) as char;
            let mut s = format!(
                "{}{}{}{}",
                file(mv.from),
                rank(mv.from),
                file(mv.to),
                rank(mv.to)
            );
            if let Some(promo) = mv.promotion {
                s.push(match promo {
                    Piece::Queen => 'q',
                    Piece::Rook => 'r',
                    Piece::Bishop => 'b',
                    Piece::Knight => 'n',
                    _ => unreachable!(),
                });
            }
            s
        }

        fn main() {
            use std::io::{BufRead, Write};
            use std::time::Duration;

            let args: Vec<String> = std::env::args().collect();

            let mut bot = if let Some(ms) = args
                .windows(2)
                .find(|w| w[0] == "--time")
                .and_then(|w| w[1].parse::<u64>().ok())
            {
                Bot::with_time(Duration::from_millis(ms))
            } else if let Some(depth) = args
                .windows(2)
                .find(|w| w[0] == "--depth")
                .and_then(|w| w[1].parse::<u32>().ok())
            {
                Bot::with_depth(depth)
            } else {
                Bot::new()
            };

            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            let mut out = stdout.lock();

            for line in stdin.lock().lines() {
                let fen = line.unwrap();
                let fen = fen.trim();
                if fen.is_empty() {
                    continue;
                }
                let mut gs = GameState::from_fen(fen);
                let response = match bot.best_move(&mut gs) {
                    Some(mv) => move_to_uci(mv),
                    None => "none".to_string(),
                };
                writeln!(out, "{}", response).unwrap();
                out.flush().unwrap();
            }
        }
    };
}
