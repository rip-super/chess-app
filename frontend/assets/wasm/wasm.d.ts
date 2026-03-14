/* tslint:disable */
/* eslint-disable */

export class ChessEngine {
    free(): void;
    [Symbol.dispose](): void;
    static from_fen(fen: string): ChessEngine;
    game_result(): string;
    is_in_check(): boolean;
    king_square(color: string): number | undefined;
    legal_moves(): any[];
    make_move(mv: ChessMove): void;
    constructor();
    parse_uci(input: string): ChessMove | undefined;
    piece_on(square: number): string | undefined;
    side_to_move(): string;
    undo_move(): void;
}

export class ChessMove {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    from_sq(): number;
    is_capture(): boolean;
    is_castle(): boolean;
    is_double_pawn_push(): boolean;
    is_en_passant(): boolean;
    is_kingside_castle(): boolean;
    is_promotion(): boolean;
    is_queenside_castle(): boolean;
    promotion_piece(): string | undefined;
    to_sq(): number;
    to_uci(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_chessengine_free: (a: number, b: number) => void;
    readonly __wbg_chessmove_free: (a: number, b: number) => void;
    readonly chessengine_from_fen: (a: number, b: number) => number;
    readonly chessengine_game_result: (a: number) => [number, number];
    readonly chessengine_is_in_check: (a: number) => number;
    readonly chessengine_king_square: (a: number, b: number, c: number) => number;
    readonly chessengine_legal_moves: (a: number) => [number, number];
    readonly chessengine_make_move: (a: number, b: number) => [number, number];
    readonly chessengine_new: () => number;
    readonly chessengine_parse_uci: (a: number, b: number, c: number) => number;
    readonly chessengine_piece_on: (a: number, b: number) => [number, number];
    readonly chessengine_side_to_move: (a: number) => [number, number];
    readonly chessengine_undo_move: (a: number) => void;
    readonly chessmove_from_sq: (a: number) => number;
    readonly chessmove_is_capture: (a: number) => number;
    readonly chessmove_is_castle: (a: number) => number;
    readonly chessmove_is_double_pawn_push: (a: number) => number;
    readonly chessmove_is_en_passant: (a: number) => number;
    readonly chessmove_is_kingside_castle: (a: number) => number;
    readonly chessmove_is_promotion: (a: number) => number;
    readonly chessmove_is_queenside_castle: (a: number) => number;
    readonly chessmove_promotion_piece: (a: number) => [number, number];
    readonly chessmove_to_sq: (a: number) => number;
    readonly chessmove_to_uci: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
