use crate::board::*;
use crate::helper::*;
use crate::magic::*;

pub const PAWN_VALUE: (i32, i32) = (105, 153);
pub const KNIGHT_VALUE: (i32, i32) = (471, 508);
pub const BISHOP_VALUE: (i32, i32) = (488, 535);
pub const ROOK_VALUE: (i32, i32) = (662, 828);
pub const QUEEN_VALUE: (i32, i32) = (1380, 1649);

//all PSQT have a1 on bottom left as viewing the code
//currently picked pretty arbitrarily

//meant to give idea that more central pawns are more valuable in mg
#[rustfmt::skip]
pub const PAWN_TABLE: [(i32, i32); 64] = [(22, 22), (27, 31), (-2, -2), (-13, -11), (24, 18), (15, 13), (-22, -24), (-1, 2), (169, 210), (13, 24), (128, 142), (166, 175), (68, 78), (55, 58), (109, 125), (296, 249), (66, 64), (56, 66), (39, 45), (33, 27), (72, 62), (60, 59), (19, 18)
, (66, 62), (13, 18), (6, 16), (9, 14), (16, 7), (28, 12), (30, 22), (21, 25), (17, 20), (-3, 0), (-4, -5), (-1, -1), (8,
 -1), (12, -6), (17, 8), (13, 13), (0, -2), (-19, -12), (-7, -1), (-9, -10), (-10, -1), (0, 1), (9, 11), (11, 10), (4, 2)
, (-9, -11), (-20, -1), (-20, -12), (-6, -2), (-2, 4), (6, 14), (11, 5), (-6, -4), (-8, -1), (-19, -13), (-18, -13), (-4,
 0), (-19, -21), (-5, -9), (25, 15), (-3, -5)];

#[rustfmt::skip]
pub const KNIGHT_TABLE: [(i32, i32); 64] = [(-23, -23), (-84, -78), (-13, -15), (-20, -22), (-23, -16), (-13, -12), (-41, -45), (-29, -30), (-26, -31), (5, 9), (20,
 23), (16, 19), (27, 28), (6, 5), (-1, -3), (-10, -12), (10, 8), (3, 2), (24, 23), (29, 32), (12, 16), (-12, -13), (40, 25), (-7, -6), (9, 9), (-10, -8), (-6, -3), (29, 28), (3, 4), (21, 26), (10, 9), (-10, -4), (-6, -2), (-12, -12), (1, 1), 
(14, 15), (27, 30), (23, 22), (2, 5), (-13, -17), (-20, -21), (-8, -7), (-5, -5), (0, 2), (1, 4), (3, -1), (-5, -8), (13,
 9), (-24, -23), (-19, -19), (-15, -14), (11, 9), (-2, -2), (-19, -24), (-13, -13), (-20, -16), (-14, -14), (-14, -17), (
-7, -8), (-17, -16), (-1, -3), (-24, -35), (-9, -3), (-1, 3)];

#[rustfmt::skip]
pub const BISHOP_TABLE: [(i32, i32); 64] = [(-24, -29), (-9, -10), (-14, -18), (-23, -17), (26, 20), (1, 3), (2, 0), (1, 2), (10, 12), (0, -2), (0, -1), (-4, 0), (2
, 9), (6, 4), (-8, -12), (-29, -24), (1, 0), (-14, -13), (-1, 7), (3, 4), (22, 26), (89, 72), (-6, -2), (-14, -18), (4, 2
), (-6, -3), (16, 15), (27, 31), (13, 14), (-8, -4), (-14, -11), (-6, -5), (-3, -8), (-6, -7), (-15, -13), (27, 33), (5, 
6), (-7, 4), (-19, -15), (15, 8), (-5, -8), (2, -1), (2, 2), (6, 4), (-2, -2), (8, 9), (-9, -6), (8, 11), (7, 6), (-5, -6
), (10, 5), (-9, 1), (-1, 0), (-6, -4), (14, 16), (9, 5), (-4, -5), (-6, -10), (-8, -8), (0, 0), (-5, -7), (-12, -11), (-
13, -13), (-13, -15)];

#[rustfmt::skip]
pub const ROOK_TABLE: [(i32, i32); 64] = [(0, 4), (7, 14), (14, 16), (27, 24), (28, 33), (-34, -18), (22, 27), (12, 10), (4, 8), (0, 0), (-7, -3), (19, 23), (8, 11), (11, 13), (35, 30), (4, 4), (5, 9), (27, 27), (-11, -6), (4, 8), (26, 28), (18, 13), (33, 27), (22, 23), (-14, -9), (
-3, -2), (-4, -5), (-4, 4), (0, -2), (15, 13), (16, 14), (12, 12), (-11, -14), (-13, -10), (10, 9), (5, -8), (-25, -20), 
(-10, -11), (9, 6), (-24, -21), (2, 1), (-12, -9), (-12, -11), (-21, -19), (0, -1), (-20, -23), (-10, -6), (-25, -27), (-
3, -7), (12, 11), (1, 0), (-12, -11), (6, 2), (12, 8), (-21, -22), (-14, -14), (-10, -6), (-2, -4), (6, 1), (12, -3), (6,
 2), (-1, 0), (2, -1), (-16, -19)];

