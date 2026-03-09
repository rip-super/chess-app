#![allow(unused)]

// region: Macros

macro_rules! add_attack {
    ($attacks:ident, $shifted:expr, $mask:expr) => {
        if $shifted & $mask != 0 {
            $attacks |= $shifted;
        }
    };
}

// endregion

// region: Constants

const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;
const NOT_HG_FILE: u64 = 0x3F3F3F3F3F3F3F3F;
const NOT_AB_FILE: u64 = 0xFCFCFCFCFCFCFCFC;

#[rustfmt::skip]
const BISHOP_RELEVANT_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 
    5, 5, 5, 5, 5, 5, 5, 5, 
    5, 5, 7, 7, 7, 7, 5, 5, 
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 
    5, 5, 7, 7, 7, 7, 5, 5, 
    5, 5, 5, 5, 5, 5, 5, 5, 
    6, 5, 5, 5, 5, 5, 5, 6,
];

#[rustfmt::skip]
const ROOK_RELEVANT_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    12, 11, 11, 11, 11, 11, 11, 12,
];

const ROOK_MAGIC_NUMBERS: [u64; 64] = [
    0x8a80104000800020,
    0x140002000100040,
    0x2801880a0017001,
    0x100081001000420,
    0x200020010080420,
    0x3001c0002010008,
    0x8480008002000100,
    0x2080088004402900,
    0x800098204000,
    0x2024401000200040,
    0x100802000801000,
    0x120800800801000,
    0x208808088000400,
    0x2802200800400,
    0x2200800100020080,
    0x801000060821100,
    0x80044006422000,
    0x100808020004000,
    0x12108a0010204200,
    0x140848010000802,
    0x481828014002800,
    0x8094004002004100,
    0x4010040010010802,
    0x20008806104,
    0x100400080208000,
    0x2040002120081000,
    0x21200680100081,
    0x20100080080080,
    0x2000a00200410,
    0x20080800400,
    0x80088400100102,
    0x80004600042881,
    0x4040008040800020,
    0x440003000200801,
    0x4200011004500,
    0x188020010100100,
    0x14800401802800,
    0x2080040080800200,
    0x124080204001001,
    0x200046502000484,
    0x480400080088020,
    0x1000422010034000,
    0x30200100110040,
    0x100021010009,
    0x2002080100110004,
    0x202008004008002,
    0x20020004010100,
    0x2048440040820001,
    0x101002200408200,
    0x40802000401080,
    0x4008142004410100,
    0x2060820c0120200,
    0x1001004080100,
    0x20c020080040080,
    0x2935610830022400,
    0x44440041009200,
    0x280001040802101,
    0x2100190040002085,
    0x80c0084100102001,
    0x4024081001000421,
    0x20030a0244872,
    0x12001008414402,
    0x2006104900a0804,
    0x1004081002402,
];

const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
    0x40040844404084,
    0x2004208a004208,
    0x10190041080202,
    0x108060845042010,
    0x581104180800210,
    0x2112080446200010,
    0x1080820820060210,
    0x3c0808410220200,
    0x4050404440404,
    0x21001420088,
    0x24d0080801082102,
    0x1020a0a020400,
    0x40308200402,
    0x4011002100800,
    0x401484104104005,
    0x801010402020200,
    0x400210c3880100,
    0x404022024108200,
    0x810018200204102,
    0x4002801a02003,
    0x85040820080400,
    0x810102c808880400,
    0xe900410884800,
    0x8002020480840102,
    0x220200865090201,
    0x2010100a02021202,
    0x152048408022401,
    0x20080002081110,
    0x4001001021004000,
    0x800040400a011002,
    0xe4004081011002,
    0x1c004001012080,
    0x8004200962a00220,
    0x8422100208500202,
    0x2000402200300c08,
    0x8646020080080080,
    0x80020a0200100808,
    0x2010004880111000,
    0x623000a080011400,
    0x42008c0340209202,
    0x209188240001000,
    0x400408a884001800,
    0x110400a6080400,
    0x1840060a44020800,
    0x90080104000041,
    0x201011000808101,
    0x1a2208080504f080,
    0x8012020600211212,
    0x500861011240000,
    0x180806108200800,
    0x4000020e01040044,
    0x300000261044000a,
    0x802241102020002,
    0x20906061210001,
    0x5a84841004010310,
    0x4010801011c04,
    0xa010109502200,
    0x4a02012000,
    0x500201010098b028,
    0x8040002811040900,
    0x28000010020204,
    0x6000020202d0240,
    0x8918844842082200,
    0x4010011029020020,
];

pub struct Square;
impl Square {
    pub const A1: u8 = 0;
    pub const B1: u8 = 1;
    pub const C1: u8 = 2;
    pub const D1: u8 = 3;
    pub const E1: u8 = 4;
    pub const F1: u8 = 5;
    pub const G1: u8 = 6;
    pub const H1: u8 = 7;

    pub const A2: u8 = 8;
    pub const B2: u8 = 9;
    pub const C2: u8 = 10;
    pub const D2: u8 = 11;
    pub const E2: u8 = 12;
    pub const F2: u8 = 13;
    pub const G2: u8 = 14;
    pub const H2: u8 = 15;

