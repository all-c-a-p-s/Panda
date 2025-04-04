#![allow(unused)]

/*
 At the moment, the HCE is completely unused. However, I have kept the code here for reference.
*/

use crate::board::*;
use crate::helper::*;
use crate::magic::*;

use crate::nnue::Accumulator;

pub const PAWN_VALUE: (i32, i32) = (142, 168);
pub const KNIGHT_VALUE: (i32, i32) = (519, 547);
pub const BISHOP_VALUE: (i32, i32) = (567, 589);
pub const ROOK_VALUE: (i32, i32) = (726, 885);
pub const QUEEN_VALUE: (i32, i32) = (1391, 1646);

//all PSQT have a1 on bottom left as viewing the code

#[rustfmt::skip]
pub const PAWN_SAME_SIDE_TABLE: [(i32, i32); 64] = [
    (25, 30), (40, 37), (-4, -5), (-11, -9), (0, -4), (12, 10), (-24, -25), (-12, -14),
    (205, 157), (16, 13), (199, 238), (184, 170), (63, 80), (144, 171), (72, 79), (43, 47),
    (27, 16), (43, 48), (16, 20), (41, 53), (68, 53), (99, 104), (67, 74), (56, 50),
    (26, 32), (2, 3), (11, 11), (8, 8), (55, 12), (24, 24), (33, 30), (14, 16),
    (-14, -9), (-24, -2), (3, 5), (3, 7), (25, 22), (25, 22), (6, 10), (-5, -2),
    (1, 3), (-16, -6), (-5, -4), (-41, -27), (20, 18), (11, 10), (36, 8), (9, 6),
    (3, 4), (2, 1), (4, 2), (-39, -37), (-11, -12), (16, 16), (39, 5), (-2, -8),
    (0, 1), (-38, -30), (-24, -26), (3, 1), (-8, -10), (2, -1), (6, 5), (-4, -4),
];

#[rustfmt::skip]
pub const PAWN_OTHER_SIDE_TABLE: [(i32, i32); 64] = [
    (27, 24), (32, 31), (-3, -4), (-10, -9), (21, 21), (16, 16), (-34, -29), (-8, -6),
    (172, 185), (15, 24), (152, 173), (177, 167), (66, 68), (80, 79), (84, 75), (49, 60),
    (26, 64), (42, 35), (42, 35), (44, 38), (62, 52), (75, 84), (26, 24), (46, 38),
    (12, 9), (3, 5), (6, 6), (32, 7), (47, 12), (26, 22), (19, 21), (8, 8),
    (-24, -7), (-25, -2), (2, 0), (32, 14), (30, 25), (18, 18), (-3, -4), (-4, -6),
    (-15, -15), (-28, -6), (-7, -9), (-2, -1), (8, 11), (15, 14), (8, 13), (2, 3),
    (-23, -10), (-28, -1), (-33, -12), (-20, -22), (-15, -14), (-3, -1), (-2, 3), (-14, -8),
    (4, 2), (-26, -24), (-12, -22), (-3, -5), (-14, -18), (-1, -4), (19, 19), (-6, -6),
];

#[rustfmt::skip]
pub const KNIGHT_TABLE: [(i32, i32); 64] = [
    (-43, -47), (-106, -84), (-10, -11), (-32, -33), (-25, -24), (-13, -11), (-40, -40), (-43, -52),
    (-26, -22), (4, 4), (33, 25), (25, 23), (58, 40), (13, 13), (1, 3), (-11, -10),
    (11, 10), (13, 11), (28, 29), (116, 32), (21, 21), (-6, -5), (72, 49), (-3, -1),
    (4, 3), (-7, -6), (11, 12), (61, 68), (18, 18), (35, 30), (2, 13), (-9, -6),
    (-17, -18), (-5, -6), (13, 11), (3, 16), (21, 20), (17, 16), (-2, 0), (-10, -8),
    (-57, -26), (-10, -12), (-3, -7), (8, 8), (10, 10), (9, -1), (5, 6), (-27, -28),
    (-57, -43), (-22, -22), (-23, -23), (5, 3), (4, 1), (-11, -11), (-19, -20), (-21, -20),
    (-18, -20), (-19, -23), (-10, -10), (-16, -15), (-14, -11), (-14, -13), (-30, -34), (2, 3),
];

