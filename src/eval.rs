use crate::board::*;
use crate::helper::*;
use crate::is_attacked;
use crate::magic::*;
use crate::search::INFINITY;

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 305;
const BISHOP_VALUE: i32 = 333;
const ROOK_VALUE: i32 = 513;
const QUEEN_VALUE: i32 = 920;

#[rustfmt::skip]
const WP_TABLE: [i32; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
    0,  10,  10, -20, -20,  15,  10,   0,
    5,  -5, -10,   0,   0, -15,  -5,   5,
    0,   0,   0,  15,  15,   0,   0,   0,
    5,   5,  10,  25,  25,  10,   5,  15,
   30,  30,  30,  40,  40,  30,  30,  35,
   60,  60,  60,  70,  70,  60,  60,  60,
    0,   0,   0,   0,   0,   0,   0,   0
];

#[rustfmt::skip]
const WN_TABLE: [i32; 64] = [
    -25, -20, -15, -15, -15, -15, -20, -25,
    -15, -20,   0  , 0,   0,   0, -20, -15,
    -10,   0,  15,   5,   5,  15,   0, -10,
    -10,   5,  15,  20,  20,  15,   5, -10,
     -5,   0,  15,  20,  20,  15,   0,  -5,
    -15,   5,  20,  25,  25,  20,   5, -15,
    -20, -20,   0,   5,   5,   0, -20, -20,
    -50, -40, -30, -30, -30, -30, -40, -50
];

#[rustfmt::skip]
const WB_TABLE: [i32; 64] = [
     -5, -10, -15, -10, -10, -15, -10,  -5,
    -10,  20,   0,   0,   0,   0,  20, -10,
    -10,  10,  10,  15,  15,  10,  10, -10,
     10,   0,  20,  15,  15,  20,   0, -10,
     10,   5,   5,  10,  10,   5,  10, -10,
    -10,   0,   5,   0,   0,   5,   0, -10,
    -20, -10,  -5,  -5,  -5,  -5, -10, -20,
    -30, -30, -30, -30, -30, -30, -30, -30
];

#[rustfmt::skip]
const WR_TABLE: [i32; 64] = [
    0,  0, 10, 15, 15, 10,  0, 0,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   15, 25, 25, 25, 25, 25, 25, 15,
    0,  0,  0,  0,  0,  0,  0, 0
];

#[rustfmt::skip]
const WQ_TABLE: [i32; 64] = [
    -20, -10, -10,  0, -5, -10, -10, -20,
    -10,   0,   0,  0,  0,   0,   0, -10,
    -10,   0,   5,  5,  5,   5,   0, -10,
     -5,   0,   5,  5,  5,   5,   0,  -5,
      0,   0,   5,  5,  5,   5,   0,  -5,
    -10,   5,   5,  5,  5,   5,   0, -10,
    -10,   0,   5,  0,  0,   0,   0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20
];

#[rustfmt::skip]
const WK_TABLE: [i32; 64] = [
    20, 30,   20, -10,  -5, -10,  30,  10,
    20,  0,  -10, -10, -10, -10,   0,  20,
   -10, -20, -20, -20, -20, -20, -20, -10,
   -20, -30, -30, -40, -40, -30, -30, -20,
   -30, -40, -40, -50, -50, -40, -40, -30,
   -30, -40, -40, -50, -50, -40, -40, -30,
   -30, -40, -40, -50, -50, -40, -40, -30,
   -30, -40, -40, -50, -50, -40, -40, -30
];

#[rustfmt::skip]
const WK_ENDGAME: [i32; 64] = [
    -40, -35, -30, -25, -25, -30, -35, -40,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -50, -34, -21, -11, -28, -14, -24, -50
];

#[rustfmt::skip]
const BP_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
   60, 60, 60, 70, 70, 60, 60, 60,
   20, 25, 25, 40, 40, 25, 25, 20,
    5,  5, 10, 25, 25, 10,  5, 10,
    0,  0,  0, 15, 15,  0,  0,  0,
    5, -5, -10, 0,  0,-15, -5,  5,
    0, 10, 10,-20,-20, 15, 10,  0,
    0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