    pub const A3: u8 = 16;
    pub const B3: u8 = 17;
    pub const C3: u8 = 18;
    pub const D3: u8 = 19;
    pub const E3: u8 = 20;
    pub const F3: u8 = 21;
    pub const G3: u8 = 22;
    pub const H3: u8 = 23;

    pub const A4: u8 = 24;
    pub const B4: u8 = 25;
    pub const C4: u8 = 26;
    pub const D4: u8 = 27;
    pub const E4: u8 = 28;
    pub const F4: u8 = 29;
    pub const G4: u8 = 30;
    pub const H4: u8 = 31;

    pub const A5: u8 = 32;
    pub const B5: u8 = 33;
    pub const C5: u8 = 34;
    pub const D5: u8 = 35;
    pub const E5: u8 = 36;
    pub const F5: u8 = 37;
    pub const G5: u8 = 38;
    pub const H5: u8 = 39;

    pub const A6: u8 = 40;
    pub const B6: u8 = 41;
    pub const C6: u8 = 42;
    pub const D6: u8 = 43;
    pub const E6: u8 = 44;
    pub const F6: u8 = 45;
    pub const G6: u8 = 46;
    pub const H6: u8 = 47;

    pub const A7: u8 = 48;
    pub const B7: u8 = 49;
    pub const C7: u8 = 50;
    pub const D7: u8 = 51;
    pub const E7: u8 = 52;
    pub const F7: u8 = 53;
    pub const G7: u8 = 54;
    pub const H7: u8 = 55;

    pub const A8: u8 = 56;
    pub const B8: u8 = 57;
    pub const C8: u8 = 58;
    pub const D8: u8 = 59;
    pub const E8: u8 = 60;
    pub const F8: u8 = 61;
    pub const G8: u8 = 62;
    pub const H8: u8 = 63;
}

// endregion

// region: Attack Tables

mod tables;
use tables::*;

fn get_bishop_attacks(sq: usize, occ: u64) -> u64 {
    let masked = occ & BISHOP_MASKS[sq];
    let idx = masked.wrapping_mul(BISHOP_MAGICS[sq]) >> BISHOP_SHIFTS[sq];
    BISHOP_DATA[BISHOP_OFFSETS[sq] + idx as usize]
}

fn get_rook_attacks(sq: usize, occ: u64) -> u64 {
    let masked = occ & ROOK_MASKS[sq];
    let idx = masked.wrapping_mul(ROOK_MAGICS[sq]) >> ROOK_SHIFTS[sq];
    ROOK_DATA[ROOK_OFFSETS[sq] + idx as usize]
}

// endregion

// region: Enums

#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[repr(usize)]
#[derive(Copy, Clone, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

enum SlidingPiece {
    Bishop,
    Rook,
}

// endregion

// region: BitIter
struct BitIter(u64);

impl BitIter {
    fn new(bits: u64) -> BitIter {
        BitIter(bits)
    }
}

impl Iterator for BitIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let lsb = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(lsb)
    }
}

// endregion

// region: Moves

#[derive(Copy, Clone, PartialEq)]
pub enum MoveFlag {
    Quiet,
    Capture,
    DoublePawnPush,
    KingCastle,
    QueenCastle,
    EnPassant,
    Promotion,
    PromotionCapture,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promotion: Option<Piece>,
    pub flag: MoveFlag,
}

#[derive(Copy, Clone)]
pub struct Undo {
    moved_piece: Piece,
    captured: Option<Piece>,
    castling_rights: u8,
    en_passant: Option<u8>,
    halfmove_clock: u32,
    fullmove_count: u32,
}

// endregion

// region: Bitboards

#[derive(Clone, Copy)]
pub struct Bitboards {
    // [color][piece]
    pub pieces: [[u64; 6]; 2],
    color: [u64; 2],
    all: u64,
    mailbox: [Option<(u8, u8)>; 64],
}

impl Bitboards {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut bb = Bitboards {
            pieces: [
                [
                    0x000000000000FF00,
                    0x0000000000000042,
                    0x0000000000000024,
                    0x0000000000000081,
                    0x0000000000000008,
                    0x0000000000000010,
                ],
                [
                    0x00FF000000000000,
                    0x4200000000000000,
                    0x2400000000000000,
                    0x8100000000000000,
                    0x0800000000000000,
                    0x1000000000000000,
                ],
            ],
            color: [0; 2],
            all: 0,
            mailbox: [None; 64],
        };
        bb.recompute();
        bb
    }

    pub fn empty() -> Self {
        Bitboards {
            pieces: [[0u64; 6]; 2],
            color: [0; 2],
            all: 0,
            mailbox: [None; 64],
        }
    }

    pub fn recompute(&mut self) {
        self.color[0] = self.pieces[0].iter().fold(0, |a, b| a | b);
        self.color[1] = self.pieces[1].iter().fold(0, |a, b| a | b);
        self.all = self.color[0] | self.color[1];

        self.mailbox = [None; 64];
        for c in 0..2usize {
            for p in 0..6usize {
                let mut bb = self.pieces[c][p];
                while bb != 0 {
                    let sq = bb.trailing_zeros() as u8;
                    self.mailbox[sq as usize] = Some((c as u8, p as u8));
                    bb &= bb - 1;
                }
            }
        }
    }

    #[inline(always)]
    pub fn set_piece(&mut self, color: usize, piece: usize, sq: u8) {
        let mask = 1u64 << sq;
        self.pieces[color][piece] |= mask;
        self.color[color] |= mask;
        self.all |= mask;
        self.mailbox[sq as usize] = Some((color as u8, piece as u8));
    }

    #[inline(always)]
    pub fn clear_piece(&mut self, color: usize, piece: usize, sq: u8) {
        let mask = 1u64 << sq;
        self.pieces[color][piece] &= !mask;
        self.color[color] &= !mask;
        self.all &= !mask;
        self.mailbox[sq as usize] = None;
    }
}

