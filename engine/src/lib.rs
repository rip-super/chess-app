#![allow(unused)]

macro_rules! add_attack {
    ($attacks:ident, $shifted:expr, $mask:expr) => {
        if $shifted & $mask != 0 {
            $attacks |= $shifted;
        }
    };
}

const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;
const NOT_HG_FILE: u64 = 0x3F3F3F3F3F3F3F3F;
const NOT_AB_FILE: u64 = 0xFCFCFCFCFCFCFCFC;

pub struct Square;
impl Square {
    pub const A8: usize = 0;
    pub const B8: usize = 1;
    pub const C8: usize = 2;
    pub const D8: usize = 3;
    pub const E8: usize = 4;
    pub const F8: usize = 5;
    pub const G8: usize = 6;
    pub const H8: usize = 7;

    pub const A7: usize = 8;
    pub const B7: usize = 9;
    pub const C7: usize = 10;
    pub const D7: usize = 11;
    pub const E7: usize = 12;
    pub const F7: usize = 13;
    pub const G7: usize = 14;
    pub const H7: usize = 15;

    pub const A6: usize = 16;
    pub const B6: usize = 17;
    pub const C6: usize = 18;
    pub const D6: usize = 19;
    pub const E6: usize = 20;
    pub const F6: usize = 21;
    pub const G6: usize = 22;
    pub const H6: usize = 23;

    pub const A5: usize = 24;
    pub const B5: usize = 25;
    pub const C5: usize = 26;
    pub const D5: usize = 27;
    pub const E5: usize = 28;
    pub const F5: usize = 29;
    pub const G5: usize = 30;
    pub const H5: usize = 31;

    pub const A4: usize = 32;
    pub const B4: usize = 33;
    pub const C4: usize = 34;
    pub const D4: usize = 35;
    pub const E4: usize = 36;
    pub const F4: usize = 37;
    pub const G4: usize = 38;
    pub const H4: usize = 39;

    pub const A3: usize = 40;
    pub const B3: usize = 41;
    pub const C3: usize = 42;
    pub const D3: usize = 43;
    pub const E3: usize = 44;
    pub const F3: usize = 45;
    pub const G3: usize = 46;
    pub const H3: usize = 47;

    pub const A2: usize = 48;
    pub const B2: usize = 49;
    pub const C2: usize = 50;
    pub const D2: usize = 51;
    pub const E2: usize = 52;
    pub const F2: usize = 53;
    pub const G2: usize = 54;
    pub const H2: usize = 55;

    pub const A1: usize = 56;
    pub const B1: usize = 57;
    pub const C1: usize = 58;
    pub const D1: usize = 59;
    pub const E1: usize = 60;
    pub const F1: usize = 61;
    pub const G1: usize = 62;
    pub const H1: usize = 63;
}

pub fn get_bit(board: u64, square: usize) -> usize {
    ((board >> square) & 1) as usize
}
pub fn set_bit(board: &mut u64, square: usize) {
    *board |= 1 << square
}
pub fn pop_bit(board: &mut u64, square: usize) {
    *board &= !(1 << square)
}

pub struct AttackTables {
    // 0 = black, 1 = white
    pawn_attacks: [[u64; 64]; 2],
    knight_attacks: [u64; 64],
    king_attacks: [u64; 64],
}

impl AttackTables {
    fn new() -> Self {
        let mut pawn_attacks = [[0; 64]; 2];
        let mut knight_attacks = [0; 64];
        let mut king_attacks = [0u64; 64];

        for square in 0..64 {
            pawn_attacks[0][square] = Self::mask_pawn_attacks(0, square);
            pawn_attacks[1][square] = Self::mask_pawn_attacks(1, square);

            knight_attacks[square] = Self::mask_knight_attacks(square);
            king_attacks[square] = Self::mask_king_attacks(square);
        }

        AttackTables {
            pawn_attacks,
            knight_attacks,
            king_attacks,
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
}
