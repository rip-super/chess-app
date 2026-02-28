#![allow(unused)]

// region: Macros

macro_rules! nofmt {
    ($($code:tt)*) => { $($code)* }
}

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

// region: Bit Helpers

pub fn get_bit(board: u64, square: usize) -> usize {
    ((board >> square) & 1) as usize
}

pub fn set_bit(board: &mut u64, square: usize) {
    *board |= 1 << square
}

pub fn pop_bit(board: &mut u64, square: usize) {
    *board &= !(1 << square)
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

// region: Magics

fn find_magic_number(
    square: usize,
    relevant_bits: u32,
    sliding_piece: SlidingPiece,
    rng: &mut Rng,
) -> u64 {
    let mut occupancies = [0u64; 4096];
    let mut attacks = [0u64; 4096];
    let mut used_attacks = [0u64; 4096];

    let attack_mask = match sliding_piece {
        SlidingPiece::Bishop => AttackTables::mask_bishop_attacks(square),
        SlidingPiece::Rook => AttackTables::mask_rook_attacks(square),
    };

    let occupancy_indices = 1 << relevant_bits;

    for index in 0..occupancy_indices {
        occupancies[index] = AttackTables::set_occupancy(index as u32, relevant_bits, attack_mask);
        attacks[index] = match sliding_piece {
            SlidingPiece::Bishop => AttackTables::bishop_attacks_fly(square, occupancies[index]),
            SlidingPiece::Rook => AttackTables::rook_attacks_fly(square, occupancies[index]),
        };
    }

    for _ in 0..100_000_000 {
        let magic_number = rng.generate_magic_number();

        if ((attack_mask.wrapping_mul(magic_number)) & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }

        used_attacks.fill(0);

        let mut fail = false;

        for index in 0..occupancy_indices {
            let magic_index =
                ((occupancies[index].wrapping_mul(magic_number)) >> (64 - relevant_bits)) as usize;

            if used_attacks[magic_index] == 0 {
                used_attacks[magic_index] = attacks[index];
            } else if used_attacks[magic_index] != attacks[index] {
                fail = true;
                break;
            }
        }

        if !fail {
            return magic_number;
        }
    }

    println!("Magic number search failed. No magic number found.");

    0
}

fn create_magic_numbers(seed: u32) {
    let mut rng = Rng::new(seed);

    for (square, relevant_bits) in ROOK_RELEVANT_BITS.into_iter().enumerate() {
        let magic = find_magic_number(square, relevant_bits, SlidingPiece::Rook, &mut rng);
        println!("{:#x},", magic);
    }

    println!("\n");

    for (square, relevant_bits) in BISHOP_RELEVANT_BITS.into_iter().enumerate() {
        let magic = find_magic_number(square, relevant_bits, SlidingPiece::Bishop, &mut rng);
        println!("{:#x},", magic);
    }
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

// region: Bitboards

#[derive(Clone, Copy, Default)]
pub struct Bitboards {
    // [color][piece]
    pieces: [[u64; 6]; 2],
}

impl Bitboards {
    pub fn new() -> Bitboards {
        Bitboards {
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
        }
    }

    pub fn empty() -> Bitboards {
        Bitboards {
            pieces: [[0u64; 6]; 2],
        }
    }

    fn white(&self) -> u64 {
        self.pieces[Color::White as usize]
            .iter()
            .fold(0, |acc, bb| acc | bb)
    }

    fn black(&self) -> u64 {
        self.pieces[Color::Black as usize]
            .iter()
            .fold(0, |acc, bb| acc | bb)
    }

    fn all(&self) -> u64 {
        self.white() | self.black()
    }
}

// endregion

// region: Attack Tables

#[derive(Clone)]
pub struct SlidingTable {
    mask: [u64; 64],
    magic: [u64; 64],
    shift: [u8; 64],
    offset: [usize; 64],
    data: Vec<u64>,
}

#[derive(Clone)]
pub struct AttackTables {
    // 0 = black, 1 = white
    pawn_attacks: [[u64; 64]; 2],
    knight_attacks: [u64; 64],
    king_attacks: [u64; 64],

    bishop_table: SlidingTable,
    rook_table: SlidingTable,
}

impl AttackTables {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut pawn_attacks = [[0; 64]; 2];
        let mut knight_attacks = [0; 64];
        let mut king_attacks = [0u64; 64];

        for square in 0..64 {
            pawn_attacks[0][square] = Self::mask_pawn_attacks(0, square);
            pawn_attacks[1][square] = Self::mask_pawn_attacks(1, square);

            knight_attacks[square] = Self::mask_knight_attacks(square);
            king_attacks[square] = Self::mask_king_attacks(square);
        }

        let (bishop_table, rook_table) = Self::compute_sliding_attack_tables();

        AttackTables {
            pawn_attacks,
            knight_attacks,
            king_attacks,
            bishop_table,
            rook_table,
        }
    }

    pub fn mask_pawn_attacks(color: u8, square: usize) -> u64 {
        let mut attacks = 0;
        let bitboard = 1u64 << square;

        if color == 0 {
            add_attack!(attacks, bitboard << 7, NOT_H_FILE);
            add_attack!(attacks, bitboard << 9, NOT_A_FILE);
        } else {
            add_attack!(attacks, bitboard >> 9, NOT_H_FILE);
            add_attack!(attacks, bitboard >> 7, NOT_A_FILE);
        }

        attacks
    }

    pub fn mask_knight_attacks(square: usize) -> u64 {
        let mut attacks = 0u64;
        let bitboard = 1u64 << square;

        add_attack!(attacks, bitboard >> 17, NOT_H_FILE);
        add_attack!(attacks, bitboard >> 15, NOT_A_FILE);
        add_attack!(attacks, bitboard >> 10, NOT_HG_FILE);
        add_attack!(attacks, bitboard >> 6, NOT_AB_FILE);

        add_attack!(attacks, bitboard << 17, NOT_A_FILE);
        add_attack!(attacks, bitboard << 15, NOT_H_FILE);
        add_attack!(attacks, bitboard << 10, NOT_AB_FILE);
        add_attack!(attacks, bitboard << 6, NOT_HG_FILE);

        attacks
    }

    pub fn mask_king_attacks(square: usize) -> u64 {
        let mut attacks = 0u64;
        let bitboard = 1u64 << square;

        add_attack!(attacks, bitboard >> 8, !0);
        add_attack!(attacks, bitboard >> 9, NOT_H_FILE);
        add_attack!(attacks, bitboard >> 7, NOT_A_FILE);
        add_attack!(attacks, bitboard >> 1, NOT_H_FILE);
        add_attack!(attacks, bitboard << 8, !0);
        add_attack!(attacks, bitboard << 9, NOT_A_FILE);
        add_attack!(attacks, bitboard << 7, NOT_H_FILE);
        add_attack!(attacks, bitboard << 1, NOT_A_FILE);

        attacks
    }

    pub fn mask_bishop_attacks(square: usize) -> u64 {
        let mut attacks = 0u64;

        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;

        const DIRECTIONS: [(i32, i32); 4] = [(1, 1), (-1, 1), (1, -1), (-1, -1)];

        for (dr, df) in DIRECTIONS {
            let mut r = rank + dr;
            let mut f = file + df;

            while (1..=6).contains(&r) && (1..=6).contains(&f) {
                attacks |= 1u64 << (r * 8 + f);
                r += dr;
                f += df;
            }
        }

        attacks
    }

    pub fn mask_rook_attacks(square: usize) -> u64 {
        let mut attacks = 0u64;

        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;

        for r in (rank + 1)..=6 {
            attacks |= 1u64 << (r * 8 + file);
        }

        for r in (1..rank).rev() {
            attacks |= 1u64 << (r * 8 + file);
        }

        for f in (file + 1)..=6 {
            attacks |= 1u64 << (rank * 8 + f);
        }

        for f in (1..file).rev() {
            attacks |= 1u64 << (rank * 8 + f);
        }

        attacks
    }

    pub fn bishop_attacks_fly(square: usize, occupancy: u64) -> u64 {
        let mut attack_mask = 0u64;

        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;

        let directions = [(1, 1), (-1, 1), (1, -1), (-1, -1)];

        for &(dr, df) in &directions {
            let mut r = rank + dr;
            let mut f = file + df;

            while (0..=7).contains(&r) && (0..=7).contains(&f) {
                let target_square = (r * 8 + f) as u8;
                let target_mask = 1u64 << target_square;

                attack_mask |= target_mask;

                if target_mask & occupancy != 0 {
                    break;
                }

                r += dr;
                f += df;
            }
        }

        attack_mask
    }

    pub fn rook_attacks_fly(square: usize, occupancy: u64) -> u64 {
        let mut attack_mask = 0u64;

        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;

        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        for &(rank_delta, file_delta) in &directions {
            let mut r = rank + rank_delta;
            let mut f = file + file_delta;

            while (0..=7).contains(&r) && (0..=7).contains(&f) {
                let target_square = (r * 8 + f) as u8;
                let target_mask = 1u64 << target_square;

                attack_mask |= target_mask;

                if target_mask & occupancy != 0 {
                    break;
                }

                r += rank_delta;
                f += file_delta;
            }
        }

        attack_mask
    }

    pub fn set_occupancy(index: u32, bits_in_mask: u32, mut attack_mask: u64) -> u64 {
        let mut occupancy = 0;

        for count in 0..bits_in_mask {
            let square = attack_mask.trailing_zeros();

            attack_mask &= !(1 << (square));

            if index & (1 << count) != 0 {
                occupancy |= (1 << square);
            }
        }

        occupancy
    }

    pub fn compute_sliding_attack_tables() -> (SlidingTable, SlidingTable) {
        let mut bishop_masks = [0u64; 64];
        let mut bishop_magic = [0u64; 64];
        let mut bishop_shift = [0u8; 64];
        let mut bishop_offset = [0usize; 64];

        let mut rook_masks = [0u64; 64];
        let mut rook_magic = [0u64; 64];
        let mut rook_shift = [0u8; 64];
        let mut rook_offset = [0usize; 64];

        let total_bishop_entries: usize = BISHOP_RELEVANT_BITS.iter().map(|&bits| 1 << bits).sum();

        let total_rook_entries: usize = ROOK_RELEVANT_BITS.iter().map(|&bits| 1 << bits).sum();

        let mut bishop_data = Vec::with_capacity(total_bishop_entries);
        let mut rook_data = Vec::with_capacity(total_rook_entries);

        let mut bishop_current_offset = 0;
        let mut rook_current_offset = 0;

        for square in 0..64 {
            bishop_masks[square] = Self::mask_bishop_attacks(square);
            rook_masks[square] = Self::mask_rook_attacks(square);

            bishop_magic[square] = BISHOP_MAGIC_NUMBERS[square];
            bishop_shift[square] = (64 - BISHOP_RELEVANT_BITS[square]) as u8;

            rook_magic[square] = ROOK_MAGIC_NUMBERS[square];
            rook_shift[square] = (64 - ROOK_RELEVANT_BITS[square]) as u8;

            let bishop_bits = BISHOP_RELEVANT_BITS[square] as usize;
            let bishop_entries = 1 << bishop_bits;
            bishop_offset[square] = bishop_current_offset;
            bishop_current_offset += bishop_entries;
            bishop_data.resize(bishop_current_offset, 0);

            for index in 0..bishop_entries {
                let occupancy =
                    Self::set_occupancy(index as u32, bishop_bits as u32, bishop_masks[square]);
                let magic_index = ((occupancy.wrapping_mul(bishop_magic[square]))
                    >> bishop_shift[square]) as usize;
                bishop_data[bishop_offset[square] + magic_index] =
                    Self::bishop_attacks_fly(square, occupancy);
            }

            let rook_bits = ROOK_RELEVANT_BITS[square] as usize;
            let rook_entries = 1 << rook_bits;
            rook_offset[square] = rook_current_offset;
            rook_current_offset += rook_entries;
            rook_data.resize(rook_current_offset, 0);

            for index in 0..rook_entries {
                let occupancy =
                    Self::set_occupancy(index as u32, rook_bits as u32, rook_masks[square]);
                let magic_index =
                    ((occupancy.wrapping_mul(rook_magic[square])) >> rook_shift[square]) as usize;
                rook_data[rook_offset[square] + magic_index] =
                    Self::rook_attacks_fly(square, occupancy);
            }
        }

        let bishop_table = SlidingTable {
            mask: bishop_masks,
            magic: bishop_magic,
            shift: bishop_shift,
            offset: bishop_offset,
            data: bishop_data,
        };

        let rook_table = SlidingTable {
            mask: rook_masks,
            magic: rook_magic,
            shift: rook_shift,
            offset: rook_offset,
            data: rook_data,
        };

        (bishop_table, rook_table)
    }

    pub fn get_bishop_attacks(&self, square: usize, occupancy: u64) -> u64 {
        let occ = occupancy & self.bishop_table.mask[square];
        let index = ((occ.wrapping_mul(self.bishop_table.magic[square]))
            >> self.bishop_table.shift[square]) as usize;
        self.bishop_table.data[self.bishop_table.offset[square] + index]
    }

    pub fn get_rook_attacks(&self, square: usize, occupancy: u64) -> u64 {
        let occ = occupancy & self.rook_table.mask[square];
        let index = ((occ.wrapping_mul(self.rook_table.magic[square]))
            >> self.rook_table.shift[square]) as usize;
        self.rook_table.data[self.rook_table.offset[square] + index]
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
    pub captured: Option<Piece>,
    pub castling_rights: u8,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u32,
}

// endregion

// region: Position

#[derive(Clone)]
pub struct Position {
    pub bitboards: Bitboards,
    pub side_to_move: Color,
    pub castling_rights: u8, // 1 bit per thing
    pub en_passant: Option<u8>,
    pub halfmove_clock: u32,
    pub attack_tables: AttackTables,
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
            attack_tables: AttackTables::new(),
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
            attack_tables: AttackTables::new(),
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

        let fullmove = (self.halfmove_clock / 2) + 1;
        fen.push(' ');
        fen.push_str(&fullmove.to_string());

        fen
    }

    pub fn piece_on(&self, square: u8) -> Option<(Color, Piece)> {
        let mask = 1 << square;

        for color in [Color::White, Color::Black] {
            for piece in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
                Piece::King,
            ] {
                let board = self.bitboards.pieces[color as usize][piece as usize];

                if board & mask != 0 {
                    return Some((color, piece));
                }
            }
        }

        None
    }

    pub fn square_under_attack(&self, square: u8, by_color: Color) -> bool {
        let idx = by_color as usize;
        let square = square as usize;

        // rustfmt wtf?!?!?!?!?!??
        if (self.attack_tables.pawn_attacks[idx][square]
            & self.bitboards.pieces[idx][Piece::Pawn as usize])
            != 0
            || (self.attack_tables.knight_attacks[square]
                & self.bitboards.pieces[idx][Piece::Knight as usize])
                != 0
            || (self.attack_tables.king_attacks[square]
                & self.bitboards.pieces[idx][Piece::King as usize])
                != 0
        {
            return true;
        }

        // is rustfmt okay?
        let bishop_attacks = self
            .attack_tables
            .get_bishop_attacks(square, self.bitboards.all());
        if bishop_attacks
            & (self.bitboards.pieces[idx][Piece::Bishop as usize]
                | self.bitboards.pieces[idx][Piece::Queen as usize])
            != 0
        {
            return true;
        }

        // is everything okay at home rustfmt??
        let rook_attacks = self
            .attack_tables
            .get_rook_attacks(square, self.bitboards.all());
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
        let undo = Undo {
            captured: self.piece_on(mv.to).map(|(_, piece)| piece),
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
        };

        if matches!(
            mv.flag,
            MoveFlag::Capture | MoveFlag::PromotionCapture | MoveFlag::Promotion
        ) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if mv.flag == MoveFlag::EnPassant {
            let cap_sq = if self.side_to_move == Color::White {
                mv.to - 8
            } else {
                mv.to + 8
            };
            let color_idx = 1 - self.side_to_move as usize;
            let pawn_idx = Piece::Pawn as usize;
            self.bitboards.pieces[color_idx][pawn_idx] &= !(1u64 << cap_sq);
        }

        let color_idx = self.side_to_move as usize;
        let piece_idx = self.piece_on(mv.from).unwrap().1 as usize;

        match mv.flag {
            MoveFlag::Promotion | MoveFlag::PromotionCapture => {
                let promo_idx = mv.promotion.unwrap() as usize;
                self.bitboards.pieces[color_idx][promo_idx] |= 1u64 << mv.to;
                self.bitboards.pieces[color_idx][Piece::Pawn as usize] &= !(1u64 << mv.from);
            }
            _ => {
                self.bitboards.pieces[color_idx][piece_idx] |= 1u64 << mv.to;
                self.bitboards.pieces[color_idx][piece_idx] &= !(1u64 << mv.from);
            }
        }

        if let Some(captured) = undo.captured {
            let captured_color = 1 - color_idx;
            self.bitboards.pieces[captured_color][captured as usize] &= !(1u64 << mv.to);
        }

        match mv.flag {
            MoveFlag::KingCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (7, 5)
                } else {
                    (63, 61)
                };
                let rook_idx = Piece::Rook as usize;
                self.bitboards.pieces[color_idx][rook_idx] &= !(1u64 << rook_from);
                self.bitboards.pieces[color_idx][rook_idx] |= 1u64 << rook_to;
            }
            MoveFlag::QueenCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (0, 3)
                } else {
                    (56, 59)
                };
                let rook_idx = Piece::Rook as usize;
                self.bitboards.pieces[color_idx][rook_idx] &= !(1u64 << rook_from);
                self.bitboards.pieces[color_idx][rook_idx] |= 1u64 << rook_to;
            }
            _ => {}
        }

        match self.side_to_move {
            Color::White => {
                if piece_idx == Piece::King as usize {
                    self.castling_rights &= !0b11;
                }
                if piece_idx == Piece::Rook as usize {
                    if mv.from == 0 {
                        self.castling_rights &= !0b10;
                    }
                    if mv.from == 7 {
                        self.castling_rights &= !0b01;
                    }
                }
            }
            Color::Black => {
                if piece_idx == Piece::King as usize {
                    self.castling_rights &= !0b1100;
                }
                if piece_idx == Piece::Rook as usize {
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

        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        undo
    }

    pub fn undo_move(&mut self, mv: Move, undo: Undo) {
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        let color_idx = self.side_to_move as usize;

        match mv.flag {
            MoveFlag::Promotion | MoveFlag::PromotionCapture => {
                let promo_idx = mv.promotion.unwrap() as usize;
                self.bitboards.pieces[color_idx][Piece::Pawn as usize] |= 1u64 << mv.from;
                self.bitboards.pieces[color_idx][promo_idx] &= !(1u64 << mv.to);
            }
            _ => {
                let piece_idx = self.piece_on(mv.to).unwrap().1 as usize;
                self.bitboards.pieces[color_idx][piece_idx] |= 1u64 << mv.from;
                self.bitboards.pieces[color_idx][piece_idx] &= !(1u64 << mv.to);
            }
        }

        if let Some(captured) = undo.captured {
            let captured_color = 1 - color_idx;
            self.bitboards.pieces[captured_color][captured as usize] |= 1u64 << mv.to;
        }

        if mv.flag == MoveFlag::EnPassant {
            let cap_sq = if self.side_to_move == Color::White {
                mv.to - 8
            } else {
                mv.to + 8
            };
            let captured_color = 1 - color_idx;
            self.bitboards.pieces[captured_color][Piece::Pawn as usize] |= 1u64 << cap_sq;
        }

        match mv.flag {
            MoveFlag::KingCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (7, 5)
                } else {
                    (63, 61)
                };
                let rook_idx = Piece::Rook as usize;
                self.bitboards.pieces[color_idx][rook_idx] |= 1u64 << rook_from;
                self.bitboards.pieces[color_idx][rook_idx] &= !(1u64 << rook_to);
            }
            MoveFlag::QueenCastle => {
                let (rook_from, rook_to) = if self.side_to_move == Color::White {
                    (0, 3)
                } else {
                    (56, 59)
                };
                let rook_idx = Piece::Rook as usize;
                self.bitboards.pieces[color_idx][rook_idx] |= 1u64 << rook_from;
                self.bitboards.pieces[color_idx][rook_idx] &= !(1u64 << rook_to);
            }
            _ => {}
        }

        self.castling_rights = undo.castling_rights;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
    }

    pub fn get_pawn_moves(&self, moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        let idx = color as usize;
        let enemy_idx = color.opposite() as usize;

        let our_pawns = self.bitboards.pieces[idx][Piece::Pawn as usize];
        let enemy_occupancy = if color == Color::White {
            self.bitboards.black()
        } else {
            self.bitboards.white()
        };
        let all_occupancy = self.bitboards.all();
        let empty = !all_occupancy;

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
            let attacks = self.attack_tables.pawn_attacks[pawn_idx][from as usize];
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
                let attacks = self.attack_tables.pawn_attacks[pawn_idx][from as usize];
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

    pub fn get_castle_moves(&self, moves: &mut Vec<Move>) {
        let rank = if self.side_to_move == Color::White {
            0
        } else {
            7
        };
        let enemy_color = self.side_to_move.opposite();

        if self.is_in_check(self.side_to_move) {
            return;
        }

        let king_side_mask = if self.side_to_move == Color::White {
            0b0001
        } else {
            0b0100
        };
        if self.castling_rights & king_side_mask != 0 {
            let f = rank * 8 + 5;
            let g = rank * 8 + 6;
            let e = rank * 8 + 4;

            if self.piece_on(f as u8).is_none()
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
            let e = rank * 8 + 4;

            if self.piece_on(d as u8).is_none()
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

    pub fn get_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let color = self.side_to_move;
        let idx = color as usize;
        let enemy_idx = color.opposite() as usize;

        let all_occupancy = self.bitboards.all();
        let our_pieces = self.bitboards.pieces[idx]
            .iter()
            .fold(0, |acc, &board| acc | board);
        let enemy_occupancy = self.bitboards.pieces[enemy_idx]
            .iter()
            .fold(0, |acc, &board| acc | board);

        use Piece::*;

        self.get_pawn_moves(&mut moves);

        let knights = self.bitboards.pieces[idx][Knight as usize];
        for from in BitIter::new(knights) {
            let attacks = self.attack_tables.knight_attacks[from as usize] & !our_pieces;
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
            let attacks = self
                .attack_tables
                .get_bishop_attacks(from as usize, all_occupancy)
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

        let rooks = self.bitboards.pieces[idx][Rook as usize];
        for from in BitIter::new(rooks) {
            let attacks = self
                .attack_tables
                .get_rook_attacks(from as usize, all_occupancy)
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

        let queens = self.bitboards.pieces[idx][Queen as usize];
        for from in BitIter::new(queens) {
            let attacks = (self
                .attack_tables
                .get_bishop_attacks(from as usize, all_occupancy)
                | self
                    .attack_tables
                    .get_rook_attacks(from as usize, all_occupancy))
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
            let attacks = self.attack_tables.king_attacks[from as usize] & !our_pieces;
            for to in BitIter::new(attacks) {
                moves.push(Move {
                    from,
                    to,
                    promotion: None,
                    flag: MoveFlag::Quiet,
                });
            }
        }

        self.get_castle_moves(&mut moves);

        moves
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        self.get_pseudo_legal_moves()
    }
}

// endregion

// region: GameState

// endregion