const BN_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -20, -20,   0,   5,   5,   0, -20, -20,
    -15,   5,  20,  25,  25,  20,   5, -15,
     -5,   0,  15,  20,  20,  15,   0,  -5,
    -10,   5,  15,  20,  20,  15,   5, -10,
    -10,   0,  15,   5,   5,  15,   0, -10,
    -15, -20,   0  , 0,   0,   0, -20, -15,
    -25, -20, -15, -15, -15, -15, -20, -25
];

#[rustfmt::skip]
const BB_TABLE: [i32; 64] = [
    -30, -30, -30, -30, -30, -30, -30, -30,
    -20, -10,  -5,  -5,  -5,  -5, -10, -20,
    -10,   0,   5,   0,   0,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,  10, -10,
    -10,   0,  20,  15,  15,  20,   0, -10,
    -10,  10,  10,  15,  15,  10,  10, -10,
    -10,  20,   0,   0,   0,   0,  20, -10,
    -20, -10, -15, -10, -10, -15, -10, -20
];

#[rustfmt::skip]
const BR_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0, 0,
   15, 25, 25, 25, 25, 25, 25, 15,
    5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0, 10, 15, 15, 10,  0, 0
];

#[rustfmt::skip]
const BQ_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10,   0,   5,  0,  0,   0,   0, -10,
    -10,   5,   5,  5,  5,   5,   0, -10,
      0,   0,   5,  5,  5,   5,   0,  -5,
     -5,   0,   5,  5,  5,   5,   0,  -5,
    -10,   0,   5,  5,  5,   5,   0, -10,
    -10,   0,   0,  0,  0,   0,   0, -10,
    -20, -10, -10,  0, -5, -10, -10, -20
];

#[rustfmt::skip]
const BK_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
     20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  0,  -10, -10, -10, -10,   0,  20,
     20, 30,   20, -10,  -5, -10,  30,  10
];

//edit this later
#[rustfmt::skip]
const BK_ENDGAME: [i32; 64] = [
    -40, -35, -30, -25, -25, -30, -35, -40,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -50, -34, -21, -11, -28, -14, -24, -50
];

pub const fn passed_pawn_mask_white(square: usize) -> u64 {
    let mut res = 0u64;
    let mut sq = 0;
    while sq < 64 {
        if rank(sq) > rank(square) {
            res |= set_bit(sq, 0);
        }
        sq += 1;
    }
    res &= match square % 8 {
        0 => A_FILE | B_FILE,
        1 => A_FILE | B_FILE | C_FILE,
        2 => B_FILE | C_FILE | D_FILE,
        3 => C_FILE | D_FILE | E_FILE,
        4 => D_FILE | E_FILE | F_FILE,
        5 => E_FILE | F_FILE | G_FILE,
        6 => F_FILE | G_FILE | H_FILE,
        7 => G_FILE | H_FILE,
        _ => panic!("impossible"),
    };

    res
}

pub const fn passed_pawn_mask_black(square: usize) -> u64 {
    let mut res = 0u64;
    let mut sq = 0;
    while sq < 64 {
        if rank(sq) < rank(square) {
            res |= set_bit(sq, 0);
        }
        sq += 1;
    }
    res &= match square % 8 {
        0 => A_FILE | B_FILE,
        1 => A_FILE | B_FILE | C_FILE,
        2 => B_FILE | C_FILE | D_FILE,
        3 => C_FILE | D_FILE | E_FILE,
        4 => D_FILE | E_FILE | F_FILE,
        5 => E_FILE | F_FILE | G_FILE,
        6 => F_FILE | G_FILE | H_FILE,
        7 => G_FILE | H_FILE,
        _ => panic!("impossible"),
    };
    res
}