// endregion

// region: Rng

struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Rng {
        Rng { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;

        self.state
    }

    fn next_u64(&mut self) -> u64 {
        let n1 = (self.next_u32() as u64) & 0xFFFF;
        let n2 = (self.next_u32() as u64) & 0xFFFF;
        let n3 = (self.next_u32() as u64) & 0xFFFF;
        let n4 = (self.next_u32() as u64) & 0xFFFF;

        n1 | (n2 << 16) | (n3 << 32) | (n4 << 48)
    }

    fn generate_magic_number(&mut self) -> u64 {
        self.next_u64() & self.next_u64() & self.next_u64()
    }
}

// endregion

// region: Zobrist Hash

#[derive(Clone)]
pub struct ZobristKeys {
    pieces: [[[u64; 64]; 6]; 2], // [color][piece][square]
    castling: [u64; 4],
    en_passant: [u64; 8],
    side_to_move: u64,
}

impl ZobristKeys {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut rng = Rng::new(0x3D1E32C2);

        let mut pieces = [[[0; 64]; 6]; 2];
        for color in &mut pieces {
            for piece in color {
                for sq in piece {
                    *sq = rng.next_u64();
                }
            }
        }

        let mut castling = [0u64; 4];
        for castle in &mut castling {
            *castle = rng.next_u64();
        }

        let mut en_passant = [0u64; 8];
        for ep in &mut en_passant {
            *ep = rng.next_u64();
        }

        let side_to_move = rng.next_u64();

        Self {
            pieces,
            castling,
            en_passant,
            side_to_move,
        }
    }
}

// endregion

// region: Position

#[derive(Clone)]
pub struct Position {
    pub bitboards: Bitboards,
    pub side_to_move: Color,
    pub castling_rights: u8, // 1 bit per thing
    pub en_passant: Option<u8>,
    halfmove_clock: u32,
    fullmove_count: u32,
    zobrist_keys: ZobristKeys,
}

impl Position {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            bitboards: Bitboards::new(),
            side_to_move: Color::White,
            castling_rights: 0b1111,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_count: 1,
            zobrist_keys: ZobristKeys::new(),
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        if fen == "startpos" {
            return Self::new();
        }

        let parts: Vec<&str> = fen.split_whitespace().collect();
        assert!(parts.len() >= 6, "Invalid FEN: expected at least 6 fields");

        let piece_placement = parts[0];
        let active_color = parts[1];
        let castling_rights_str = parts[2];
        let en_passant_str = parts[3];
        let halfmove_clock: u32 = parts[4].parse().unwrap_or(0);
        let fullmove_count: u32 = parts[5].parse().unwrap_or(1);

        let mut bitboards = Bitboards::empty();

        for (rank_idx, row) in piece_placement.split("/").enumerate() {
            let mut file = 0;
            for ch in row.chars() {
                if ch.is_ascii_digit() {
                    file += ch.to_digit(10).unwrap() as usize;
                } else {
                    let sq = (7 - rank_idx) * 8 + file;
                    let (color, piece) = match ch {
                        'P' => (Color::White, Piece::Pawn),
                        'N' => (Color::White, Piece::Knight),
                        'B' => (Color::White, Piece::Bishop),
                        'R' => (Color::White, Piece::Rook),
                        'Q' => (Color::White, Piece::Queen),
                        'K' => (Color::White, Piece::King),
                        'p' => (Color::Black, Piece::Pawn),
                        'n' => (Color::Black, Piece::Knight),
                        'b' => (Color::Black, Piece::Bishop),
                        'r' => (Color::Black, Piece::Rook),
                        'q' => (Color::Black, Piece::Queen),
                        'k' => (Color::Black, Piece::King),
                        _ => panic!("Invalid piece in FEN: {}", ch),
                    };

                    bitboards.pieces[color as usize][piece as usize] |= 1 << sq;
                    file += 1;
                }
            }
        }

        assert!(
            bitboards.pieces[Color::White as usize][Piece::King as usize] != 0,
            "No white king"
        );
        assert!(
            bitboards.pieces[Color::Black as usize][Piece::King as usize] != 0,
            "No black king"
        );

        bitboards.recompute();

        let side_to_move = match active_color {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Invalid active color: {}", active_color),
        };

        let mut castling_rights = 0u8;
        for c in castling_rights_str.chars() {
            match c {
                'K' => castling_rights |= 1 << 0,
                'Q' => castling_rights |= 1 << 1,
                'k' => castling_rights |= 1 << 2,
                'q' => castling_rights |= 1 << 3,
                '-' => {}
                _ => panic!("Invalid castling right: {}", c),
            }
        }

        let en_passant = if en_passant_str != "-" {
            let b = en_passant_str.as_bytes();
            let file = b[0] - b'a';
            let rank = b[1] - b'1';
            Some(rank * 8 + file)
        } else {
            None
        };

