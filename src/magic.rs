use crate::helper::*;
use crate::rng::*;
// max number of relevant occupancy bits

pub enum SliderType {
    Bishop,
    Rook,
}

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

pub fn gen_magic(square: usize, relevant_bits: usize, slider: SliderType) -> u64 {
    let (mut occupancies, mut attacks) = ([0u64; 4096], [0u64; 4096]);
    let attack_mask = match slider {
        SliderType::Bishop => bishop_rays(square),
        SliderType::Rook => rook_rays(square),
    };
    let blocker_combinations = 1 << relevant_bits;
    // max number of blocker combinations = 2 ^ k
    for i in 0..blocker_combinations {
        occupancies[i] = set_occupancy(i, relevant_bits, attack_mask);
        attacks[i] = match slider {
            SliderType::Bishop => mask_bishop_attacks(square, occupancies[i]),
            SliderType::Rook => mask_rook_attacks(square, occupancies[i]),
        };
    }

    for _ in 0..10_000_000 {
        let mut used_attacks = [0u64; 4096]; //reset used_attacks
        let magic = magic_candidate();
        if count((magic * attack_mask) & 0xFF_00_00_00_00_00_00_00) < 6 {
            //test swapping this to heuristic 6
            //impossible for magic to work
            continue;
        }
        let mut ok: bool = true;
        for i in 0..blocker_combinations {
            let magic_index: usize = ((occupancies[i] * magic) >> (64 - relevant_bits))
                .try_into()
                .unwrap();
            //test magic number multiplication
            if used_attacks[magic_index] == 0 {
                used_attacks[magic_index] = attacks[i]
            } else {
                //magic doesn't work
                ok = false;
                break;
            }
        }
        if ok {
            return magic;
        }
    }
    eprintln!("failed to generate magic for {}", coordinate(square));
    0
}

pub fn init_magics() {
    println!("BISHOP MAGICS:");
    println!("==============\n");
    for (i, bits) in BISHOP_RELEVANT_BITS.iter().enumerate() {
        println!("{},", gen_magic(i, *bits, SliderType::Bishop))
    }

    print!("\n\n\n");
    println!("ROOK MAGICS:");
    println!("==============\n");

    for (i, bits) in ROOK_RELEVANT_BITS.iter().enumerate() {
        println!("{},", gen_magic(i, *bits, SliderType::Rook))
    }
}

pub const BISHOP_MAGICS: [u64; 64] = [
    865289279978471456,
    5188859263535153152,
    1267745530323008,    
    298367875691839488,  
    13839711328349978628,
    36593465093720320,
    2344207186181309512,
    2326577478631940,
    76726190541440000,
    10206225476356225,
    20275028792714240,
    572040302055424,
    108088598943369728,
    5188147875075325957,
    424413904507394,
    18021004177835012,
    4644923648262272,
    9243638381475344896,
    312155921075732996,
    4614008204232230992,
    1691066633882369,
    70927095308292,
    148055842066792960,
    4786244991590784,
    2603987668959754,
    2310351082997941384,
    4612813125254119680,
    577028101915021344,
    1225260646652723200,
    144191054948893192,
    36332270902837504,
    18300409005245440,
    290519593860145152,
    299105864651082,
    40166535200808,
    72101608862910977,
    2254016019435776,
    146725465192138752,
    147501830337742080,
    9367647762237128832,
    73201094854971396,
    9223451279287207936,
    2495016192402524160,
    86234957826,
    4647723680528695552,
    9296581953573093440,
    10282652271771904,
    288218633273856,
    2306424656771418121,
    9367529040771557377,
    81065001666740866,
    288230411618877440,
    5188182230117253121,
    36882155615326208,
    1158023522304524288,
    2269512434987040,
    10412434497291894809,
    550963941376,
    36180582270554128,
    4775856298611206144,
    1134700431212800,
    2377903635639107840,
    4899926084042228736,
    578836831324357120
];

pub const ROOK_MAGICS: [u64; 64] = [
    900720200370152064,
    4629700554376872002,
    2738197927901012240,
    1297041159585136704,
    36037595259731970,
    5908872588355961344,
    36029346808332544,
    72069140017393920,
    1153484596298514562,
    1189020945516339200,
    4785212059829264,
    1153203048319819784,
    2306124518600475664,
    1729945241291588096,
    28148064624576514,
    9404078973010575489,
    72568321212545,
    63050669929545766,
    9011872447612928,
    360853119304732708,
    433754038690381840,
    4613094492956918784,
    4398063821360,
    90355666555782177,
    36099167910658082,
    9552425634775449600,
    9042387922862336,
    9369739585238597896,
    2323866205963944065,
    864693329634264064,
    577032515529871361,
    1733903732192084993,
    581105364573880353,
    283676181012544,
    39127495949164550,
    722829389603016708,
    9248995062279308288,
    5189273220468965888,
    9511620559310424066,
    576601491410784512,
    5800636595467993088,
    657526113606074400,
    6931041480622768144,
    2594647331510026272,
    10531672161028997136,
    2341874005323055232,
    13916685805042139176,
    576479445083881476,
    1153062243168952448,
    18155518283489792,
    360323156445168384,
    9288742951461120,
    4796755849983033984,
    72620612845306368,
    45045350980977664,
    36100282458867200,
    9223408321011154953,
    16456293951998005762,
    1155041432281612353,
    13510868341948569,
    11790986792126681218,
    594756642969618433,
    180152852324091396,
    10570091814458818882
];

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