#[rustfmt::skip]
pub const BISHOP_TABLE: [(i32, i32); 64] = [
    (-29, -29), (-12, -12), (-10, -14), (-21, -24), (25, 26), (2, 2), (5, 4), (-1, 0),
    (5, 6), (0, 2), (2, 1), (-2, -1), (3, 0), (7, 7), (-1, 1), (-20, -26),
    (3, 5), (-9, -10), (-1, 7), (12, 10), (29, 24), (120, 72), (2, 2), (4, 6),
    (3, 2), (-10, -3), (7, 11), (27, 28), (24, 27), (0, 1), (-11, -7), (-11, -9),
    (-12, -14), (-3, -6), (-16, -17), (26, 30), (10, 11), (-6, -7), (-18, -19), (6, 7),
    (-12, -10), (8, 9), (0, 2), (2, 4), (-3, -1), (3, 4), (3, 0), (3, 2),
    (5, 6), (7, -6), (16, 5), (-20, 1), (-3, -3), (-2, 0), (25, 9), (4, 6),
    (-6, -7), (-1, -2), (-20, -21), (-3, -3), (-9, -7), (-40, -11), (-12, -9), (-15, -14),
];

#[rustfmt::skip]
pub const ROOK_TABLE: [(i32, i32); 64] = [
    (0, 4), (13, 9), (19, 18), (30, 34), (30, 30), (-34, -18), (22, 24), (14, 15),
    (9, 9), (-2, 2), (5, 5), (24, 25), (11, 15), (17, 17), (38, 28), (4, 6),
    (14, 14), (23, 24), (-7, -6), (9, 10), (31, 36), (23, 28), (47, 39), (27, 27),
    (-1, 0), (1, -1), (-1, -2), (4, 4), (2, 0), (17, 17), (17, 18), (17, 18),
    (-8, -6), (-15, -11), (5, 6), (3, 3), (-16, -11), (-14, -11), (3, 3), (-16, -17),
    (-6, -7), (-13, -15), (-17, -16), (-15, -13), (-6, -8), (-17, -17), (-7, -8), (-21, -14),
    (-16, -16), (-1, 2), (-1, -1), (-4, -5), (-2, -1), (0, -1), (-21, -21), (-22, -19),
    (-22, -19), (-9, -9), (6, 1), (22, -3), (16, 2), (-4, -5), (-1, 0), (-37, -19),
];

#[rustfmt::skip]
pub const QUEEN_TABLE: [(i32, i32); 64] = [
    (-3, -4), (6, 3), (-5, 0), (25, 29), (12, 18), (12, 12), (28, 26), (12, 8),
    (2, 4), (-5, -4), (26, 28), (15, 11), (30, 26), (166, 132), (23, 27), (22, 25),
    (0, 0), (2, -2), (7, 10), (-4, -3), (79, 88), (138, 156), (188, 169), (63, 71),
    (-13, -11), (-12, -8), (20, 20), (13, 55), (25, 37), (29, 35), (11, 9), (27, 30),
    (-20, -18), (-10, -11), (-6, 0), (1, 17), (3, 2), (-2, -1), (10, 16), (9, 9),
    (5, 2), (-2, -7), (-1, 0), (-7, -5), (-5, -6), (-5, 8), (13, 10), (-8, -5),
    (-26, -29), (-4, -5), (9, -4), (17, 4), (8, 4), (5, 2), (-11, -12), (4, 2),
    (7, 8), (-15, -13), (-12, -12), (6, 5), (-1, -2), (-24, -23), (-5, -4), (-48, -46),
];

