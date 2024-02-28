use crate::helper::*;

pub const fn mask_pawn_attacks(square: usize, side: SideToMove) -> u64 {
    //generate capturing attacks
    let p: u64 = set_bit(square, 0);
    match side {
        SideToMove::White => ((p >> 9) & !A_FILE) | ((p >> 7) & !H_FILE),
        SideToMove::Black => ((p << 7) & !A_FILE) | ((p << 9) & !H_FILE),
    }
}

pub const fn mask_knight_attacks(square: usize) -> u64 {
    let n: u64 = set_bit(square, 0);
    ((n >> 17) & !A_FILE)
        | ((n >> 15) & !H_FILE)
        | ((n >> 10) & !(A_FILE | B_FILE))
        | ((n >> 6) & !(G_FILE | H_FILE))
        | ((n << 6) & !(A_FILE | B_FILE))
        | ((n << 10) & !(G_FILE | H_FILE))
        | ((n << 15) & !A_FILE)
        | ((n << 17) & !H_FILE)
}

pub const fn mask_king_attacks(square: usize) -> u64 {
    let k: u64 = set_bit(square, 0);
    ((k >> 9) & !A_FILE)
        | ((k >> 7) & !H_FILE)
        | ((k << 7) & !A_FILE)
        | ((k << 9) & !H_FILE)
        | ((k >> 1) & !A_FILE)
        | ((k << 1) & !H_FILE)
        | (k << 8)
        | (k >> 8)
}

pub const WP_ATTACKS: [u64; 64] = {
    let mut i: usize = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = mask_pawn_attacks(i, SideToMove::White);
        i += 1;
    }
    table
};

pub const BP_ATTACKS: [u64; 64] = {
    let mut i: usize = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = mask_pawn_attacks(i, SideToMove::Black);
        i += 1;
    }
    table
};

pub const N_ATTACKS: [u64; 64] = {
    let mut i: usize = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = mask_knight_attacks(i);
        i += 1;
    }
    table
};

pub const K_ATTACKS: [u64; 64] = {
    let mut i: usize = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = mask_king_attacks(i);
        i += 1;
    }
    table
};
