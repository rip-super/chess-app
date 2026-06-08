use engine::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Bot {
    seed: u64,
}

impl Bot {
    fn new() -> Self {
        Self::with_depth(0)
    }

    fn with_depth(_depth: u32) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEADBEEF);
        Bot { seed }
    }

    fn with_time(_max_time: Duration) -> Self {
        Self::new()
    }

    fn rand(&mut self) -> u64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }

    fn best_move(&mut self, gs: &mut GameState) -> Option<Move> {
        let moves = gs.position.get_legal_moves();
        if moves.is_empty() {
            return None;
        }
        Some(moves[(self.rand() as usize) % moves.len()])
    }
}

gauntlet::bot_main!();