#[rustfmt::skip]
pub const KING_TABLE: [(i32, i32); 64] = [
    (-55, -61), (-37, -41), (3, 0), (-6, -4), (-1, -2), (0, 1), (-17, -21), (-56, -64),
    (-14, -19), (-14, -15), (8, 10), (-9, -10), (6, 8), (9, 14), (-38, -27), (-5, -7),
    (-48, -49), (-28, -21), (-19, -14), (-14, -13), (-4, -4), (11, 13), (17, 24), (-17, -15),
    (-11, -8), (-12, -13), (-21, -14), (9, 17), (57, 64), (22, 22), (-2, -3), (-37, -39),
    (-27, -26), (-7, -8), (-7, -1), (2, 7), (-2, 0), (-1, 1), (3, 2), (-23, -25),
    (-6, -7), (3, 4), (-5, -6), (-7, -5), (-17, -11), (3, 4), (-15, -18), (-35, -41),
    (-2, -7), (-2, -4), (-12, -13), (-27, -13), (-36, -17), (-36, -17), (-17, -17), (-25, -29),
    (-14, -11), (6, -6), (5, 3), (-76, -26), (-9, -21), (-91, -32), (1, -10), (-17, -27)
];

pub const fn set_file(square: usize) -> u64 {
    match square % 8 {
        0 => A_FILE,
        1 => B_FILE,
        2 => C_FILE,
        3 => D_FILE,
        4 => E_FILE,
        5 => F_FILE,
        6 => G_FILE,
        7 => H_FILE,
        _ => unreachable!(),
    }
}

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
        _ => unreachable!(),
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
        _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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

pub const FILES: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        table[square] = set_file(square);
        square += 1;
    }
    table
};

pub const MIRROR: [usize; 64] = {
    let mut mirror = [0usize; 64];
    let mut square = 0;
    while square < 64 {
        mirror[square] = relative_psqt_square(square, Colour::White);
        square += 1;
    }
    mirror
};

pub const BISHOP_PAIR: (i32, i32) = (37, 43);
pub const ROOK_OPEN_FILE: (i32, i32) = (11, 13);
pub const ROOK_SEMI_OPEN_FILE: (i32, i32) = (14, 11);

pub const KING_SHIELD_BONUS: (i32, i32) = (6, 6);
pub const KING_OPEN_FILE_PENALTY: (i32, i32) = (-1, 1);
pub const KING_SEMI_OPEN_FILE_PENALTY: (i32, i32) = (-6, -6);
pub const KING_VIRTUAL_MOBILITY_SCORE: [(i32, i32); 28] = [
    (5, 3),
    (0, -2),
    (-1, 0),
    (-7, 0),
    (-18, -13),
    (-15, 30),
    (-16, 29),
    (-23, 38),
    (-33, 41),
    (-45, 43),
    (-44, 40),
    (-68, 43),
    (-56, 43),
    (-70, 44),
    (-86, 46),
    (-94, 41),
    (-101, 41),
    (-75, 37),
    (-91, 34),
    (-79, 29),
    (-114, 28),
    (-93, 26),
    (-129, 23),
    (-77, 9),
    (-114, -8),
    (-124, -26),
    (-145, -32),
    (-108, -34),
];

pub const BISHOP_MOBILITY_SCORE: [(i32, i32); 14] = [
    (-37, -41),
    (-27, -27),
    (-13, -19),
    (-12, -14),
    (-3, -4),
    (3, 5),
    (15, 19),
    (13, 20),
    (13, 26),
    (16, 34),
    (19, 27),
    (28, 32),
    (27, 24),
    (3, 3),
];

pub const ROOK_MOBILITY_SCORE: [(i32, i32); 15] = [
    (-40, -42),
    (-22, -22),
    (-19, -22),
    (-9, -14),
    (-10, -9),
    (0, 1),
    (6, 4),
    (15, 16),
    (27, 20),
    (30, 33),
    (38, 39),
    (39, 39),
    (46, 47),
    (56, 63),
    (44, 51),
];

