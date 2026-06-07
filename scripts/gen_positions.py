"""
Reads a Lichess PGN file and extracts balanced positions filtered by Stockfish.

Usage:
    python gen_positions.py scripts/games.pgn positions.txt
    python gen_positions.py scripts/games.pgn positions.txt --target 5000 --eval-cap 50
"""

import argparse
import random

import chess
import chess.engine
import chess.pgn

def generate(
    pgn_path: str,
    output_path: str,
    target: int = 5000,
    eval_cap_cp: int = 50,
    stockfish_depth: int = 10,
    stockfish_path: str = "stockfish",
    min_ply: int = 16,
    max_ply: int = 40, 
    min_pieces: int = 16,
) -> None:
    found = []
    games_read = 0

    print(f"Targeting {target} positions from {pgn_path} ...")

    with open(pgn_path, encoding="utf-8", errors="ignore") as pgn_file:
        with chess.engine.SimpleEngine.popen_uci(stockfish_path) as sf:
            while len(found) < target:
                game = chess.pgn.read_game(pgn_file)
                if game is None:
                    print("Reached end of file.")
                    break

                games_read += 1
                moves = list(game.mainline_moves())

                if len(moves) < max_ply + 10:
                    continue

                ply = random.randrange(min_ply, max_ply, 2)

                board = game.board()
                for mv in moves[:ply]:
                    board.push(mv)

                if len(board.piece_map()) < min_pieces:
                    continue
                if board.is_check():
                    continue

                info = sf.analyse(board, chess.engine.Limit(depth=stockfish_depth))
                score = info["score"].white()

                if score.is_mate():
                    continue
                if abs(score.score()) > eval_cap_cp:
                    continue

                found.append(board.fen())

                if len(found) % 100 == 0:
                    print(f"  {len(found)}/{target}  (scanned {games_read} games)")

    with open(output_path, "w") as f:
        for fen in found:
            f.write(fen + "\n")

    print(f"\nSaved {len(found)} positions → {output_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("pgn",    help="Path to decompressed Lichess PGN")
    parser.add_argument("output", nargs="?", default="positions.txt")
    parser.add_argument("--target",     type=int,   default=5000)
    parser.add_argument("--eval-cap",   type=int,   default=50)
    parser.add_argument("--sf-depth",   type=int,   default=10)
    parser.add_argument("--stockfish",  default="stockfish")
    args = parser.parse_args()

    generate(
        args.pgn,
        args.output,
        target=args.target,
        eval_cap_cp=args.eval_cap,
        stockfish_depth=args.sf_depth,
        stockfish_path=args.stockfish,
    )