pub const ISOLATED_MASKS: [u64; 64] = {
    let mut res = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        res[square] = match square % 8 {
            0 => B_FILE,
            1 => A_FILE | C_FILE,
            2 => B_FILE | D_FILE,
            3 => C_FILE | E_FILE,
            4 => D_FILE | F_FILE,
            5 => E_FILE | G_FILE,
            6 => F_FILE | H_FILE,
            7 => G_FILE,
            _ => panic!("impossible"),
        };
        square += 1;
    }
    res
};

pub const DOUBLED_MASKS: [u64; 64] = {
    let mut res = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        res[square] = match square % 8 {
            0 => pop_bit(square, A_FILE),
            1 => pop_bit(square, B_FILE),
            2 => pop_bit(square, C_FILE),
            3 => pop_bit(square, D_FILE),
            4 => pop_bit(square, E_FILE),
            5 => pop_bit(square, F_FILE),
            6 => pop_bit(square, G_FILE),
            7 => pop_bit(square, H_FILE),
            _ => panic!("impossible"),
        };
        square += 1;
    }
    res
};

pub const WHITE_PASSED_MASKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        table[square] = passed_pawn_mask_white(square);
        square += 1;
    }
    table
};

pub const BLACK_PASSED_MASKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        table[square] = passed_pawn_mask_black(square);
        square += 1;
    }
    table
};

const BISHOP_BASE_MOBILITY: i32 = 4;
const ROOK_BASE_MOBILITY: i32 = 2;
const QUEEN_BASE_MOBILITY: i32 = 9;

const BISHOP_MOBILITY_UNIT: i32 = 4;
const ROOK_MOBILITY_UNIT: i32 = 3;
const QUEEN_MOBILITY_UNIT: i32 = 1;

const START_MATERIAL: i32 =
    PAWN_VALUE * 16 + KNIGHT_VALUE * 4 + BISHOP_VALUE * 4 + ROOK_VALUE * 4 + QUEEN_VALUE * 2;
//possible for promotions to in theory result in more material than this

const PASSED_PAWN_BONUS: [i32; 8] = [0, 5, 5, 20, 40, 65, 105, 0];

const ISOLATED_PAWN_PENALTY: i32 = -12;
const DOUBLED_PAWN_PENALTY: i32 = -14;

pub fn game_phase_score(material_count: i32) -> f32 {
    material_count as f32 / START_MATERIAL as f32
}