pub const QUEEN_MOBILITY_SCORE: [(i32, i32); 28] = [
    (-33, -46),
    (-25, -27),
    (-26, -22),
    (-24, -43),
    (-22, -26),
    (-15, -29),
    (-15, -14),
    (-10, -14),
    (-11, -9),
    (-6, -7),
    (-6, -9),
    (-5, -4),
    (-4, -1),
    (-1, -2),
    (1, 10),
    (7, 5),
    (7, 15),
    (1, 15),
    (12, 15),
    (2, 2),
    (15, 19),
    (3, 2),
    (2, 3),
    (-2, 1),
    (-33, -27),
    (-18, -17),
    (-1, -1),
    (-56, -57),
];

pub const KNIGHT_MOBILITY_SCORE: [(i32, i32); 9] = [
    (-62, -49),
    (-20, -18),
    (-4, -10),
    (3, -5),
    (13, 6),
    (12, 12),
    (14, 19),
    (24, 27),
    (28, 28),
];

pub const START_PHASE_SCORE: i32 = 12;
//possible for promotions to in theory result in more material than this

// bonus[rank][can advance]
pub const PASSED_PAWN_BONUS: [[(i32, i32); 2]; 8] = [
    [(18, 17), (0, 0)],
    [(-9, -1), (2, 6)],
    [(1, 0), (3, 14)],
    [(4, 9), (12, 47)],
    [(29, 34), (57, 69)],
    [(69, 58), (107, 137)],
    [(26, 29), (197, 331)],
    [(8, 9), (-12, -11)],
];

pub const ISOLATED_PAWN_PENALTY: (i32, i32) = (-31, -24);
pub const DOUBLED_PAWN_PENALTY: (i32, i32) = (-10, -12); //only given to the first pawn

pub const TEMPO: (i32, i32) = (27, 14);
pub const ROOK_ON_SEVENTH: (i32, i32) = (41, 34);

pub fn game_phase_score(b: &Board, side: Colour) -> i32 {
    //score in starting position will be 4*1 + 2*2 + 1*2 + 1*2 = 12
    //lower score = closer to endgame
    (match side {
        Colour::Black => {
            count(b.bitboards[BQ]) * 4
                + count(b.bitboards[BR]) * 2
                + count(b.bitboards[BB])
                + count(b.bitboards[BN])
        }
        Colour::White => {
            count(b.bitboards[WQ]) * 4
                + count(b.bitboards[WR]) * 2
                + count(b.bitboards[WB])
                + count(b.bitboards[WN])
        }
    }) as i32
}

pub fn tapered_score(weight: (i32, i32), phase_score: i32) -> i32 {
    //(mg, eg)
    (phase_score * weight.0 + (START_PHASE_SCORE - phase_score) * weight.1) / START_PHASE_SCORE
}

pub const fn relative_psqt_square(square: usize, c: Colour) -> usize {
    match c {
        Colour::White => {
            //piece-square tables have a1 on bottom left -> a8 at index 0
            let relative_rank = 7 - rank(square);
            let file = square % 8;
            relative_rank * 8 + file
        }
        Colour::Black => square,
    }
}