#[rustfmt::skip]
pub const QUEEN_TABLE: [(i32, i32); 64] = [(-3, -6), (5, 7), (-1, -3), (20, 22), (12, 16), (11, 10), (20, 13), (6, 4), (3, 7), (8, 17), (26, 26), (17, 25), (28, 34
), (56, 56), (17, 13), (3, 5), (3, 3), (-1, -4), (5, 10), (-6, -3), (47, 49), (44, 41), (63, 67), (6, 6), (-14, -10), (-6
, -5), (21, 19), (26, 55), (29, 35), (22, 28), (12, 17), (42, 46), (-16, -15), (-22, -21), (3, 0), (20, 17), (0, 1), (-4,
 -2), (23, 23), (5, 3), (12, 5), (-10, -10), (9, 7), (-3, -1), (-6, -8), (12, 10), (27, 25), (-19, -19), (-26, -28), (-16
, -15), (0, -3), (6, 10), (7, 10), (10, 13), (-13, -14), (8, 10), (8, 9), (-13, -10), (-4, -8), (7, 7), (-9, -7), (-23, -
26), (-6, -11), (-39, -56)];

#[rustfmt::skip]
pub const KING_TABLE: [(i32, i32); 64] = [(-55, -45), (-38, -39), (-1, 2), (-7, -6), (-3, -7), (0, -2), (-18, -15), (-50, -46), (-21, -19), (-18, -14), (8, 12), (
-12, -11), (9, 8), (10, 13), (-39, -27), (-8, -8), (-43, -37), (-24, -24), (-22, -19), (-17, -14), (-8, -11), (9, 9), (20
, 20), (-13, -6), (-10, -5), (-14, -20), (-24, -14), (11, 17), (45, 48), (17, 18), (-2, -3), (-46, -34), (-24, -26), (-7,
 -7), (-7, -1), (3, 7), (-2, 0), (-1, 1), (4, 5), (-15, -16), (-3, -2), (7, 5), (-5, -2), (-13, -11), (-19, -20), (9, 4),
 (-19, -23), (-22, -27), (-3, -7), (3, 1), (-10, -8), (-14, -18), (-25, -22), (-20, -17), (-16, -13), (-23, -22), (-1, -7
), (5, -6), (4, 7), (-34, -26), (-5, -21), (-35, -29), (-1, -10), (-20, -27)];

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

pub const BISHOP_PAIR: (i32, i32) = (23, 27);
pub const ROOK_OPEN_FILE: (i32, i32) = (10, 12);
pub const ROOK_SEMI_OPEN_FILE: (i32, i32) = (5, 7);

pub const KING_SHIELD_BONUS: (i32, i32) = (5, 3);
pub const KING_OPEN_FILE_PENALTY: (i32, i32) = (5, -1);
pub const KING_SEMI_OPEN_FILE_PENALTY: (i32, i32) = (-6, -8);
pub const KING_VIRTUAL_MOBILITY_SCORE: [(i32, i32); 28] = [
    (0, 0),
    (0, 2),
    (2, 0),
    (4, 2),
    (-16, 35),
    (-16, 30),
    (-23, 29),
    (-30, 38),
    (-40, 41),
    (-49, 43),
    (-55, 40),
    (-73, 43),
    (-76, 43),
    (-92, 44),
    (-103, 46),
    (-93, 41),
    (-116, 41),
    (-124, 37),
    (-126, 34),
    (-112, 29),
    (-121, 28),
    (-142, 26),
    (-149, 23),
    (-128, 9),
    (-148, -8),
    (-134, -26),
    (-130, -32),
    (-135, -34),
];

pub const BISHOP_MOBILITY_SCORE: [(i32, i32); 14] = [
    (-31, -35),
    (-24, -30),
    (-7, -19),
    (-3, -6),
    (1, -3),
    (8, 10),
    (14, 11),
    (17, 20),
    (15, 26),
    (18, 34),
    (25, 23),
    (18, 20),
    (26, 30),
    (2, 1),
];

pub const ROOK_MOBILITY_SCORE: [(i32, i32); 15] = [
    (-33, -31),
    (-24, -25),
    (-18, -18),
    (-11, -23),
    (-13, -10),
    (2, -1),
    (4, 0),
    (11, 10),
    (16, 21),
    (22, 32),
    (34, 33),
    (26, 41),
    (43, 36),
    (44, 43),
    (29, 36),
];

pub const QUEEN_MOBILITY_SCORE: [(i32, i32); 28] = [
    (-34, -46),
    (-44, -40),
    (-31, -38),
    (-25, -43),
    (-23, -22),
    (-21, -29),
    (-14, -13),
    (-11, -13),
    (-8, -18),
    (-11, -10),
    (-5, -2),
    (3, 3),
    (3, 4),
    (1, 2),
    (3, 10),
    (11, 11),
    (11, 15),
    (4, 15),
    (25, 23),
    (-4, 2),
    (15, 24),
    (-2, -1),
    (2, 4),
    (-1, -2),
    (-57, -55),
    (-21, -25),
    (1, -1),
    (-65, -69),
];

pub const KNIGHT_MOBILITY_SCORE: [(i32, i32); 9] = [
    (-99, -114),
    (-28, -24),
    (-10, -10),
    (-1, -5),
    (6, 6),
    (8, 8),
    (12, 17),
    (22, 26),
    (27, 22),
];

pub const START_PHASE_SCORE: i32 = 12;
//possible for promotions to in theory result in more material than this

// bonus[rank][can advance]
pub const PASSED_PAWN_BONUS: [[(i32, i32); 2]; 8] = [
    [(24, 23), (-5, -7)],
    [(-1, -7), (5, 6)],
    [(-3, -6), (11, 10)],
    [(8, 9), (20, 47)],
    [(32, 29), (52, 69)],
    [(37, 43), (88, 123)],
    [(23, 29), (110, 122)],
    [(8, 5), (-14, -4)],
];

pub const ISOLATED_PAWN_PENALTY: (i32, i32) = (-25, -21);
pub const DOUBLED_PAWN_PENALTY: (i32, i32) = (-4, -6); //only given to the first pawn

pub const TEMPO: (i32, i32) = (34, 14);
pub const ROOK_ON_SEVENTH: (i32, i32) = (7, 35);

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