pub fn evaluate(b: &Board) -> i32 {
    //TODO: king safety
    let mut eval: i32 = 0;

    let mut white_material: i32 = 0;
    let mut black_material: i32 = 0;

    white_material += count(b.bitboards[0]) as i32 * PAWN_VALUE;
    white_material += count(b.bitboards[1]) as i32 * KNIGHT_VALUE;
    white_material += count(b.bitboards[2]) as i32 * BISHOP_VALUE;
    white_material += count(b.bitboards[3]) as i32 * ROOK_VALUE;
    white_material += count(b.bitboards[4]) as i32 * QUEEN_VALUE;

    black_material += count(b.bitboards[6]) as i32 * PAWN_VALUE;
    black_material += count(b.bitboards[7]) as i32 * KNIGHT_VALUE;
    black_material += count(b.bitboards[8]) as i32 * BISHOP_VALUE;
    black_material += count(b.bitboards[9]) as i32 * ROOK_VALUE;
    black_material += count(b.bitboards[10]) as i32 * QUEEN_VALUE;

    let phase_score = game_phase_score(white_material + black_material);

    eval += white_material;
    eval -= black_material;

    for i in 0..12 {
        let mut bitboard = b.bitboards[i];
        while bitboard > 0 {
            let square = lsfb(bitboard).unwrap();
            match i {
                0 => {
                    eval += WP_TABLE[square];
                    if WHITE_PASSED_MASKS[square] & b.bitboards[6] == 0 {
                        //no blocking black pawns
                        eval += PASSED_PAWN_BONUS[rank(square)];
                    }

                    if ISOLATED_MASKS[square] & b.bitboards[0] == 0 {
                        //penalty for isolated pawns
                        eval += ISOLATED_PAWN_PENALTY;
                    }

                    if DOUBLED_MASKS[square] & b.bitboards[0] != 0 {
                        //doubled pawn penalty
                        eval += DOUBLED_PAWN_PENALTY;
                    }
                }
                1 => eval += WN_TABLE[square],
                2 => {
                    eval += WB_TABLE[square];
                    eval += (count(get_bishop_attacks(square, b.occupancies[2])) as i32
                        - BISHOP_BASE_MOBILITY) as i32
                        * BISHOP_MOBILITY_UNIT;
                }
                3 => {
                    eval += WR_TABLE[square];
                    eval += (count(get_rook_attacks(square, b.occupancies[2])) as i32
                        - ROOK_BASE_MOBILITY) as i32
                        * ROOK_MOBILITY_UNIT;
                }
                4 => {
                    eval += WQ_TABLE[square];
                    eval += std::cmp::max(
                        (count(get_queen_attacks(square, b.occupancies[2])) as i32
                            - QUEEN_BASE_MOBILITY) as i32,
                        0,
                    ) * QUEEN_MOBILITY_UNIT;
                }
                5 => {
                    eval += {
                        ((WK_TABLE[square] as f32 * phase_score
                            + WK_ENDGAME[square] as f32 * (1f32 - phase_score))
                            / 2f32) as i32
                    }
                }

                6 => {
                    eval -= BP_TABLE[square];
                    if BLACK_PASSED_MASKS[square] & b.bitboards[0] == 0 {
                        //no blocking black pawns
                        eval -= PASSED_PAWN_BONUS[7 - rank(square)];
                    }

                    if ISOLATED_MASKS[square] & b.bitboards[6] == 0 {
                        //penalty for isolated pawns
                        eval -= ISOLATED_PAWN_PENALTY;
                    }

                    if DOUBLED_MASKS[square] & b.bitboards[6] != 0 {
                        eval -= DOUBLED_PAWN_PENALTY;
                    }
                }
                7 => eval -= BN_TABLE[square],
                8 => {
                    eval -= BB_TABLE[square];
                    eval -= (count(get_bishop_attacks(square, b.occupancies[2])) as i32
                        - BISHOP_BASE_MOBILITY) as i32
                        * BISHOP_MOBILITY_UNIT;
                }
                9 => {
                    eval -= BR_TABLE[square];
                    eval -= (count(get_rook_attacks(square, b.occupancies[2])) as i32
                        - ROOK_BASE_MOBILITY) as i32
                        * ROOK_MOBILITY_UNIT;
                }
                10 => {
                    eval -= BQ_TABLE[square];
                    eval -= std::cmp::max(
                        (count(get_queen_attacks(square, b.occupancies[2])) as i32
                            - QUEEN_BASE_MOBILITY) as i32,
                        0,
                    ) * QUEEN_MOBILITY_UNIT;
                }
                11 => {
                    eval -= {
                        ((BK_TABLE[square] as f32 * phase_score
                            + BK_ENDGAME[square] as f32 * (1f32 - phase_score))
                            / 2f32) as i32
                    }
                }
                _ => panic!("impossible"),
            }
            bitboard = pop_bit(square, bitboard);
        }
    }

    match b.side_to_move {
        //return from perspective of side to move
        Colour::White => eval,
        Colour::Black => -eval,
    }
}

pub fn is_checkmate(b: Board) -> i32 {
    //should only be called when there are no legal moves
    match b.side_to_move {
        Colour::White => {
            let king_square = lsfb(b.bitboards[5]).unwrap(); //there must be a king on the board
            if is_attacked(king_square, Colour::Black, &b) {
                return INFINITY;
            }
        }
        Colour::Black => {
            let king_square = lsfb(b.bitboards[11]).unwrap(); //there must be a king on the board
            if is_attacked(king_square, Colour::White, &b) {
                return INFINITY;
            }
        }
    }
    0
}
