use crate::board::*;
use crate::helper::*;
use crate::magic::*;

pub const PAWN_VALUE: (i32, i32) = (138, 168);
pub const KNIGHT_VALUE: (i32, i32) = (538, 515);
pub const BISHOP_VALUE: (i32, i32) = (585, 527);
pub const ROOK_VALUE: (i32, i32) = (714, 879);
pub const QUEEN_VALUE: (i32, i32) = (1380, 1649);

//all PSQT have a1 on bottom left as viewing the code
//currently picked pretty arbitrarily

#[rustfmt::skip]
pub const PAWN_TABLE: [(i32, i32); 64] = [
    (18, 18), (26, 23), (-3, -4), (-14, -16), (24, 22), (14, 14), (-22, -24), (-3, -1),
    (190, 210), (15, 24), (191, 212), (166, 175), (64, 72), (61, 56), (124, 132), (239, 249),
    (37, 64), (44, 66), (38, 45), (36, 38), (53, 46), (55, 59), (19, 23), (52, 54),
    (10, 15), (7, 16), (14, 13), (16, 7), (29, 12), (38, 22), (20, 25), (17, 20),
    (-6, -1), (-7, -2), (1, 3), (10, -1), (17, -6), (19, 8), (13, 12), (1, -1),
    (-18, -12), (-5, -6), (-9, -10), (-11, -1), (0, -2), (9, 11), (10, 11), (4, 2),
    (-10, -11), (-20, -1), (-26, -12), (-8, -4), (0, 4), (6, 14), (10, 5), (-7, -4),
    (-4, -4), (-19, -19), (-17, -18), (-5, -4), (-15, -19), (-3, -7), (23, 27), (-1, -1)
];

#[rustfmt::skip]
pub const KNIGHT_TABLE: [(i32, i32); 64] = [
    (-20, -20),  (-93, -75),  (-13, -11),  (-32, -30),  (-23, -22),  (-15, -12),  (-37, -40),  (-33, -25),  
    (-25, -30),  (6, 5),  (21, 23),  (18, 23),  (25, 26),  (10, 5),  (-1, 1),  (-14, -16),  
    (10, 12),  (11, 12),  (28, 25),  (66, 32),  (11, 12),  (-10, -9),  (57, 51),  (-3, -2),  
    (8, 7),  (-8, -6),  (5, 7),  (34, 33),  (5, 5),  (22, 25),  (13, 13),  (-10, -8),  
    (-12, -12),  (-7, -4),  (11, 11),  (11, 16),  (22, 26),  (15, 15),  (1, 5),  (-8, -8),  
    (-27, -26),  (-9, -7),  (-7, -7),  (8, 7),  (9, 6),  (4, -1),  (1, -8),  (1, 2),  
    (-42, -33),  (-19, -20),  (-23, -25),  (8, 6),  (-2, -2),  (-17, -13),  (-17, -17),  (-21, -20),  
    (-15, -14),  (-14, -14),  (-10, -11),  (-16, -14),  (-6, -6),  (-14, -13),  (-12, -11),  (0, 0),  
];

#[rustfmt::skip]
pub const BISHOP_TABLE: [(i32, i32); 64] = [
    (-25, -21), (-11, -9), (-13, -11), (-24, -24), (26, 26), (4, 2), (3, 3), (-2, -2),
    (6, 12), (4, 2), (2, 1), (-2, 0), (1, -1), (7, 8), (-3, -8), (-29, -26),
    (3, 2), (-14, -14), (0, 7), (7, 5), (27, 31), (88, 72), (0, -5), (-1, 1),
    (3, 3), (-6, -3), (7, 11), (25, 31), (24, 17), (0, 2), (-9, -7), (-8, -9),
    (-12, -10), (-4, -5), (-11, -8), (25, 25), (10, 11), (-6, -5), (-20, -15), (7, 8),
    (-9, -10), (3, 4), (2, 2), (4, 4), (-4, -1), (3, 2), (2, 3), (2, 0),
    (3, 5), (3, -6), (16, 5), (-16, 1), (-3, -1), (-2, 0), (21, 9), (4, 9),
    (-7, -8), (-3, -2), (-20, -21), (-3, -3), (-9, -9), (-37, -11), (-14, -15), (-15, -18)
];

#[rustfmt::skip]
pub const ROOK_TABLE: [(i32, i32); 64] = [
    (-2, 4), (8, 9), (20, 18), (32, 34), (30, 30), (-34, -18), (23, 20), (13, 14),
    (7, 9), (0, 2), (3, 5), (24, 24), (12, 15), (11, 12), (32, 33), (6, 7),
    (9, 7), (23, 22), (-8, -6), (9, 8), (26, 27), (19, 20), (42, 39), (27, 27),
    (-1, -3), (-1, 1), (-4, -2), (2, 0), (2, 2), (17, 17), (18, 19), (17, 18),
    (-10, -8), (-13, -9), (5, 6), (4, 3), (-16, -11), (-14, -11), (6, 4), (-16, -16),
    (-3, -4), (-13, -15), (-17, -14), (-16, -12), (-6, -4), (-17, -16), (-8, -6), (-21, -14),
    (-13, -12), (0, 2), (-1, -1), (-4, -10), (-1, -1), (0, 0), (-25, -23), (-16, -15),
    (-23, -23), (-5, -5), (7, 1), (22, -3), (13, 2), (-3, -5), (-2, 0), (-38, -19)
];

