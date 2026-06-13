use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use bot::Bot;
use engine::*;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let bot = Arc::new(Mutex::new(Bot::new()));
    let mut game = GameState::new();

    let search_handle: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let cmd = line.trim();

        let cmd_type = cmd.split_whitespace().next().unwrap_or("");

        match cmd_type {
            "uci" => {
                println!("id name bad_engine");
                println!("id author rip_super");
                println!("uciok");
            }

            "isready" => println!("readyok"),

            "ucinewgame" => {
                game = GameState::new();
            }

            "position" => {
                process_position(cmd, &mut game);
            }

            "go" => {
                if let Some(handle) = search_handle.lock().unwrap().take() {
                    bot.lock().unwrap().stop();
                    let _ = handle.join();
                }

                let bot_clone = bot.clone();
                let mut game_clone = game.clone();

                let cmd_string = cmd.to_string();

                let handle = thread::spawn(move || {
                    let mut bot = bot_clone.lock().unwrap();

                    process_go(&cmd_string, &mut bot, &mut game_clone);
                });

                *search_handle.lock().unwrap() = Some(handle);
            }

            "stop" => {
                if let Some(handle) = search_handle.lock().unwrap().take() {
                    bot.lock().unwrap().stop();
                    let _ = handle.join();
                }
            }

            "quit" => {
                if let Some(handle) = search_handle.lock().unwrap().take() {
                    bot.lock().unwrap().stop();
                    let _ = handle.join();
                }
                break;
            }

            _ => {}
        }

        stdout.flush().unwrap();
    }
}

fn process_position(cmd: &str, game: &mut GameState) {
    let lower = cmd.to_lowercase();

    if lower.contains("startpos") {
        *game = GameState::new();
    } else if lower.contains("fen") {
        let fen = get_labeled(cmd, "fen", &["position", "fen", "moves"]);

        *game = GameState::from_fen(&fen);
    }

    let moves_str = get_labeled(cmd, "moves", &["position", "fen", "moves"]);
    if !moves_str.is_empty() {
        for mv_str in moves_str.split_whitespace() {
            if let Some(mv) = parse_uci_move(&mut game.position, mv_str) {
                let _ = game.make_move(mv);
            } else {
                eprintln!("Invalid move: {}", mv_str);
            }
        }
    }
}

fn process_go(cmd: &str, bot: &mut Bot, game: &mut GameState) {
    let labels = [
        "go",
        "movetime",
        "wtime",
        "btime",
        "winc",
        "binc",
        "movestogo",
    ];

    if cmd.contains("movetime") {
        let ms = get_labeled_int(cmd, "movetime", &labels, 500);
        bot.max_time = Some(Duration::from_millis(ms as u64));
    } else {
        let wtime = get_labeled_int(cmd, "wtime", &labels, 0);
        let btime = get_labeled_int(cmd, "btime", &labels, 0);
        let winc = get_labeled_int(cmd, "winc", &labels, 0);
        let binc = get_labeled_int(cmd, "binc", &labels, 0);

        let side_time = if game.position.side_to_move == Color::White {
            wtime
        } else {
            btime
        };

        let inc = if game.position.side_to_move == Color::White {
            winc
        } else {
            binc
        };

        let think_time = (side_time / 30 + inc).max(10);

        bot.max_time = Some(Duration::from_millis(think_time as u64));
    }

    bot.reset_stop();

    if let Some(best) = bot.best_move(game) {
        println!("bestmove {}", move_to_uci(best));
    } else {
        println!("bestmove 0000");
    }
}

fn get_labeled(text: &str, label: &str, all: &[&str]) -> String {
    let text = text.trim();

    if let Some(start) = text.find(label) {
        let value_start = start + label.len();
        let mut value_end = text.len();

        for &other in all {
            if other != label
                && let Some(idx) = text.find(other)
                && idx > value_start
                && idx < value_end
            {
                value_end = idx;
            }
        }

        return text[value_start..value_end].trim().to_string();
    }

    "".to_string()
}

fn get_labeled_int(text: &str, label: &str, all: &[&str], default: i32) -> i32 {
    let val = get_labeled(text, label, all);

    val.split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_uci_move(pos: &mut Position, s: &str) -> Option<Move> {
    let bytes = s.as_bytes();

    if bytes.len() < 4 {
        return None;
    }

    let from = (bytes[1] - b'1') * 8 + (bytes[0] - b'a');
    let to = (bytes[3] - b'1') * 8 + (bytes[2] - b'a');

    let promo = if bytes.len() > 4 {
        match bytes[4] {
            b'q' => Some(Piece::Queen),
            b'r' => Some(Piece::Rook),
            b'b' => Some(Piece::Bishop),
            b'n' => Some(Piece::Knight),
            _ => None,
        }
    } else {
        None
    };

    pos.get_legal_moves()
        .into_iter()
        .find(|m| m.from == from && m.to == to && m.promotion == promo)
}

fn move_to_uci(mv: Move) -> String {
    let file = |s: u8| (b'a' + (s % 8)) as char;
    let rank = |s: u8| (b'1' + (s / 8)) as char;

    let mut s = String::new();
    s.push(file(mv.from));
    s.push(rank(mv.from));
    s.push(file(mv.to));
    s.push(rank(mv.to));

    if let Some(p) = mv.promotion {
        s.push(match p {
            Piece::Queen => 'q',
            Piece::Rook => 'r',
            Piece::Bishop => 'b',
            Piece::Knight => 'n',
            _ => 'q',
        });
    }

    s
}