fn evaluate_pawns(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    //return positive values and then negate for black in main evaluation function
    let mut pawn_eval = 0;
    let mut temp_pawns = match colour {
        Colour::White => b.bitboards[WP],
        Colour::Black => b.bitboards[BP],
    };

    //SAFETY: there MUST be a king on the board
    let king_kside = unsafe {
        lsfb(match colour {
            Colour::White => b.bitboards[WK],
            Colour::Black => b.bitboards[BK],
        })
        .unwrap_unchecked()
    } % 8
        > 3;

    while let Some(square) = lsfb(temp_pawns) {
        let is_kside = square % 8 > 3;

        #[allow(non_snake_case)]
        let PAWN_TABLE = if is_kside == king_kside {
            PAWN_SAME_SIDE_TABLE
        } else {
            PAWN_OTHER_SIDE_TABLE
        };

        pawn_eval += tapered_score(PAWN_VALUE, phase_score);

        match colour {
            Colour::White => {
                pawn_eval += tapered_score(PAWN_TABLE[MIRROR[square]], phase_score);
                if WHITE_PASSED_MASKS[square] & b.bitboards[BP] == 0 {
                    //no blocking black pawns
                    let can_advance = match get_bit(square + 8, b.occupancies[BOTH]) {
                        0 => 1,
                        1 => 0,
                        _ => unreachable!(),
                    };
                    pawn_eval +=
                        tapered_score(PASSED_PAWN_BONUS[rank(square)][can_advance], phase_score);
                }

                if ISOLATED_MASKS[square] & b.bitboards[WP] == 0 {
                    //penalty for isolated pawns
                    pawn_eval += tapered_score(ISOLATED_PAWN_PENALTY, phase_score);
                }

                if DOUBLED_MASKS[square] & b.bitboards[WP] != 0 {
                    //doubled pawn penalty
                    pawn_eval += tapered_score(DOUBLED_PAWN_PENALTY, phase_score);
                }
            }
            Colour::Black => {
                pawn_eval += tapered_score(PAWN_TABLE[square], phase_score);
                if BLACK_PASSED_MASKS[square] & b.bitboards[WP] == 0 {
                    //no blocking black pawns
                    let can_advance = match get_bit(square - 8, b.occupancies[BOTH]) {
                        0 => 1,
                        1 => 0,
                        _ => unreachable!(),
                    };
                    pawn_eval += tapered_score(
                        PASSED_PAWN_BONUS[7 - rank(square)][can_advance],
                        phase_score,
                    );
                }

                if ISOLATED_MASKS[square] & b.bitboards[BP] == 0 {
                    //penalty for isolated pawns
                    pawn_eval += tapered_score(ISOLATED_PAWN_PENALTY, phase_score);
                }

                if DOUBLED_MASKS[square] & b.bitboards[BP] != 0 {
                    pawn_eval += tapered_score(DOUBLED_PAWN_PENALTY, phase_score);
                }
            }
        }
        temp_pawns = pop_bit(square, temp_pawns)
    }
    pawn_eval
}

fn evaluate_knights(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    let mut knight_eval = 0;
    let mut temp_knights = match colour {
        Colour::White => b.bitboards[WN],
        Colour::Black => b.bitboards[BN],
    };

    while let Some(square) = lsfb(temp_knights) {
        let attacks = N_ATTACKS[square]
            & !b.occupancies[if colour == Colour::White {
                WHITE
            } else {
                BLACK
            }];
        knight_eval += tapered_score(KNIGHT_MOBILITY_SCORE[count(attacks)], phase_score);
        knight_eval += tapered_score(KNIGHT_VALUE, phase_score);
        match colour {
            Colour::White => {
                knight_eval += tapered_score(KNIGHT_TABLE[MIRROR[square]], phase_score);
            }
            Colour::Black => knight_eval += tapered_score(KNIGHT_TABLE[square], phase_score),
        }
        temp_knights = pop_bit(square, temp_knights);
    }
    knight_eval
}

fn evaluate_bishops(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    let mut bishop_eval = 0;
    let mut temp_bishops = match colour {
        Colour::White => b.bitboards[WB],
        Colour::Black => b.bitboards[BB],
    };

    if count(temp_bishops) >= 2 {
        bishop_eval += tapered_score(BISHOP_PAIR, phase_score);
    }

    while let Some(square) = lsfb(temp_bishops) {
        bishop_eval += tapered_score(BISHOP_VALUE, phase_score);
        let attacks = get_bishop_attacks(square, b.occupancies[BOTH])
            & !b.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }];
        bishop_eval += tapered_score(BISHOP_MOBILITY_SCORE[count(attacks)], phase_score);
        match colour {
            Colour::White => {
                bishop_eval += tapered_score(BISHOP_TABLE[MIRROR[square]], phase_score)
            }
            Colour::Black => bishop_eval += tapered_score(BISHOP_TABLE[square], phase_score),
        }
        temp_bishops = pop_bit(square, temp_bishops);
    }
    bishop_eval
}