        Self {
            bitboards,
            side_to_move,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_count,
            zobrist_keys: ZobristKeys::new(),
        }
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let sq = rank * 8 + file;
                if let Some((color, piece)) = self.piece_on(sq) {
                    if empty_count != 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    let ch = match (color, piece) {
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
                    };
                    fen.push(ch);
                } else {
                    empty_count += 1;
                }
            }
            if empty_count != 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank != 0 {
                fen.push('/');
            }
        }

        let stm = match self.side_to_move {
            Color::White => " w",
            Color::Black => " b",
        };
        fen.push_str(stm);

        let mut castling = String::new();
        if self.castling_rights & 1 != 0 {
            castling.push('K');
        }
        if self.castling_rights & 2 != 0 {
            castling.push('Q');
        }
        if self.castling_rights & 4 != 0 {
            castling.push('k');
        }
        if self.castling_rights & 8 != 0 {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }
        fen.push(' ');
        fen.push_str(&castling);

        fen.push(' ');
        if let Some(ep) = self.en_passant {
            let file = (ep % 8) + b'a';
            let rank = (ep / 8) + b'1';
            fen.push(file as char);
            fen.push(rank as char);
        } else {
            fen.push('-');
        }

        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        fen.push(' ');
        fen.push_str(&self.fullmove_count.to_string());

        fen
    }

    fn zobrist_hash(&self) -> u64 {
        let mut hash = 0;

        for color in 0..2 {
            for piece in 0..6 {
                let mut bb = self.bitboards.pieces[color][piece];
                while bb != 0 {
                    let sq = bb.trailing_zeros() as usize;
                    hash ^= self.zobrist_keys.pieces[color][piece][sq];
                    bb &= bb - 1;
                }
            }
        }

        if self.castling_rights & 0b0001 != 0 {
            hash ^= self.zobrist_keys.castling[0];
        }
        if self.castling_rights & 0b0010 != 0 {
            hash ^= self.zobrist_keys.castling[1];
        }
        if self.castling_rights & 0b0100 != 0 {
            hash ^= self.zobrist_keys.castling[2];
        }
        if self.castling_rights & 0b1000 != 0 {
            hash ^= self.zobrist_keys.castling[3];
        }

        if let Some(ep) = self.en_passant {
            let file = ep % 8;
            hash ^= self.zobrist_keys.en_passant[file as usize];
        }

        if self.side_to_move == Color::White {
            hash ^= self.zobrist_keys.side_to_move;
        }

        hash
    }

    pub fn piece_on(&self, square: u8) -> Option<(Color, Piece)> {
        self.bitboards.mailbox[square as usize].map(|(c, p)| {
            let color = if c == 0 { Color::White } else { Color::Black };
            let piece = match p {
                0 => Piece::Pawn,
                1 => Piece::Knight,
                2 => Piece::Bishop,
                3 => Piece::Rook,
                4 => Piece::Queen,
                _ => Piece::King,
            };
            (color, piece)
        })
    }

    fn square_under_attack(&self, square: u8, by_color: Color) -> bool {
        let sq = square as usize;
        let color_idx = by_color.opposite() as usize;
        let board_idx = by_color as usize;

        if (PAWN_ATTACKS[color_idx][sq] & self.bitboards.pieces[board_idx][Piece::Pawn as usize])
            != 0
        {
            return true;
        }

        let idx = by_color as usize;

        if (KNIGHT_ATTACKS[sq] & self.bitboards.pieces[idx][Piece::Knight as usize]) != 0
            || (KING_ATTACKS[sq] & self.bitboards.pieces[idx][Piece::King as usize]) != 0
        {
            return true;
        }

        let occ = self.bitboards.all;

        let bishop_attacks = get_bishop_attacks(sq, occ);
        if bishop_attacks
            & (self.bitboards.pieces[idx][Piece::Bishop as usize]
                | self.bitboards.pieces[idx][Piece::Queen as usize])
            != 0
        {
            return true;
        }

        let rook_attacks = get_rook_attacks(sq, occ);
        if rook_attacks
            & (self.bitboards.pieces[idx][Piece::Rook as usize]
                | self.bitboards.pieces[idx][Piece::Queen as usize])
            != 0
        {
            return true;
        }

        false
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let idx = color as usize;

        let king_bb = self.bitboards.pieces[idx][Piece::King as usize];
        assert!(king_bb != 0, "No king on the board!");

        let king_sq = king_bb.trailing_zeros() as u8;

        self.square_under_attack(king_sq, color.opposite())
    }

    pub fn make_move(&mut self, mv: Move) -> Undo {
        let moved_piece = self.piece_on(mv.from).expect("No piece on from-square").1;
        let color_idx = self.side_to_move as usize;

        let undo = Undo {
            moved_piece,
            captured: self.piece_on(mv.to).map(|(_, piece)| piece),
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            fullmove_count: self.fullmove_count,
        };

        let is_capture = matches!(
            mv.flag,
            MoveFlag::Capture | MoveFlag::PromotionCapture | MoveFlag::EnPassant
        );
        let is_pawn_move = moved_piece == Piece::Pawn
            || matches!(mv.flag, MoveFlag::Promotion | MoveFlag::PromotionCapture);

        if is_capture || is_pawn_move {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if let Some(captured) = undo.captured {
            if mv.flag != MoveFlag::EnPassant {
                let captured_color_idx = 1 - color_idx;
                self.bitboards
                    .clear_piece(captured_color_idx, captured as usize, mv.to);
            }
        }

        if undo.captured == Some(Piece::Rook) && mv.flag != MoveFlag::EnPassant {
            match mv.to {
                0 => self.castling_rights &= !0b0010,
                7 => self.castling_rights &= !0b0001,
                56 => self.castling_rights &= !0b1000,
                63 => self.castling_rights &= !0b0100,
                _ => {}
            }
        }

        match mv.flag {
            MoveFlag::Promotion | MoveFlag::PromotionCapture => {
                let promo_idx = mv.promotion.expect("Promotion move missing piece") as usize;
                self.bitboards
                    .clear_piece(color_idx, Piece::Pawn as usize, mv.from);
                self.bitboards.set_piece(color_idx, promo_idx, mv.to);
            }
            _ => {
                self.bitboards
                    .clear_piece(color_idx, moved_piece as usize, mv.from);
                self.bitboards
                    .set_piece(color_idx, moved_piece as usize, mv.to);
            }
        }

        if mv.flag == MoveFlag::EnPassant {
            let cap_sq = if self.side_to_move == Color::White {
                mv.to - 8
            } else {
                mv.to + 8
            };
            self.bitboards
                .clear_piece(1 - color_idx, Piece::Pawn as usize, cap_sq);
        }

        match mv.flag {
            MoveFlag::KingCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (7u8, 5u8)
                } else {
                    (63u8, 61u8)
                };
                self.bitboards
                    .clear_piece(color_idx, Piece::Rook as usize, rook_from);
                self.bitboards
                    .set_piece(color_idx, Piece::Rook as usize, rook_to);
            }
            MoveFlag::QueenCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (0u8, 3u8)
                } else {
                    (56u8, 59u8)
                };
                self.bitboards
                    .clear_piece(color_idx, Piece::Rook as usize, rook_from);
                self.bitboards
                    .set_piece(color_idx, Piece::Rook as usize, rook_to);
            }
            _ => {}
        }

        match self.side_to_move {
            Color::White => {
                if moved_piece == Piece::King {
                    self.castling_rights &= !0b0011;
                }
                if moved_piece == Piece::Rook {
                    if mv.from == 0 {
                        self.castling_rights &= !0b0010;
                    }
                    if mv.from == 7 {
                        self.castling_rights &= !0b0001;
                    }
                }
            }
            Color::Black => {
                if moved_piece == Piece::King {
                    self.castling_rights &= !0b1100;
                }
                if moved_piece == Piece::Rook {
                    if mv.from == 56 {
                        self.castling_rights &= !0b1000;
                    }
                    if mv.from == 63 {
                        self.castling_rights &= !0b0100;
                    }
                }
            }
        }

        self.en_passant = match mv.flag {
            MoveFlag::DoublePawnPush => Some(if self.side_to_move == Color::White {
                mv.to - 8
            } else {
                mv.to + 8
            }),
            _ => None,
        };

        if self.side_to_move == Color::Black {
            self.fullmove_count += 1;
        }

        self.side_to_move = self.side_to_move.opposite();

        undo
    }

    pub fn undo_move(&mut self, mv: Move, undo: Undo) {
        self.side_to_move = self.side_to_move.opposite();
        let color_idx = self.side_to_move as usize;

        match mv.flag {
            MoveFlag::Promotion | MoveFlag::PromotionCapture => {
                let promo_idx = mv.promotion.unwrap() as usize;
                self.bitboards.clear_piece(color_idx, promo_idx, mv.to);
                self.bitboards
                    .set_piece(color_idx, Piece::Pawn as usize, mv.from);
            }
            _ => {
                self.bitboards
                    .clear_piece(color_idx, undo.moved_piece as usize, mv.to);
                self.bitboards
                    .set_piece(color_idx, undo.moved_piece as usize, mv.from);
            }
        }

        if let Some(captured) = undo.captured {
            if mv.flag != MoveFlag::EnPassant {
                self.bitboards
                    .set_piece(1 - color_idx, captured as usize, mv.to);
            }
        }

        if mv.flag == MoveFlag::EnPassant {
            let cap_sq = if self.side_to_move == Color::White {
                mv.to - 8
            } else {
                mv.to + 8
            };
            self.bitboards
                .set_piece(1 - color_idx, Piece::Pawn as usize, cap_sq);
        }

        match mv.flag {
            MoveFlag::KingCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (7u8, 5u8)
                } else {
                    (63u8, 61u8)
                };
                self.bitboards
                    .clear_piece(color_idx, Piece::Rook as usize, rook_to);
                self.bitboards
                    .set_piece(color_idx, Piece::Rook as usize, rook_from);
            }
            MoveFlag::QueenCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (0u8, 3u8)
                } else {
                    (56u8, 59u8)
                };
                self.bitboards
                    .clear_piece(color_idx, Piece::Rook as usize, rook_to);
                self.bitboards
                    .set_piece(color_idx, Piece::Rook as usize, rook_from);
            }
            _ => {}
        }

        self.castling_rights = undo.castling_rights;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
    }

    fn get_pawn_moves(&self, moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        let idx = color as usize;
        let enemy_idx = color.opposite() as usize;

        let our_pawns = self.bitboards.pieces[idx][Piece::Pawn as usize];
        let enemy_occupancy = self.bitboards.color[color.opposite() as usize];
        let empty = !self.bitboards.all;

        let single_pushes = if color == Color::White {
            (our_pawns << 8) & empty
        } else {
            (our_pawns >> 8) & empty
        };

        for to in BitIter::new(single_pushes) {
            let from = if color == Color::White {
                to - 8
            } else {
                to + 8
            };
            let promotion_rank = if color == Color::White { 56..64 } else { 0..8 };

            if promotion_rank.contains(&to) {
                for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                    moves.push(Move {
                        from,
                        to,
                        promotion: Some(promo),
                        flag: MoveFlag::Promotion,
                    });
                }
            } else {
                moves.push(Move {
                    from,
                    to,
                    promotion: None,
                    flag: MoveFlag::Quiet,
                });
            }
        }

        let double_push_mask = if color == Color::White {
            0x000000000000FF00
        } else {
            0x00FF000000000000
        };
        let single_push_pawns = if color == Color::White {
            (our_pawns & double_push_mask) << 8 & empty
        } else {
            (our_pawns & double_push_mask) >> 8 & empty
        };
        let double_pushes = if color == Color::White {
            (single_push_pawns << 8) & empty
        } else {
            (single_push_pawns >> 8) & empty
        };
        for to in BitIter::new(double_pushes) {
            let from = if color == Color::White {
                to - 16
            } else {
                to + 16
            };
            moves.push(Move {
                from,
                to,
                promotion: None,
                flag: MoveFlag::DoublePawnPush,
            });
        }

        let pawn_idx = if color == Color::White { 0 } else { 1 };
        for from in BitIter::new(our_pawns) {
            let attacks = PAWN_ATTACKS[pawn_idx][from as usize];
            let captures = attacks & enemy_occupancy;

            for to in BitIter::new(captures) {
                let promotion_rank = if color == Color::White { 56..64 } else { 0..8 };
                if promotion_rank.contains(&to) {
                    for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                        moves.push(Move {
                            from,
                            to,
                            promotion: Some(promo),
                            flag: MoveFlag::PromotionCapture,
                        });
                    }
                } else {
                    moves.push(Move {
                        from,
                        to,
                        promotion: None,
                        flag: MoveFlag::Capture,
                    });
                }
            }
        }

        if let Some(ep_sq) = self.en_passant {
            let candidates = if color == Color::White {
                our_pawns & 0x000000FF00000000
            } else {
                our_pawns & 0x00000000FF000000
            };
            for from in BitIter::new(candidates) {
                let attacks = PAWN_ATTACKS[pawn_idx][from as usize];
                if attacks & (1 << ep_sq) != 0 {
                    moves.push(Move {
                        from,
                        to: ep_sq,
                        promotion: None,
                        flag: MoveFlag::EnPassant,
                    });
                }
            }
        }
    }

    fn get_castle_moves(&self, moves: &mut Vec<Move>) {
        let rank = if self.side_to_move == Color::White {
            0
        } else {
            7
        };
        let enemy_color = self.side_to_move.opposite();

        if self.is_in_check(self.side_to_move) {
            return;
        }

        let e = rank * 8 + 4;

        let king_side_mask = if self.side_to_move == Color::White {
            0b0001
        } else {
            0b0100
        };
        if self.castling_rights & king_side_mask != 0 {
            let f = rank * 8 + 5;
            let g = rank * 8 + 6;
            let rook_sq = rank * 8 + 7;

            let rook_ok = matches!(
                self.piece_on(rook_sq as u8),
                Some((c, Piece::Rook)) if c == self.side_to_move
            );

            if rook_ok
                && self.piece_on(f as u8).is_none()
                && self.piece_on(g as u8).is_none()
                && !self.square_under_attack(f as u8, enemy_color)
                && !self.square_under_attack(g as u8, enemy_color)
            {
                moves.push(Move {
                    from: e as u8,
                    to: g as u8,
                    promotion: None,
                    flag: MoveFlag::KingCastle,
                });
            }
        }

        let queen_side_mask = if self.side_to_move == Color::White {
            0b0010
        } else {
            0b1000
        };
        if self.castling_rights & queen_side_mask != 0 {
            let d = rank * 8 + 3;
            let c = rank * 8 + 2;
            let b = rank * 8 + 1;
            let rook_sq = rank * 8;

            let rook_ok = matches!(
                self.piece_on(rook_sq as u8),
                Some((col, Piece::Rook)) if col == self.side_to_move
            );

            if rook_ok
                && self.piece_on(d as u8).is_none()
                && self.piece_on(c as u8).is_none()
                && self.piece_on(b as u8).is_none()
                && !self.square_under_attack(d as u8, enemy_color)
                && !self.square_under_attack(c as u8, enemy_color)
            {
                moves.push(Move {
                    from: e as u8,
                    to: c as u8,
                    promotion: None,
                    flag: MoveFlag::QueenCastle,
                });
            }
        }
    }

    fn get_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(80);
        let color = self.side_to_move;
        let idx = color as usize;
        let enemy_idx = color.opposite() as usize;

        let all_occupancy = self.bitboards.all;
        let our_pieces = self.bitboards.color[idx];
        let enemy_occupancy = self.bitboards.color[enemy_idx];

        use Piece::*;

        self.get_pawn_moves(&mut moves);

        let knights = self.bitboards.pieces[idx][Knight as usize];
        for from in BitIter::new(knights) {
            let attacks = KNIGHT_ATTACKS[from as usize] & !our_pieces;
            for to in BitIter::new(attacks) {
                let flag = if (1 << to) & enemy_occupancy != 0 {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                moves.push(Move {
                    from,
                    to,
                    promotion: None,
                    flag,
                });
            }
        }

        let bishops = self.bitboards.pieces[idx][Bishop as usize];
        for from in BitIter::new(bishops) {
            let attacks = get_bishop_attacks(from as usize, all_occupancy) & !our_pieces;
            for to in BitIter::new(attacks) {
                let flag = if (1 << to) & enemy_occupancy != 0 {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                moves.push(Move {
                    from,
                    to,
                    promotion: None,
                    flag,
                });
            }
        }

        let rooks = self.bitboards.pieces[idx][Rook as usize];
        for from in BitIter::new(rooks) {
            let attacks = get_rook_attacks(from as usize, all_occupancy) & !our_pieces;
            for to in BitIter::new(attacks) {
                let flag = if (1 << to) & enemy_occupancy != 0 {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                moves.push(Move {
                    from,
                    to,
                    promotion: None,
                    flag,
                });
            }
        }

        let queens = self.bitboards.pieces[idx][Queen as usize];
        for from in BitIter::new(queens) {
            let attacks = (get_bishop_attacks(from as usize, all_occupancy)
                | get_rook_attacks(from as usize, all_occupancy))
                & !our_pieces;
            for to in BitIter::new(attacks) {
                let flag = if (1 << to) & enemy_occupancy != 0 {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                moves.push(Move {
                    from,
                    to,
                    promotion: None,
                    flag,
                });
            }
        }

        let king = self.bitboards.pieces[idx][King as usize];
        for from in BitIter::new(king) {
            let attacks = KING_ATTACKS[from as usize] & !our_pieces;
            for to in BitIter::new(attacks) {
                let flag = if (1u64 << to) & enemy_occupancy != 0 {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };

                moves.push(Move {
                    from,
                    to,
                    promotion: None,
                    flag,
                });
            }
        }

        self.get_castle_moves(&mut moves);

        moves
    }

    pub fn get_legal_moves(&mut self) -> Vec<Move> {
        let pseudo = self.get_pseudo_legal_moves();
        let mut legal = Vec::with_capacity(pseudo.len());

        let side = self.side_to_move;

        for mv in pseudo {
            let undo = self.make_move(mv);

            if !self.is_in_check(side) {
                legal.push(mv);
            }

            self.undo_move(mv, undo);
        }

        legal
    }
}

// endregion

// region: GameState

use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameResult {
    Ongoing,
    Checkmate(Color),
    Stalemate,
    DrawRepetition,
    DrawFiftyMove,
    DrawInsufficientMaterial,
}

pub struct GameState {
    pub position: Position,
    history: Vec<(Move, Undo)>,
    repetition_table: HashMap<u64, u32>,
    pub result: GameResult,
}

impl GameState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut position = Position::new();

        let mut repetition_table = HashMap::new();
        repetition_table.insert(position.zobrist_hash(), 1);

        Self {
            position,
            history: Vec::new(),
            repetition_table,
            result: GameResult::Ongoing,
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        let position = Position::from_fen(fen);

        Self {
            position,
            history: Vec::new(),
            repetition_table: HashMap::new(),
            result: GameResult::Ongoing,
        }
    }

    pub fn make_move(&mut self, mv: Move) -> Result<(), &str> {
        if self.result != GameResult::Ongoing {
            return Err("Game already finished");
        }

        let legal_before = self.position.get_legal_moves();
        if !legal_before.contains(&mv) {
            return Err("Illegal Move");
        }

        let undo = self.position.make_move(mv);
        self.history.push((mv, undo));

        self.update_repetition();

        let legal_after = self.position.get_legal_moves();
        self.update_game_result(&legal_after);

        Ok(())
    }

    pub fn undo_move(&mut self) {
        if let Some((mv, undo)) = self.history.pop() {
            let hash_to_remove = self.position.zobrist_hash();

            self.position.undo_move(mv, undo);

            self.decrement_repetition(hash_to_remove);

            let legal = self.position.get_legal_moves();
            self.update_game_result(&legal);
        }
    }

    fn update_repetition(&mut self) {
        let hash = self.position.zobrist_hash();
        let count = self.repetition_table.entry(hash).or_insert(0);
        *count += 1;
    }

    fn decrement_repetition(&mut self, hash: u64) {
        if let Some(count) = self.repetition_table.get_mut(&hash) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.repetition_table.remove(&hash);
            }
        }
    }

    fn is_insufficient_material(&self) -> bool {
        use Color::*;
        use Piece::*;

        let wp = self.position.bitboards.pieces[White as usize][Pawn as usize];
        let bp = self.position.bitboards.pieces[Black as usize][Pawn as usize];
        let wr = self.position.bitboards.pieces[White as usize][Rook as usize];
        let br = self.position.bitboards.pieces[Black as usize][Rook as usize];
        let wq = self.position.bitboards.pieces[White as usize][Queen as usize];
        let bq = self.position.bitboards.pieces[Black as usize][Queen as usize];
        let wn = self.position.bitboards.pieces[White as usize][Knight as usize];
        let bn = self.position.bitboards.pieces[Black as usize][Knight as usize];
        let wb = self.position.bitboards.pieces[White as usize][Bishop as usize];
        let bb = self.position.bitboards.pieces[Black as usize][Bishop as usize];

        if wp != 0 || bp != 0 || wr != 0 || br != 0 || wq != 0 || bq != 0 {
            return false;
        }

        let white_minor_count = (wn | wb).count_ones();
        let black_minor_count = (bn | bb).count_ones();

        if white_minor_count == 0 && black_minor_count == 0 {
            return true;
        }

        if (white_minor_count == 1 && black_minor_count == 0)
            || (white_minor_count == 0 && black_minor_count == 1)
        {
            return true;
        }

        if white_minor_count == 1 && black_minor_count == 1 && wn == 0 && bn == 0 {
            let light_squares: u64 = 0x55AA55AA55AA55AA;
            let dark_squares: u64 = !light_squares;

            let white_bishop_on_light = (wb & light_squares) != 0;
            let black_bishop_on_light = (bb & light_squares) != 0;

            return white_bishop_on_light == black_bishop_on_light;
        }

        if (white_minor_count == 1 && black_minor_count == 1) {
            let white_has_bishop = wb != 0;
            let black_has_bishop = bb != 0;
            let white_has_knight = wn != 0;
            let black_has_knight = bn != 0;

            if (white_has_bishop && black_has_knight && wn == 0 && bb == 0)
                || (white_has_knight && black_has_bishop && wb == 0 && bn == 0)
            {
                return true;
            }
        }

        false
    }

    fn update_game_result(&mut self, legal: &[Move]) {
        if legal.is_empty() {
            if self.position.is_in_check(self.position.side_to_move) {
                let winner = self.position.side_to_move.opposite();
                self.result = GameResult::Checkmate(winner);
            } else {
                self.result = GameResult::Stalemate;
            }
            return;
        }

        if self.position.halfmove_clock >= 100 {
            self.result = GameResult::DrawFiftyMove;
            return;
        }

        let hash = self.position.zobrist_hash();
        if let Some(&count) = self.repetition_table.get(&hash) {
            if count >= 3 {
                self.result = GameResult::DrawRepetition;
                return;
            }
        }

        if self.is_insufficient_material() {
            self.result = GameResult::DrawInsufficientMaterial;
            return;
        }

        self.result = GameResult::Ongoing;
    }
}

