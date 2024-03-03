use crate::helper::*;
// max number of relevant occupancy bits

pub const BISHOP_RELEVANT_BITS: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

pub const ROOK_RELEVANT_BITS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

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

pub const fn bishop_rays(square: usize) -> u64 {
    //separate function needed for hashing
    //does not include squares on the edge of the board as
    //bishop cannot go past these anyway
    //blockers are treated as pieces of opposite colour (can be captured)
    let mut attacks: u64 = 0;
    let square_rank: usize = square / 8;
    let square_file: usize = square % 8;
    let mut rank: usize = square_rank;
    let mut file: usize = square_file;
    while rank > 1 && file > 1 {
        rank -= 1;
        file -= 1; //file and rank decreasing
        attacks = set_bit(rank * 8 + file, attacks);
    }

    rank = square_rank;
    file = square_file;
    while rank < 6 && file > 1 {
        rank += 1;
        file -= 1; // rank increasing, file decreasing
        attacks = set_bit(rank * 8 + file, attacks);
    }

    rank = square_rank;
    file = square_file;
    while rank > 1 && file < 6 {
        rank -= 1;
        file += 1; //rank increasing, file decreasing
        attacks = set_bit(rank * 8 + file, attacks);
    }

    rank = square_rank;
    file = square_file;
    while rank < 6 && file < 6 {
        rank += 1;
        file += 1; //both increasing
        attacks = set_bit(rank * 8 + file, attacks);
    }
    attacks
}

pub const fn mask_bishop_attacks(square: usize, blockers: u64) -> u64 {
    //does not include squares on the edge of the board as
    //bishop cannot go past these anyway
    //blockers are treated as pieces of opposite colour (can be captured)
    let mut attacks: u64 = 0;
    let square_rank: usize = square / 8;
    let square_file: usize = square % 8;
    let mut rank: usize = square_rank;
    let mut file: usize = square_file;
    while rank > 1 && file > 1 {
        rank -= 1;
        file -= 1; //file and rank decreasing
        attacks = set_bit(rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }

    rank = square_rank;
    file = square_file;
    while rank < 6 && file > 1 {
        rank += 1;
        file -= 1; // rank increasing, file decreasing
        attacks = set_bit(rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }

    rank = square_rank;
    file = square_file;
    while rank > 1 && file < 6 {
        rank -= 1;
        file += 1; //rank increasing, file decreasing
        attacks = set_bit(rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }

    rank = square_rank;
    file = square_file;
    while rank < 6 && file < 6 {
        rank += 1;
        file += 1; //both increasing
        attacks = set_bit(rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }
    attacks
}

pub const fn rook_rays(square: usize) -> u64 {
    //separate function for hashing
    let mut attacks: u64 = 0;
    let square_rank: usize = square / 8;
    let square_file: usize = square % 8;
    let mut rank = square_rank;
    let mut file = square_file;
    while rank < 6 {
        rank += 1;
        attacks = set_bit(rank * 8 + square_file, attacks);
    }
    rank = square_rank;
    while rank > 1 {
        rank -= 1;
        attacks = set_bit(rank * 8 + square_file, attacks);
    }
    while file < 6 {
        file += 1;
        attacks = set_bit(square_rank * 8 + file, attacks);
    }
    file = square_file;
    while file > 1 {
        file -= 1;
        attacks = set_bit(square_rank * 8 + file, attacks);
    }
    attacks
}

pub const fn mask_rook_attacks(square: usize, blockers: u64) -> u64 {
    let mut attacks: u64 = 0;
    let square_rank: usize = square / 8;
    let square_file: usize = square % 8;
    let mut rank = square_rank;
    let mut file = square_file;
    while rank < 6 {
        rank += 1;
        attacks = set_bit(rank * 8 + square_file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }
    rank = square_rank;
    while rank > 1 {
        rank -= 1;
        attacks = set_bit(rank * 8 + square_file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }
    while file < 6 {
        file += 1;
        attacks = set_bit(square_rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }
    file = square_file;
    while file > 1 {
        file -= 1;
        attacks = set_bit(square_rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }
    attacks
}

pub const fn mask_queen_attacks(square: usize, blockers: u64) -> u64 {
    mask_bishop_attacks(square, blockers) | mask_rook_attacks(square, blockers)
}

pub const fn set_occupancy(index: usize, bits_in_mask: usize, m: u64) -> u64 {
    //essentially count to 2^bits_in_mask - 1 in binary by toggling
    //the bits in the attack mask
    let mut occupancy: u64 = 0;
    let mut i: usize = 0;
    let mut mask = m;
    while i < bits_in_mask {
        let b = match lsfb(mask) {
            Some(k) => k,
            None => break, //no more bits in mask
        };
        mask = pop_bit(b, mask);
        if index & (1 << i) > 0 {
            occupancy |= 1 << b;
        }
        i += 1;
    }
    occupancy
}

const STATE: u32 = 893291824;
pub fn rng() -> u32 {
    let mut n = STATE;
    n ^= n << 13;
    n ^= n >> 17;
    n ^= n << 5;
    n
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