pub fn above_rank(square: usize, c: Colour) -> u64 {
    //returns all bits set above the current square
    //used to detect rooks on open/semi open files
    match c {
        Colour::White => match square / 8 {
            0 => 0x00FFFFFFFFFFFFFF,
            1 => 0x0000FFFFFFFFFFFF,
            2 => 0x000000FFFFFFFFFF,
            3 => 0x00000000FFFFFFFF,
            4 => 0x0000000000FFFFFF,
            5 => 0x000000000000FFFF,
            6 => 0x00000000000000FF,
            7 => 0x0000000000000000,
            _ => unreachable!(),
        },
        Colour::Black => match square / 8 {
            0 => 0x0000000000000000,
            1 => 0x00000000000000FF,
            2 => 0x000000000000FFFF,
            3 => 0x0000000000FFFFFF,
            4 => 0x00000000FFFFFFFF,
            5 => 0x000000FFFFFFFFFF,
            6 => 0x0000FFFFFFFFFFFF,
            7 => 0x00FFFFFFFFFFFFFF,
            _ => unreachable!(),
        },
    }
}

fn evaluate_rooks(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    let mut rook_eval = 0;
    let mut temp_rooks = match colour {
        Colour::White => b.bitboards[WR],
        Colour::Black => b.bitboards[BR],
    };
    while let Some(square) = lsfb(temp_rooks) {
        rook_eval += tapered_score(ROOK_VALUE, phase_score);

        if rank(square) == 6 && colour == Colour::White
            || rank(square) == 1 && colour == Colour::Black
        {
            rook_eval += tapered_score(ROOK_ON_SEVENTH, phase_score);
        }
        let attacks = get_rook_attacks(square, b.occupancies[BOTH])
            & !b.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }];
        let attacks_up_file = attacks & above_rank(square, colour);
        let mut open_file = false;
        let mut semi_open_file = false;
        rook_eval += tapered_score(ROOK_MOBILITY_SCORE[count(attacks)], phase_score);
        match colour {
            Colour::White => {
                rook_eval += tapered_score(ROOK_TABLE[MIRROR[square]], phase_score);
                if attacks_up_file & b.bitboards[WP] == 0 {
                    if attacks_up_file & b.bitboards[BP] == 0 {
                        open_file = true;
                    } else {
                        semi_open_file = true;
                    }
                }
            }
            Colour::Black => {
                rook_eval += tapered_score(ROOK_TABLE[square], phase_score);
                if attacks_up_file & b.bitboards[BP] == 0 {
                    if attacks_up_file & b.bitboards[WP] == 0 {
                        open_file = true;
                    } else {
                        semi_open_file = true;
                    }
                }
            }
        }

        if open_file {
            rook_eval += tapered_score(ROOK_OPEN_FILE, phase_score);
        } else if semi_open_file {
            rook_eval += tapered_score(ROOK_SEMI_OPEN_FILE, phase_score);
        }

        temp_rooks = pop_bit(square, temp_rooks);
    }

    rook_eval
}

fn evaluate_queens(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    let mut queen_eval = 0;
    let mut temp_queens = match colour {
        Colour::White => b.bitboards[WQ],
        Colour::Black => b.bitboards[BQ],
    };

    while let Some(square) = lsfb(temp_queens) {
        queen_eval += tapered_score(QUEEN_VALUE, phase_score);
        let attacks = get_queen_attacks(square, b.occupancies[BOTH])
            & !b.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }];
        queen_eval += tapered_score(QUEEN_MOBILITY_SCORE[count(attacks)], phase_score);
        match colour {
            Colour::White => {
                queen_eval += tapered_score(QUEEN_TABLE[MIRROR[square]], phase_score);
            }
            Colour::Black => queen_eval += tapered_score(QUEEN_TABLE[square], phase_score),
        }
        temp_queens = pop_bit(square, temp_queens);
    }

    //maybe experiment with queen open file bonus
    //for now I haven't added because queen on open file in mg is often exposed to rooks
    //so idk if its actually something you should reward
    queen_eval
}