// endregion

// region: Perft

fn perft(gs: &mut GameState, depth: u32) -> u64 {
    fn perft_pos(pos: &mut Position, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }

        let moves = pos.get_pseudo_legal_moves();
        let mut nodes = 0;
        let side_before = pos.side_to_move;

        for mv in moves {
            let undo = pos.make_move(mv);

            if !pos.is_in_check(side_before) {
                if depth == 1 {
                    nodes += 1;
                } else {
                    nodes += perft_pos(pos, depth - 1);
                }
            }

            pos.undo_move(mv, undo);
        }

        nodes
    }

    perft_pos(&mut gs.position, depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod perft {
        use super::*;

        #[test]
        fn perft_depth_0() {
            let mut gs = GameState::new();
            let actual = perft(&mut gs, 0);
            let expected = 1;
            assert_eq!(
                actual, expected,
                "Perft(0): expected {}, got {}",
                expected, actual
            );
        }

        #[test]
        fn perft_depth_1() {
            let mut gs = GameState::new();
            let actual = perft(&mut gs, 1);
            let expected = 20;
            assert_eq!(
                actual, expected,
                "Perft(1): expected {}, got {}",
                expected, actual
            );
        }

        #[test]
        fn perft_depth_2() {
            let mut gs = GameState::new();
            let actual = perft(&mut gs, 2);
            let expected = 400;
            assert_eq!(
                actual, expected,
                "Perft(2): expected {}, got {}",
                expected, actual
            );
        }

        #[test]
        fn perft_depth_3() {
            let mut gs = GameState::new();
            let actual = perft(&mut gs, 3);
            let expected = 8902;
            assert_eq!(
                actual, expected,
                "Perft(3): expected {}, got {}",
                expected, actual
            );
        }

        #[test]
        fn perft_depth_4() {
            let mut gs = GameState::new();
            let actual = perft(&mut gs, 4);
            let expected = 197281;
            assert_eq!(
                actual, expected,
                "Perft(4): expected {}, got {}",
                expected, actual
            );
        }

        #[test]
        fn perft_depth_5() {
            let mut gs = GameState::new();
            let actual = perft(&mut gs, 5);
            let expected = 4865609;
            assert_eq!(
                actual, expected,
                "Perft(5): expected {}, got {}",
                expected, actual
            );
        }

        #[test]
        fn perft_depth_6() {
            let mut gs = GameState::new();
            let actual = perft(&mut gs, 6);
            let expected = 119060324;
            assert_eq!(
                actual, expected,
                "Perft(6): expected {}, got {}",
                expected, actual
            );
        }
    }
}

// endregion
