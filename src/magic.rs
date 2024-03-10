use crate::board::*;
use crate::helper::*;
use crate::rng::*;
// max number of relevant blocker bits

pub enum SliderType {
    Bishop,
    Rook,
}

// bits attacked for each square
#[rustfmt::skip]
pub const BISHOP_RELEVANT_BITS: [usize; 64] = [
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
pub const ROOK_RELEVANT_BITS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

//functions with attack maps for non-sliders
pub const fn mask_pawn_attacks(square: usize, side: Colour) -> u64 {
    //generate capturing attacks
    let p: u64 = set_bit(square, 0);
    match side {
        Colour::Black => ((p >> 9) & !A_FILE) | ((p >> 7) & !H_FILE),
        Colour::White => ((p << 7) & !A_FILE) | ((p << 9) & !H_FILE),
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
    while rank > 0 && file > 0 {
        rank -= 1;
        file -= 1; //file and rank decreasing
        attacks = set_bit(rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }

    rank = square_rank;
    file = square_file;
    while rank < 7 && file > 0 {
        rank += 1;
        file -= 1; // rank increasing, file decreasing
        attacks = set_bit(rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }

    rank = square_rank;
    file = square_file;
    while rank > 0 && file < 7 {
        rank -= 1;
        file += 1; //rank increasing, file decreasing
        attacks = set_bit(rank * 8 + file, attacks);
        if (set_bit(rank * 8 + file, 0) & blockers) > 0 {
            break;
        }
    }

    rank = square_rank;
    file = square_file;
    while rank < 7 && file < 7 {
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
    //rook attacks without blockers
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
    //rook attacks accounting for blockers
    let mut attacks: u64 = 0;
    let square_rank: usize = square / 8;
    let square_file: usize = square % 8;
    let mut rank = square_rank;
    let mut file = square_file;
    while rank < 7 {
        rank += 1;
        attacks = set_bit(rank * 8 + square_file, attacks);
        if get_bit(rank * 8 + square_file, blockers) > 0 {
            break;
        }
    }
    rank = square_rank;
    while rank > 0 {
        rank -= 1;
        attacks = set_bit(rank * 8 + square_file, attacks);
        if get_bit(rank * 8 + square_file, blockers) > 0 {
            break;
        }
    }
    while file < 7 {
        file += 1;
        attacks = set_bit(square_rank * 8 + file, attacks);
        if get_bit(square_rank * 8 + file, blockers) > 0 {
            break;
        }
    }
    file = square_file;
    while file > 0 {
        file -= 1;
        attacks = set_bit(square_rank * 8 + file, attacks);
        if get_bit(square_rank * 8 + file, blockers) > 0 {
            break;
        }
    }
    attacks
}

pub const fn mask_queen_attacks(square: usize, blockers: u64) -> u64 {
    mask_bishop_attacks(square, blockers) | mask_rook_attacks(square, blockers)
}

pub const fn set_blockers(index: usize, bits_in_mask: usize, m: u64) -> u64 {
    //essentially count to 2^bits_in_mask - 1 in binary by toggling
    //the bits in the attack mask
    let mut blocker: u64 = 0;
    let mut i: usize = 0;
    let mut mask = m;
    while i < bits_in_mask {
        let b = match lsfb(mask) {
            Some(k) => k,
            None => break, //no more bits in mask
        };
        mask = pop_bit(b, mask);
        if index & (1 << i) > 0 {
            blocker |= 1 << b;
        }
        i += 1;
    }
    blocker
}

//generate magic for given square
pub fn gen_magic(square: usize, relevant_bits: usize, slider: SliderType) -> u64 {
    let (mut blockers, mut attacks) = ([0u64; 4096], [0u64; 4096]);
    let attack_mask = match slider {
        SliderType::Bishop => bishop_rays(square),
        SliderType::Rook => rook_rays(square),
    };
    let blocker_combinations = 1 << relevant_bits;
    // max number of blocker combinations = 2 ^ k
    for i in 0..blocker_combinations {
        blockers[i] = set_blockers(i, relevant_bits, attack_mask);
        attacks[i] = match slider {
            SliderType::Bishop => mask_bishop_attacks(square, blockers[i]),
            SliderType::Rook => mask_rook_attacks(square, blockers[i]),
        };
    }

    for _ in 0..10_000_000 {
        let mut used_attacks = [0u64; 4096]; //reset used_attacks
        let magic = magic_candidate();
        if count((magic * attack_mask) & 0xFF_00_00_00_00_00_00_00) < 6 {
            //heuristic suggesting to skip this candidate
            continue;
        }
        let mut ok: bool = true;
        for i in 0..blocker_combinations {
            let magic_index: usize = ((blockers[i] * magic) >> (64 - relevant_bits))
                .try_into()
                .unwrap(); //shift ensures it is between 0 and 4095
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
        println!("0x{:X}u64,", gen_magic(i, *bits, SliderType::Bishop))
    }

    print!("\n\n\n");
    println!("ROOK MAGICS:");
    println!("==============\n");

    for (i, bits) in ROOK_RELEVANT_BITS.iter().enumerate() {
        println!("0x{:X}u64,", gen_magic(i, *bits, SliderType::Rook))
    }
}

// magic constants copy-pasted from output of init_magics()
pub const BISHOP_MAGICS: [u64; 64] = [
    0x4485204022A00u64,
    0x4012244010002u64,
    0x208880450800004u64,
    0x20A0A02900084u64,
    0x8004504000008010u64,
    0x101042004A00000u64,
    0x1008210420020u64,
    0x8020A0901011000u64,
    0x50407104008880u64,
    0x200440408004104u64,
    0x4122802002000u64,
    0x82040400040u64,
    0x900020210000809u64,
    0x100208220200001u64,
    0x140020090090800u64,
    0x9040950401040200u64,
    0x1C0001012B20C40u64,
    0x830408711022004Au64,
    0x10000482208102u64,
    0x408201044004003u64,
    0x8D020090400740u64,
    0x8086001501010100u64,
    0x412000121102241u64,
    0x2010002008084A0u64,
    0x8048200088021010u64,
    0x601080020A21400u64,
    0x440410010040080u64,
    0x2002028008008062u64,
    0xA0840120802000u64,
    0x80460431008202u64,
    0x108008148420844u64,
    0x1216002020820Au64,
    0xA010880440200401u64,
    0x424C43002441000u64,
    0x100280804040024u64,
    0x1021200800130811u64,
    0xE160008480040021u64,
    0x110100081004040u64,
    0x41880500220500u64,
    0xA004012048802400u64,
    0x1040360004002u64,
    0x101081A03001004u64,
    0x980420041101001u64,
    0x2C184010402209u64,
    0xC040400101011214u64,
    0x781100900400A00u64,
    0xC528022C000240u64,
    0x4080060424100u64,
    0xE0404A0814404081u64,
    0x200840402020025u64,
    0x1809820106C81213u64,
    0x110020008404200Cu64,
    0xB800400D04400Au64,
    0x400612012008100u64,
    0x8110302118048802u64,
    0x4848411800A10000u64,
    0x4440442208024082u64,
    0x20820221211800u64,
    0x40400824220800u64,
    0x244400400208800u64,
    0x9042000890020210u64,
    0x824308A100300u64,
    0x8082088209D00u64,
    0x2C0210202004308u64,
];

pub const ROOK_MAGICS: [u64; 64] = [
    0x480022016804004u64,
    0x8040001000200040u64,
    0x100100900402000u64,
    0x100042008100100u64,
    0x180080080F40022u64,
    0x1A00301102000448u64,
    0x2100010000820004u64,
    0x100020220458100u64,
    0x8011802080004000u64,
    0x400402010004000u64,
    0x8400801000200080u64,
    0x1000808010000800u64,
    0x2000800800800400u64,
    0x282800400800201u64,
    0x1209000100040200u64,
    0x122000120840042u64,
    0x100208000401080u64,
    0x5160010100400080u64,
    0x888020001000u64,
    0x20828008001000u64,
    0x4008008008004u64,
    0x2400808004000200u64,
    0x40040001080210u64,
    0x4C000E0020804405u64,
    0x12400280228000u64,
    0x4000910100204002u64,
    0x4000100080200080u64,
    0x68001010010200u64,
    0x9048008180080400u64,
    0x20E2000200100408u64,
    0x120810C00504228u64,
    0x901C0200009541u64,
    0x440002C800480u64,
    0x20002040401008u64,
    0x100820046002010u64,
    0x100080800800u64,
    0xC40A080080800400u64,
    0x2382001004040020u64,
    0x5A02000802000401u64,
    0x2800404402000081u64,
    0x844000CB28828000u64,
    0x8002010080220044u64,
    0x1840420084120020u64,
    0x1000A0010420020u64,
    0x4000040008008080u64,
    0x4048040002008080u64,
    0x40224108040090u64,
    0x4809000088410022u64,
    0x200410024800300u64,
    0x20102040088080u64,
    0x2A8200480100880u64,
    0x2410020800128080u64,
    0x3080040080080080u64,
    0xE801042010400801u64,
    0x5002004801248200u64,
    0x40802F0008C080u64,
    0x1010401080042103u64,
    0x904608210240D082u64,
    0xC000084100102001u64,
    0x84200410000901u64,
    0x202000410200846u64,
    0x8083003608040029u64,
    0x92810108204u64,
    0x50003100440082u64,
];

pub const WP_ATTACKS: [u64; 64] = {
    let mut i: usize = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = mask_pawn_attacks(i, Colour::White);
        i += 1;
    }
    table
};

pub const BP_ATTACKS: [u64; 64] = {
    let mut i: usize = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = mask_pawn_attacks(i, Colour::Black);
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

pub static mut BISHOP_RAYS: [u64; 64] = [0u64; 64];
pub static mut ROOK_RAYS: [u64; 64] = [0u64; 64];

pub static mut BISHOP_ATTACKS: [[u64; 512]; 64] = [[0u64; 512]; 64];
pub static mut ROOK_ATTACKS: [[u64; 4096]; 64] = [[0u64; 4096]; 64];

// init() functions calculate BISHOP_RAYS, BISHOP_ATTACKS, ROOK_RAYS, ROOK_ATTACKS
pub fn init_bishop_attacks() {
    for square in 0..64 {
        let relevant_bits: usize = unsafe {
            BISHOP_RAYS[square] = bishop_rays(square);
            count(BISHOP_RAYS[square])
        };
        let blocker_indices = 1 << relevant_bits;
        for i in 0..blocker_indices {
            let blockers = unsafe { set_blockers(i, relevant_bits, BISHOP_RAYS[square]) };
            let magic_index =
                (blockers * BISHOP_MAGICS[square]) >> (64 - BISHOP_RELEVANT_BITS[square]);
            unsafe {
                BISHOP_ATTACKS[square][magic_index as usize] = mask_bishop_attacks(square, blockers);
            };
        }
    }
}

pub fn init_rook_attacks() {
    for square in 0..64 {
        let relevant_bits: usize = unsafe {
            ROOK_RAYS[square] = rook_rays(square);
            count(ROOK_RAYS[square])
        };
        let blocker_indices = 1 << relevant_bits;
        for i in 0..blocker_indices {
            let blockers = unsafe { set_blockers(i, relevant_bits, ROOK_RAYS[square]) };
            let magic_index = (blockers * ROOK_MAGICS[square]) >> (64 - ROOK_RELEVANT_BITS[square]);
            unsafe {
                ROOK_ATTACKS[square][magic_index as usize] = mask_rook_attacks(square, blockers);
            };
        }
    }
}

pub fn init_slider_attacks() {
    init_bishop_attacks();
    init_rook_attacks();
}

pub fn get_bishop_attacks(square: usize, blockers: u64) -> u64 {
    let mut b: u64 = blockers;
    unsafe {
        b &= BISHOP_RAYS[square]; //bits where blockers block rays
        b *= BISHOP_MAGICS[square]; //magic hashing
        b >>= 64 - BISHOP_RELEVANT_BITS[square];
    };
    unsafe { BISHOP_ATTACKS[square][b as usize] }
}

pub fn get_rook_attacks(square: usize, blockers: u64) -> u64 {
    let mut b: u64 = blockers;
    unsafe {
        b &= ROOK_RAYS[square];
        b *= ROOK_MAGICS[square];
        b >>= 64 - ROOK_RELEVANT_BITS[square];
    };
    unsafe { ROOK_ATTACKS[square][b as usize] }
}

pub fn get_queen_attacks(square: usize, blockers: u64) -> u64 {
    let mut bishop_blockers: u64 = blockers;
    let mut rook_blockers: u64 = blockers;

    unsafe {
        bishop_blockers &= BISHOP_RAYS[square];
        bishop_blockers *= BISHOP_MAGICS[square];
        bishop_blockers >>= 64 - BISHOP_RELEVANT_BITS[square];
    };

    let mut res = unsafe { BISHOP_ATTACKS[square][bishop_blockers as usize] };

    unsafe {
        rook_blockers &= ROOK_RAYS[square];
        rook_blockers *= ROOK_MAGICS[square];
        rook_blockers >>= 64 - ROOK_RELEVANT_BITS[square];
    };

    unsafe { res |= ROOK_ATTACKS[square][rook_blockers as usize] };
    res
}