pub fn evaluate_king(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    let mut king_eval = 0;
    let king_bb = match colour {
        Colour::White => b.bitboards[WK],
        Colour::Black => b.bitboards[BK],
    };

    //SAFETY: there MUST be a king on the board
    let king_square = unsafe { lsfb(king_bb).unwrap_unchecked() };

    king_eval += match colour {
        Colour::White => tapered_score(KING_TABLE[MIRROR[king_square]], phase_score),
        Colour::Black => tapered_score(KING_TABLE[king_square], phase_score),
    };

    //will get queen attacks anyway for virtual mobility so this is better than
    //getting rook attacks and then queen attacks
    let attacks = get_queen_attacks(king_square, b.occupancies[BOTH])
        & !b.occupancies[match colour {
            Colour::White => WHITE,
            Colour::Black => BLACK,
        }];

    let attacks_up_file = attacks & above_rank(king_square, colour) & FILES[king_square];

    let mut open_file = false;
    let mut semi_open_file = false;

    let mut safety_score: i32 = 0;
    match colour {
        Colour::White => {
            //bonus for pawns shielding the king
            safety_score += count(K_ATTACKS[king_square] & b.bitboards[WP]) as i32
                * tapered_score(KING_SHIELD_BONUS, phase_score);
            if attacks_up_file & b.bitboards[BP] == 0 {
                if attacks_up_file & b.bitboards[WP] == 0 {
                    open_file = true;
                } else {
                    semi_open_file = true;
                }
            }
        }
        Colour::Black => {
            safety_score += count(K_ATTACKS[king_square] & b.bitboards[BP]) as i32
                * tapered_score(KING_SHIELD_BONUS, phase_score);
            if attacks_up_file & b.bitboards[WP] == 0 {
                if attacks_up_file & b.bitboards[BP] == 0 {
                    open_file = true;
                } else {
                    semi_open_file = true;
                }
            }
        }
    };

    //idea of virtual mobility heuristic:
    //count number of attacks king would have if it were a queen and give penalty
    //scaled by how many there are (i.e. how exposed the king is)
    safety_score += tapered_score(KING_VIRTUAL_MOBILITY_SCORE[count(attacks)], phase_score);

    if open_file {
        safety_score += tapered_score(KING_OPEN_FILE_PENALTY, phase_score);
    } else if semi_open_file {
        safety_score += tapered_score(KING_SEMI_OPEN_FILE_PENALTY, phase_score);
    }

    king_eval += safety_score;
    king_eval
}

fn side_has_sufficient_matieral(b: &Board, side: Colour) -> bool {
    match side {
        Colour::White => {
            count(b.bitboards[WP]) > 0
                || count(b.bitboards[WR]) > 0
                || count(b.bitboards[WQ]) > 0
                || count(b.bitboards[WB]) > 1
        }
        Colour::Black => {
            count(b.bitboards[BP]) > 0
                || count(b.bitboards[BR]) > 0
                || count(b.bitboards[BQ]) > 0
                || count(b.bitboards[BB]) > 1
        }
    }
}

pub fn evaluate(b: &Board) -> i32 {
    let s = b.nnue.evaluate(b.side_to_move);

    //TODO: endgame tablebase for better draw detection
    let side_sm = side_has_sufficient_matieral(b, b.side_to_move);
    let opp_sm = side_has_sufficient_matieral(b, b.side_to_move.opponent());

    if side_sm && opp_sm {
        s
    } else if side_sm {
        std::cmp::max(s, 0)
    } else {
        std::cmp::min(s, 0)
    }
}
