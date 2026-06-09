"""
Runs a tournament between two bot binaries using a positions file.

Each bot is a long-lived process that:
  - reads a FEN on stdin
  - writes a UCI move on stdout
  - repeats until stdin closes

Usage:
    python scripts/run_tournament.py \
        ./target/release/bot1 \
        ./target/release/bot2 \
        scripts/positions.txt \
        --name-a bot1 --name-b bot2 \
        --move-time 100
"""

import argparse
import random
import subprocess
import sys
import time
from dataclasses import dataclass

MAX_PLIES = 400

# region: Bot process

class BotProcess:
    def __init__(self, binary: str, extra_args: list[str], name: str):
        self.name = name
        self._proc = subprocess.Popen(
            [binary] + extra_args,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

    def get_move(self, fen: str) -> str | None:
        """Send a FEN, return the UCI move string, or None on failure."""
        try:
            self._proc.stdin.write(fen + "\n")
            self._proc.stdin.flush()
            move = self._proc.stdout.readline().strip()
            return move if move and move != "none" else None
        except Exception as e:
            print(f"\n[{self.name}] communication error: {e}", file=sys.stderr)
            return None

    def close(self) -> None:
        try:
            self._proc.stdin.close()
            self._proc.wait(timeout=5)
        except Exception:
            self._proc.kill()

# endregion

# region: Game

import chess

def play_game(white: BotProcess, black: BotProcess, start_fen: str) -> str:
    """Returns 'white', 'black', or 'draw'."""
    board = chess.Board(start_fen)

    for _ in range(MAX_PLIES):
        if board.is_game_over(claim_draw=True):
            break

        bot = white if board.turn == chess.WHITE else black
        move_str = bot.get_move(board.fen())

        if move_str is None:
            return "black" if board.turn == chess.WHITE else "white"
        try:
            move = chess.Move.from_uci(move_str)
        except ValueError:
            return "black" if board.turn == chess.WHITE else "white"
        if move not in board.legal_moves:
            return "black" if board.turn == chess.WHITE else "white"

        board.push(move)
    else:
        return "draw"

    result = board.result(claim_draw=True)
    return {"1-0": "white", "0-1": "black"}.get(result, "draw")

# endregion

# region: Tournament

@dataclass
class Results:
    a_wins: int = 0
    b_wins: int = 0
    draws:  int = 0

    @property
    def total(self) -> int:
        return self.a_wins + self.b_wins + self.draws

    def print_summary(self, name_a: str, name_b: str) -> None:
        t = max(self.total, 1)
        print(f"\n{'─' * 44}")
        print(f"  {name_a}  vs  {name_b}")
        print(f"{'─' * 44}")
        print(f"  Games  : {self.total}")
        print(f"  {name_a:<12} : {self.a_wins:4}  ({100 * self.a_wins / t:.1f}%)")
        print(f"  {name_b:<12} : {self.b_wins:4}  ({100 * self.b_wins / t:.1f}%)")
        print(f"  Draws        : {self.draws:4}  ({100 * self.draws / t:.1f}%)")
        print(f"{'─' * 44}")


def run_tournament(
    binary_a: str, binary_b: str,
    positions_file: str,
    n_positions: int,
    extra_a: list[str], extra_b: list[str],
    name_a: str, name_b: str,
    move_time: int | None = None,
) -> Results:
    if move_time is not None:
        extra_a = ["--time", str(move_time)]
        extra_b = ["--time", str(move_time)]

    all_positions = [l.strip() for l in open(positions_file) if l.strip()]

    if len(all_positions) < n_positions:
        print(f"Only {len(all_positions)} positions available; using all.")
        n_positions = len(all_positions)

    selected = random.sample(all_positions, n_positions)
    results = Results()

    bot_a = BotProcess(binary_a, extra_a, name_a)
    bot_b = BotProcess(binary_b, extra_b, name_b)

    start = time.time()

    try:
        for i, fen in enumerate(selected):
            for a_is_white in (True, False):
                white, black = (bot_a, bot_b) if a_is_white else (bot_b, bot_a)
                outcome = play_game(white, black, fen)

                if outcome == "white":
                    (results.a_wins if a_is_white else results.b_wins).__class__
                    if a_is_white: results.a_wins += 1
                    else:          results.b_wins += 1
                elif outcome == "black":
                    if a_is_white: results.b_wins += 1
                    else:          results.a_wins += 1
                else:
                    results.draws += 1

            games_done = (i + 1) * 2
            elapsed = time.time() - start
            rate = games_done / elapsed if elapsed > 0 else 1
            eta = (n_positions * 2 - games_done) / rate
            print(
                f"\r  {i+1}/{n_positions} positions | "
                f"{name_a}: {results.a_wins}  {name_b}: {results.b_wins}  draws: {results.draws} | "
                f"ETA {eta:.0f}s   ",
                end="", flush=True,
            )
    finally:
        bot_a.close()
        bot_b.close()

    print()
    return results

# endregion

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("bot_a",     help="Path to bot A binary")
    parser.add_argument("bot_b",     help="Path to bot B binary")
    parser.add_argument("positions", help="Positions file (one FEN per line)")
    parser.add_argument("--n",       type=int, default=500, dest="n_positions")
    parser.add_argument("--name-a",  default=None)
    parser.add_argument("--name-b",  default=None)
    parser.add_argument("--args-a",  nargs=argparse.REMAINDER, default=[])
    parser.add_argument("--args-b",  nargs=argparse.REMAINDER, default=[])
    parser.add_argument("--move-time", type=int, default=None,
                        help="Max ms per move, applied to both bots")
    
    args = parser.parse_args()

    name_a = args.name_a or args.bot_a
    name_b = args.name_b or args.bot_b

    results = run_tournament(
        args.bot_a, args.bot_b,
        args.positions,
        args.n_positions,
        args.args_a, args.args_b,
        name_a, name_b,
        move_time=args.move_time,
    )

    results.print_summary(name_a, name_b)