#[rustfmt::skip]
pub const QUEEN_TABLE: [(i32, i32); 64] = [
    (-4, -4), (4, 3), (-5, 0), (22, 23), (15, 14), (11, 12), (22, 22), (9, 11),
    (2, 3), (2, 2), (31, 27), (17, 16), (33, 34), (149, 132), (17, 20), (12, 12),
    (-1, 0), (0, 2), (5, 10), (-7, -5), (73, 80), (109, 121), (175, 165), (29, 20),
    (-13, -11), (-8, -8), (19, 20), (14, 55), (32, 30), (32, 34), (12, 17), (28, 23),
    (-21, -16), (-10, -11), (-4, 0), (5, 17), (2, 4), (-2, 0), (13, 16), (9, 9),
    (10, 5), (-5, -4), (3, 7), (-8, -6), (-6, -6), (1, 8), (13, 10), (-11, -16),
    (-22, -25), (-3, -6), (9, -4), (16, 4), (6, 4), (4, 7), (-11, -12), (4, 5),
    (9, 8), (-15, -15), (-12, -8), (6, 5), (-3, -10), (-22, -23), (-5, -8), (-48, -35)
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

pub fn game_phase_score(b: &Board) -> i32 {
    //score in starting position will be 4*1 + 2*2 + 1*2 + 1*2 = 12
    //lower score = closer to endgame
    (match b.side_to_move {
        Colour::White => {
            count(b.bitboards[BQ]) * 4
                + count(b.bitboards[BR]) * 2
                + count(b.bitboards[BB])
                + count(b.bitboards[BN])
        }
        Colour::Black => {
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
    while temp_pawns > 0 {
        let square = lsfb(temp_pawns);
        pawn_eval += tapered_score(PAWN_VALUE, phase_score);

        match colour {
            Colour::White => {
                pawn_eval += tapered_score(PAWN_TABLE[MIRROR[square]], phase_score);
                if WHITE_PASSED_MASKS[square] & b.bitboards[BP] == 0 {
                    //no blocking black pawns
                    let can_advance = match get_bit(square + 8, b.occupancies[BOTH]) {
                        0 => 1,
                        1 => 0,
                        _ => panic!("this is very problematic..."),
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
                        _ => panic!("this aint good chief"),
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

    while temp_knights > 0 {
        let square = lsfb(temp_knights);
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

    while temp_bishops > 0 {
        let square = lsfb(temp_bishops);
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
            _ => panic!("impossible"),
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
            _ => panic!("impossible"),
        },
    }
}

fn evaluate_rooks(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    let mut rook_eval = 0;
    let mut temp_rooks = match colour {
        Colour::White => b.bitboards[WR],
        Colour::Black => b.bitboards[BR],
    };
    while temp_rooks > 0 {
        rook_eval += tapered_score(ROOK_VALUE, phase_score);
        let square = lsfb(temp_rooks);

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

    while temp_queens > 0 {
        let square = lsfb(temp_queens);
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

fn evaluate_king(b: &Board, phase_score: i32, colour: Colour) -> i32 {
    let mut king_eval = 0;
    let king_bb = match colour {
        Colour::White => b.bitboards[WK],
        Colour::Black => b.bitboards[BK],
    };
    let king_square = lsfb(king_bb);
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
    //count number of attacks king would have if it were a queen and give penatly
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

pub fn evaluate(b: &Board) -> i32 {
    let mut eval: i32 = 0;
    let phase_score = game_phase_score(b);

    eval += evaluate_pawns(b, phase_score, Colour::White);
    eval += evaluate_knights(b, phase_score, Colour::White);
    eval += evaluate_bishops(b, phase_score, Colour::White);
    eval += evaluate_rooks(b, phase_score, Colour::White);
    eval += evaluate_queens(b, phase_score, Colour::White);
    eval += evaluate_king(b, phase_score, Colour::White);

    eval -= evaluate_pawns(b, phase_score, Colour::Black);
    eval -= evaluate_knights(b, phase_score, Colour::Black);
    eval -= evaluate_bishops(b, phase_score, Colour::Black);
    eval -= evaluate_rooks(b, phase_score, Colour::Black);
    eval -= evaluate_queens(b, phase_score, Colour::Black);
    eval -= evaluate_king(b, phase_score, Colour::Black);

    tapered_score(TEMPO, phase_score)
        + match b.side_to_move {
            //return from perspective of side to move
            Colour::White => eval,
            Colour::Black => -eval,
        }
}
