use indicatif::{ProgressBar, ProgressStyle};
use polars::prelude::*;
use rand::Rng;
use std::error::Error;

use crate::eval::*;

use crate::*;

/* Genetic Algorithm Parameters */
const POSITIONS_TO_USE: usize = 100000;
const MUTATION_RATE: i32 = 20;
const POPULATION_SIZE: i32 = 50;
const NUM_GENERATIONS: i32 = 50;

/* Simulated Annealing Parameters */
const MAX_CONSTANT: usize = 10000;
const MAX_TEMP: f32 = 1.0;
const K: f32 = 0.99;

pub const PAWN_VALUE_IDX: usize = 0;
pub const KNIGHT_VALUE_IDX: usize = 1;
pub const BISHOP_VALUE_IDX: usize = 2;
pub const ROOK_VALUE_IDX: usize = 3;
pub const QUEEN_VALUE_IDX: usize = 4;
pub const PAWN_TABLE_IDX: usize = 5;
pub const KNIGHT_TABLE_IDX: usize = 6;
pub const BISHOP_TABLE_IDX: usize = 7;
pub const ROOK_TABLE_IDX: usize = 8;
pub const QUEEN_TABLE_IDX: usize = 9;
pub const KING_TABLE_IDX: usize = 10;
pub const BISHOP_PAIR_IDX: usize = 11;
pub const ROOK_OPEN_FILE_IDX: usize = 12;
pub const ROOK_SEMI_OPEN_FILE_IDX: usize = 13;
pub const KING_SHIELD_BONUS_IDX: usize = 14;
pub const KING_OPEN_FILE_PENALTY_IDX: usize = 15;
pub const KING_SEMI_OPEN_FILE_PENALTY_IDX: usize = 16;
pub const KING_VIRTUAL_MOBILITY_IDX: usize = 17;
pub const BISHOP_MOBILITY_SCORE_IDX: usize = 18;
pub const ROOK_MOBILITY_SCORE_IDX: usize = 19;
pub const QUEEN_MOBILITY_SCORE_IDX: usize = 20;
pub const KNIGHT_MOBILITY_SCORE_IDX: usize = 21;
pub const PASSED_PAWN_BONUS_IDX: usize = 22;
pub const ISOLATED_PAWN_PENALTY_IDX: usize = 23;
pub const DOUBLED_PAWN_PENALTY_IDX: usize = 24;

fn init_weights() -> Vec<Vec<(i32, i32)>> {
    vec![
        // PAWN_VALUE_IDX = 0
        vec![(132, 118)],
        // KNIGHT_VALUE_IDX = 1
        vec![(238, 230)],
        // BISHOP_VALUE_IDX = 2
        vec![(478, 350)],
        // ROOK_VALUE_IDX = 3
        vec![(763, 800)],
        // QUEEN_VALUE_IDX = 4
        vec![(1109, 965)],
        // PAWN_TABLE_IDX = 5
        vec![
            (0, -3),
            (3, 0),
            (8, 0),
            (-5, -7),
            (0, 2),
            (10, 0),
            (-4, 0),
            (0, 0),
            (150, 132),
            (70, 51),
            (119, 103),
            (93, 95),
            (84, 91),
            (70, 93),
            (127, 136),
            (151, 148),
            (47, 54),
            (50, 47),
            (49, 51),
            (39, 43),
            (37, 30),
            (58, 48),
            (34, 50),
            (68, 59),
            (26, 21),
            (7, 11),
            (15, 6),
            (4, 3),
            (2, -2),
            (11, 4),
            (8, 8),
            (15, 16),
            (-9, 4),
            (-10, -4),
            (0, 2),
            (2, -2),
            (-8, -3),
            (-6, -9),
            (-1, -4),
            (0, 4),
            (7, 3),
            (-3, -6),
            (1, -2),
            (5, 6),
            (-13, 0),
            (-1, -2),
            (0, -2),
            (3, 6),
            (9, 3),
            (0, -3),
            (4, 4),
            (-14, -13),
            (-2, 2),
            (7, 5),
            (20, 4),
            (3, -1),
            (0, 0),
            (-3, 0),
            (-16, 0),
            (0, -4),
            (-1, 0),
            (0, 0),
            (4, 1),
            (1, 0),
        ],
        // KNIGHT_TABLE_IDX = 6
        vec![
            (-17, -24),
            (-41, -43),
            (-25, -15),
            (-23, -12),
            (-10, -12),
            (-8, -6),
            (-22, -21),
            (-28, -27),
            (-36, -14),
            (-13, -12),
            (20, -8),
            (3, -5),
            (20, -5),
            (-6, -7),
            (-6, -6),
            (-15, -13),
            (-5, -3),
            (-1, 1),
            (26, 28),
            (24, 24),
            (12, 8),
            (-10, 5),
            (1, -1),
            (-13, -10),
            (6, -5),
            (-2, 0),
            (7, 7),
            (14, 12),
            (5, 7),
            (17, 7),
            (0, 0),
            (-6, -7),
            (-8, -6),
            (-9, -6),
            (2, 3),
            (6, 8),
            (17, 8),
            (6, 6),
            (9, 10),
            (-10, -8),
            (-10, -3),
            (1, -1),
            (-8, 2),
            (0, 1),
            (2, -1),
            (-1, 3),
            (9, 1),
            (-8, -11),
            (-21, -6),
            (-9, -6),
            (-1, -5),
            (-3, -6),
            (2, -3),
            (-7, -10),
            (-10, -8),
            (-4, -10),
            (-7, -7),
            (-4, -3),
            (-7, -6),
            (-8, -12),
            (-2, 2),
            (-20, -5),
            (-12, -9),
            (-11, -14),
        ],
        // BISHOP_TABLE_IDX = 7
        vec![
            (-15, -7),
            (-9, -7),
            (-17, -10),
            (-30, -9),
            (-10, -10),
            (3, -10),
            (-1, 1),
            (1, -8),
            (-7, -4),
            (-1, -4),
            (6, -2),
            (8, 11),
            (-2, -3),
            (-5, -3),
            (-23, -23),
            (-16, -8),
            (-7, 0),
            (5, 1),
            (-4, -8),
            (-1, 3),
            (15, 3),
            (39, 35),
            (-4, 1),
            (3, 8),
            (-2, 2),
            (3, -2),
            (19, 3),
            (7, 10),
            (15, 13),
            (5, 3),
            (-10, 0),
            (-7, -2),
            (2, -3),
            (1, -4),
            (2, 0),
            (6, 8),
            (17, 8),
            (3, 3),
            (-5, 1),
            (8, 9),
            (-3, -4),
            (5, 0),
            (-3, 1),
            (2, 3),
            (3, 3),
            (12, 1),
            (2, 3),
            (-3, -4),
            (-3, -6),
            (15, 0),
            (0, 0),
            (0, -1),
            (8, 1),
            (-1, 0),
            (10, 11),
            (-4, -4),
            (-4, -5),
            (-7, -6),
            (-3, -3),
            (-5, -6),
            (-6, -6),
            (3, -4),
            (-8, -6),
            (-4, -3),
        ],
        // ROOK_TABLE_IDX = 8
        vec![
            (7, 10),
            (7, 5),
            (24, 22),
            (4, 5),
            (4, 1),
            (8, 9),
            (6, 2),
            (3, 3),
            (15, 15),
            (8, 8),
            (8, 6),
            (10, 13),
            (6, 8),
            (7, 8),
            (2, 3),
            (1, 4),
            (-2, 2),
            (10, 8),
            (-4, 4),
            (20, 22),
            (5, 7),
            (3, 4),
            (0, 1),
            (7, 3),
            (-7, -4),
            (-10, 0),
            (-2, 2),
            (7, 3),
            (5, 4),
            (1, 2),
            (1, 0),
            (1, 1),
            (-16, -2),
            (-4, -4),
            (-11, 1),
            (0, 3),
            (-13, 3),
            (-2, 1),
            (-4, 0),
            (-7, -5),
            (-14, -3),
            (2, -2),
            (0, -2),
            (-8, -8),
            (-16, 0),
            (-7, -4),
            (0, -5),
            (-9, -6),
            (-10, -5),
            (4, 6),
            (3, -2),
            (-3, 0),
            (3, 1),
            (10, 11),
            (-25, -18),
            (2, -1),
            (-1, 2),
            (-5, -5),
            (-2, 1),
            (0, 0),
            (-4, -3),
            (8, 11),
            (-3, -8),
            (-17, -12),
        ],
        // QUEEN_TABLE_IDX = 9
        vec![
            (3, -5),
            (7, 6),
            (4, 10),
            (6, 11),
            (13, 12),
            (0, 10),
            (34, 9),
            (4, 3),
            (-8, -7),
            (9, 10),
            (1, 14),
            (16, 20),
            (14, 13),
            (45, 58),
            (15, 15),
            (1, -1),
            (0, -1),
            (-4, -2),
            (4, 10),
            (13, 14),
            (20, 25),
            (41, 13),
            (22, 27),
            (9, 8),
            (-11, 1),
            (4, 1),
            (17, 18),
            (40, 36),
            (36, 34),
            (10, 23),
            (13, 20),
            (19, 24),
            (-10, -7),
            (-12, 14),
            (-2, -1),
            (24, 27),
            (-5, -5),
            (-1, 15),
            (9, 12),
            (-6, 10),
            (4, -10),
            (17, -6),
            (6, 3),
            (13, 3),
            (-1, 5),
            (11, 6),
            (12, 12),
            (2, 3),
            (-10, -11),
            (-12, -8),
            (-4, -5),
            (-6, -8),
            (-5, -5),
            (4, -8),
            (-10, -9),
            (13, -11),
            (-10, -15),
            (-9, -6),
            (-9, -12),
            (6, -20),
            (-8, -10),
            (-20, -16),
            (-18, -10),
            (-17, -20),
        ],
        // KING_TABLE_IDX = 10
        vec![
            (-59, -49),
            (-21, -17),
            (-22, -24),
            (-21, -19),
            (-13, -16),
            (-24, -6),
            (-20, -12),
            (-49, -50),
            (-18, -15),
            (-31, 12),
            (17, 14),
            (-42, 15),
            (3, 9),
            (16, 22),
            (-42, -44),
            (-5, -2),
            (-34, 4),
            (-38, -33),
            (-30, 14),
            (-21, -22),
            (12, 15),
            (15, 16),
            (12, 16),
            (-4, -5),
            (0, 1),
            (-17, 12),
            (-17, -17),
            (12, 12),
            (13, 16),
            (18, 16),
            (8, 7),
            (-26, 2),
            (-21, -18),
            (7, 9),
            (4, 8),
            (9, 9),
            (10, 9),
            (-22, 13),
            (-18, 9),
            (-11, -13),
            (-15, -21),
            (0, -1),
            (-2, -2),
            (-25, -1),
            (-1, 0),
            (-5, -6),
            (-2, -1),
            (-14, -12),
            (-8, -12),
            (-12, -12),
            (-18, -14),
            (-16, -17),
            (-22, -26),
            (-22, -8),
            (15, 16),
            (-34, -31),
            (-10, -51),
            (-28, -36),
            (34, -20),
            (-16, -16),
            (-25, -32),
            (-21, -20),
            (-21, -20),
            (-16, -15),
        ],
        // BISHOP_PAIR_IDX = 11
        vec![(21, 32)],
        // ROOK_OPEN_FILE_IDX = 12
        vec![(11, 9)],
        // ROOK_SEMI_OPEN_FILE_IDX = 13
        vec![(5, 6)],
        // KING_SHIELD_BONUS_IDX = 14
        vec![(0, 1)],
        // KING_OPEN_FILE_PENALTY_IDX = 15
        vec![(-7, -6)],
        // KING_SEMI_OPEN_FILE_PENALTY_IDX = 16
        vec![(-5, -1)],
        // KING_VIRTUAL_MOBILITY_IDX = 17
        vec![
            (-4, 0),
            (0, -1),
            (-2, -5),
            (0, 3),
            (1, 0),
            (1, 0),
            (-7, 0),
            (-1, 0),
            (-5, 0),
            (0, -3),
            (-2, 0),
            (0, -3),
            (-2, 0),
            (0, 4),
            (-1, 0),
            (3, 0),
            (3, 0),
            (0, -1),
            (-3, 0),
            (0, 0),
            (6, 0),
            (2, 0),
            (-4, 0),
            (0, 0),
            (0, 0),
            (2, 0),
            (-9, 0),
            (0, 0),
        ],
        // BISHOP_MOBILITY_SCORE_IDX = 18
        vec![
            (-58, -140),
            (-77, -93),
            (-4, 0),
            (-11, -11),
            (-2, 1),
            (15, 15),
            (14, 13),
            (26, 29),
            (37, 37),
            (25, 20),
            (42, 47),
            (41, 31),
            (42, 35),
            (1, -1),
        ],
        // ROOK_MOBILITY_SCORE_IDX = 19
        vec![
            (-113, -111),
            (-112, -95),
            (-54, -64),
            (-16, -21),
            (2, 5),
            (22, 18),
            (27, 24),
            (-8, 35),
            (33, 39),
            (47, 41),
            (15, 17),
            (24, 51),
            (8, 19),
            (41, 47),
            (8, 9),
        ],
        // QUEEN_MOBILITY_SCORE_IDX = 20
        vec![
            (-46, -19),
            (-279, -313),
            (-169, -171),
            (-138, -177),
            (-141, -130),
            (-63, -65),
            (-26, -26),
            (-1, -1),
            (4, 6),
            (20, 23),
            (22, 28),
            (34, 41),
            (27, 21),
            (3, 4),
            (14, 44),
            (16, 48),
            (21, 17),
            (12, 49),
            (48, 47),
            (14, 36),
            (19, 17),
            (8, 6),
            (-12, -19),
            (-17, -16),
            (-31, -36),
            (-50, -55),
            (-20, -20),
            (-34, -31),
        ],
        // KNIGHT_MOBILITY_SCORE_IDX = 21
        vec![
            (-74, -70),
            (-38, -76),
            (-20, -16),
            (-4, -4),
            (17, 13),
            (3, 1),
            (20, 21),
            (25, 31),
            (28, 26),
        ],
        // PASSED_PAWN_BONUS_IDX = 22
        vec![
            (4, 5),
            (-3, -4),
            (-9, -4),
            (26, 25),
            (-33, 6),
            (33, 28),
            (31, 25),
            (39, 31),
            (14, 46),
            (70, 63),
            (48, 48),
            (88, 92),
            (47, 55),
            (80, 91),
            (-6, 0),
            (-3, 0),
        ],
        // ISOLATED_PAWN_PENALTY_IDX = 23
        vec![(-12, -10)],
        // DOUBLED_PAWN_PENALTY_IDX = 24
        vec![(-27, -25)],
    ]
}

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
    (phase_score * weight.0 + (START_PHASE_SCORE - phase_score) * weight.1) / START_PHASE_SCORE
}

pub const fn relative_psqt_square(square: usize, c: Colour) -> usize {
    match c {
        Colour::White => {
            let relative_rank = 7 - rank(square);
            let file = square % 8;
            relative_rank * 8 + file
        }
        Colour::Black => square,
    }
}

fn evaluate_pawns(
    b: &Board,
    phase_score: i32,
    colour: Colour,
    weights: &Vec<Vec<(i32, i32)>>,
) -> i32 {
    let mut pawn_eval = 0;
    let mut temp_pawns = match colour {
        Colour::White => b.bitboards[WP],
        Colour::Black => b.bitboards[BP],
    };
    while temp_pawns > 0 {
        let square = lsfb(temp_pawns);
        pawn_eval += tapered_score(weights[PAWN_VALUE_IDX][0], phase_score);

        match colour {
            Colour::White => {
                pawn_eval += tapered_score(weights[PAWN_TABLE_IDX][MIRROR[square]], phase_score);
                if WHITE_PASSED_MASKS[square] & b.bitboards[BP] == 0 {
                    let can_advance = match get_bit(square + 8, b.occupancies[BOTH]) {
                        0 => 1,
                        1 => 0,
                        _ => panic!("this is very problematic..."),
                    };
                    pawn_eval += tapered_score(
                        weights[PASSED_PAWN_BONUS_IDX][rank(square) * 2 + can_advance],
                        phase_score,
                    );
                }

                if ISOLATED_MASKS[square] & b.bitboards[WP] == 0 {
                    pawn_eval += tapered_score(weights[ISOLATED_PAWN_PENALTY_IDX][0], phase_score);
                }

                if DOUBLED_MASKS[square] & b.bitboards[WP] != 0 {
                    pawn_eval += tapered_score(weights[DOUBLED_PAWN_PENALTY_IDX][0], phase_score);
                }
            }
            Colour::Black => {
                pawn_eval += tapered_score(weights[PAWN_TABLE_IDX][square], phase_score);
                if BLACK_PASSED_MASKS[square] & b.bitboards[WP] == 0 {
                    let can_advance = match get_bit(square - 8, b.occupancies[BOTH]) {
                        0 => 1,
                        1 => 0,
                        _ => panic!("this aint good chief"),
                    };
                    pawn_eval += tapered_score(
                        weights[PASSED_PAWN_BONUS_IDX][(7 - rank(square)) * 2 + can_advance],
                        phase_score,
                    );
                }

                if ISOLATED_MASKS[square] & b.bitboards[BP] == 0 {
                    pawn_eval += tapered_score(weights[ISOLATED_PAWN_PENALTY_IDX][0], phase_score);
                }

                if DOUBLED_MASKS[square] & b.bitboards[BP] != 0 {
                    pawn_eval += tapered_score(weights[DOUBLED_PAWN_PENALTY_IDX][0], phase_score);
                }
            }
        }
        temp_pawns = pop_bit(square, temp_pawns)
    }
    pawn_eval
}

fn evaluate_knights(
    b: &Board,
    phase_score: i32,
    colour: Colour,
    weights: &Vec<Vec<(i32, i32)>>,
) -> i32 {
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
        knight_eval += tapered_score(
            weights[KNIGHT_MOBILITY_SCORE_IDX][count(attacks)],
            phase_score,
        );
        knight_eval += tapered_score(weights[KNIGHT_VALUE_IDX][0], phase_score);
        match colour {
            Colour::White => {
                knight_eval +=
                    tapered_score(weights[KNIGHT_TABLE_IDX][MIRROR[square]], phase_score);
            }
            Colour::Black => {
                knight_eval += tapered_score(weights[KNIGHT_TABLE_IDX][square], phase_score)
            }
        }
        temp_knights = pop_bit(square, temp_knights);
    }
    knight_eval
}

fn evaluate_bishops(
    b: &Board,
    phase_score: i32,
    colour: Colour,
    weights: &Vec<Vec<(i32, i32)>>,
) -> i32 {
    let mut bishop_eval = 0;
    let mut temp_bishops = match colour {
        Colour::White => b.bitboards[WB],
        Colour::Black => b.bitboards[BB],
    };

    if count(temp_bishops) >= 2 {
        bishop_eval += tapered_score(weights[BISHOP_PAIR_IDX][0], phase_score);
    }

    while temp_bishops > 0 {
        let square = lsfb(temp_bishops);
        bishop_eval += tapered_score(weights[BISHOP_VALUE_IDX][0], phase_score);
        let attacks = get_bishop_attacks(square, b.occupancies[BOTH])
            & !b.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }];
        bishop_eval += tapered_score(
            weights[BISHOP_MOBILITY_SCORE_IDX][count(attacks)],
            phase_score,
        );
        match colour {
            Colour::White => {
                bishop_eval += tapered_score(weights[BISHOP_TABLE_IDX][MIRROR[square]], phase_score)
            }
            Colour::Black => {
                bishop_eval += tapered_score(weights[BISHOP_TABLE_IDX][square], phase_score)
            }
        }
        temp_bishops = pop_bit(square, temp_bishops);
    }
    bishop_eval
}

fn evaluate_rooks(
    b: &Board,
    phase_score: i32,
    colour: Colour,
    weights: &Vec<Vec<(i32, i32)>>,
) -> i32 {
    let mut rook_eval = 0;
    let mut temp_rooks = match colour {
        Colour::White => b.bitboards[WR],
        Colour::Black => b.bitboards[BR],
    };
    while temp_rooks > 0 {
        rook_eval += tapered_score(weights[ROOK_VALUE_IDX][0], phase_score);
        let square = lsfb(temp_rooks);
        let attacks = get_rook_attacks(square, b.occupancies[BOTH])
            & !b.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }];
        let attacks_up_file = attacks & above_rank(square, colour);
        let mut open_file = false;
        let mut semi_open_file = false;
        rook_eval += tapered_score(
            weights[ROOK_MOBILITY_SCORE_IDX][count(attacks)],
            phase_score,
        );
        match colour {
            Colour::White => {
                rook_eval += tapered_score(weights[ROOK_TABLE_IDX][MIRROR[square]], phase_score);
                if attacks_up_file & b.bitboards[WP] == 0 {
                    if attacks_up_file & b.bitboards[BP] == 0 {
                        open_file = true;
                    } else {
                        semi_open_file = true;
                    }
                }
            }
            Colour::Black => {
                rook_eval += tapered_score(weights[ROOK_TABLE_IDX][square], phase_score);
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
            rook_eval += tapered_score(weights[ROOK_OPEN_FILE_IDX][0], phase_score);
        } else if semi_open_file {
            rook_eval += tapered_score(weights[ROOK_SEMI_OPEN_FILE_IDX][0], phase_score);
        }

        temp_rooks = pop_bit(square, temp_rooks);
    }

    rook_eval
}

fn evaluate_queens(
    b: &Board,
    phase_score: i32,
    colour: Colour,
    weights: &Vec<Vec<(i32, i32)>>,
) -> i32 {
    let mut queen_eval = 0;
    let mut temp_queens = match colour {
        Colour::White => b.bitboards[WQ],
        Colour::Black => b.bitboards[BQ],
    };

    while temp_queens > 0 {
        let square = lsfb(temp_queens);
        queen_eval += tapered_score(weights[QUEEN_VALUE_IDX][0], phase_score);
        let attacks = get_queen_attacks(square, b.occupancies[BOTH])
            & !b.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }];
        queen_eval += tapered_score(
            weights[QUEEN_MOBILITY_SCORE_IDX][count(attacks)],
            phase_score,
        );
        match colour {
            Colour::White => {
                queen_eval += tapered_score(weights[QUEEN_TABLE_IDX][MIRROR[square]], phase_score);
            }
            Colour::Black => {
                queen_eval += tapered_score(weights[QUEEN_TABLE_IDX][square], phase_score)
            }
        }
        temp_queens = pop_bit(square, temp_queens);
    }
    queen_eval
}

fn evaluate_king(
    b: &Board,
    phase_score: i32,
    colour: Colour,
    weights: &Vec<Vec<(i32, i32)>>,
) -> i32 {
    let mut king_eval = 0;
    let king_bb = match colour {
        Colour::White => b.bitboards[WK],
        Colour::Black => b.bitboards[BK],
    };
    let king_square = lsfb(king_bb);
    king_eval += match colour {
        Colour::White => tapered_score(weights[KING_TABLE_IDX][MIRROR[king_square]], phase_score),
        Colour::Black => tapered_score(weights[KING_TABLE_IDX][king_square], phase_score),
    };

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
            safety_score += count(K_ATTACKS[king_square] & b.bitboards[WP]) as i32
                * tapered_score(weights[KING_SHIELD_BONUS_IDX][0], phase_score);
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
                * tapered_score(weights[KING_SHIELD_BONUS_IDX][0], phase_score);
            if attacks_up_file & b.bitboards[WP] == 0 {
                if attacks_up_file & b.bitboards[BP] == 0 {
                    open_file = true;
                } else {
                    semi_open_file = true;
                }
            }
        }
    };

    safety_score += tapered_score(
        weights[KING_VIRTUAL_MOBILITY_IDX][count(attacks)],
        phase_score,
    );

    if open_file {
        safety_score += tapered_score(weights[KING_OPEN_FILE_PENALTY_IDX][0], phase_score);
    } else if semi_open_file {
        safety_score += tapered_score(weights[KING_SEMI_OPEN_FILE_PENALTY_IDX][0], phase_score);
    }

    king_eval += safety_score;
    king_eval
}

pub fn evaluate(b: &Board, weights: &Vec<Vec<(i32, i32)>>) -> i32 {
    let mut eval: i32 = 0;
    let phase_score = game_phase_score(b);

    eval += evaluate_pawns(b, phase_score, Colour::White, weights);
    eval += evaluate_knights(b, phase_score, Colour::White, weights);
    eval += evaluate_bishops(b, phase_score, Colour::White, weights);
    eval += evaluate_rooks(b, phase_score, Colour::White, weights);
    eval += evaluate_queens(b, phase_score, Colour::White, weights);
    eval += evaluate_king(b, phase_score, Colour::White, weights);

    eval -= evaluate_pawns(b, phase_score, Colour::Black, weights);
    eval -= evaluate_knights(b, phase_score, Colour::Black, weights);
    eval -= evaluate_bishops(b, phase_score, Colour::Black, weights);
    eval -= evaluate_rooks(b, phase_score, Colour::Black, weights);
    eval -= evaluate_queens(b, phase_score, Colour::Black, weights);
    eval -= evaluate_king(b, phase_score, Colour::Black, weights);

    TEMPO
        + match b.side_to_move {
            Colour::White => eval,
            Colour::Black => -eval,
        }
}

fn is_insufficient_material(b: &Board) -> bool {
    if count(
        b.bitboards[WP]
            | b.bitboards[WR]
            | b.bitboards[WQ]
            | b.bitboards[BP]
            | b.bitboards[BR]
            | b.bitboards[BQ],
    ) != 0
    {
        return false;
    }
    if count(b.bitboards[WB]) >= 2 || count(b.bitboards[BB]) >= 2 {
        return false;
    }
    count(b.bitboards[WN]) <= 2 && count(b.bitboards[BN]) <= 2
    //can technically arise a position where KvKNN is mate so this
    //could cause some bug in theory lol
}

unsafe fn is_drawn(position: &Board) -> bool {
    if position.fifty_move == 100 {
        return true;
    }
    unsafe {
        for key in REPETITION_TABLE.iter().take(position.ply - 1) {
            //take ply - 1 because the start position (with 0 ply) is included
            if *key == position.hash_key {
                return true;
                //return true on one repetition because otherwise the third
                //repetition will not be reached because the search will stop
                //after a tt hit on the second repetition
            }
        }
    }
    is_insufficient_material(position)
}

const SEE_VALUES: [i32; 6] = [85, 306, 322, 490, 925, INFINITY];

impl Individual {
    fn quiescence_search(&mut self, position: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        unsafe {
            if is_drawn(position) {
                return 0;
            }
        }

        let eval = evaluate(position, &self.weights);
        //node count = every position that gets evaluated
        if eval >= beta {
            return beta;
        }

        //don't need repetition detection as it's impossible to have repetition with captures
        let delta = 1000; //delta pruning - try to avoid wasting time on hopeless positions
        if eval < alpha - delta {
            return alpha;
        }

        alpha = std::cmp::max(alpha, eval);

        let mut captures = MoveList::gen_captures(position);
        captures.order_moves(position, &Searcher::new(Instant::now()), &NULL_MOVE);

        for c in captures.moves {
            if c.is_null() {
                //no more pseudo-legal moves
                break;
            }

            let worst_case = SEE_VALUES[piece_type(position.pieces_array[c.square_to()])]
                - SEE_VALUES[piece_type(c.piece_moved(position))];

            if eval + worst_case > beta {
                //prune in the case that our move > beta even if we lose the piece
                //that we just moved
                return beta;
            }

            if !c.static_exchange_evaluation(position, 0) {
                //prune moves that fail see by threshold
                continue;
            }

            //prune neutral captures in bad positions (up to NxB)
            if eval + 200 <= alpha
                && !c.static_exchange_evaluation(
                    position,
                    SEE_VALUES[KNIGHT] - SEE_VALUES[BISHOP - 1],
                )
            {
                continue;
            }

            let (commit, ok) = position.try_move(c);

            if !ok {
                position.undo_move(c, commit);
                continue;
            }

            let eval = -self.quiescence_search(position, -beta, -alpha);
            position.undo_move(c, commit);
            if eval > alpha {
                alpha = eval;
            }
            if alpha >= beta {
                break;
            }
        }

        alpha
    }

    fn get_cost(&mut self, positions: &Vec<&str>, evals: &Vec<i32>) -> Result<(), Box<dyn Error>> {
        let mut total_error: u32 = 0;

        for (pos, sf_eval) in positions.iter().zip(evals.iter()) {
            let mut b = Board::from(pos);
            let eval = self.quiescence_search(&mut b, -INFINITY, INFINITY);

            total_error += (sf_eval - eval).abs() as u32;
        }
        self.cost = total_error; //want to minimise this value
        Ok(())
    }

    fn mutate(&self) -> Self {
        let mut n = Self {
            weights: self.weights.clone(),
            cost: 0,
        };
        for (i, w) in self.weights.iter().enumerate() {
            for (j, (v1, v2)) in w.iter().enumerate() {
                let mut rng = rand::thread_rng();

                //tune first
                let r = rng.gen_range(1..=100);

                if r <= MUTATION_RATE {
                    let delta = rng.gen_range(5..=15);
                    let change = (*v1 * delta) / 100;

                    let noise = rng.gen_range(-4..=4);

                    let up = rng.gen_bool(0.5);
                    let new_value = if up { *v1 + change } else { *v1 - change } + noise;
                    n.weights[i][j].0 = new_value;
                }

                //tune second
                let r = rng.gen_range(1..=100);

                if r <= MUTATION_RATE {
                    let delta = rng.gen_range(5..=15);
                    let change = (*v2 * delta) / 100;

                    //percentage change
                    let noise = rng.gen_range(-4..=4);

                    let up = rng.gen_bool(0.5);
                    let new_value = if up { *v1 + change } else { *v1 - change } + noise;
                    n.weights[i][j].1 = new_value;
                }
            }
        }

        n
    }

    fn new() -> Self {
        Self {
            cost: 0,
            weights: init_weights(),
        }
    }

    fn combine(&self, other: &Self) -> Self {
        let mut x = Self {
            cost: 0,
            weights: self.weights.clone(),
        };
        let mut rng = rand::thread_rng();
        for i in 0..self.weights.len() {
            for j in 0..self.weights[i].len() {
                let b = rng.gen_bool(0.5);
                if b {
                    x.weights[i][j] = other.weights[i][j];
                }
            }
        }

        x.mutate()
    }
}

#[derive(Clone, PartialEq)]
struct Individual {
    weights: Vec<Vec<(i32, i32)>>,
    cost: u32,
}

fn take_sample<'a>(positions: &'a Vec<&'a str>, evals: &Vec<i32>) -> (Vec<&'a str>, Vec<i32>) {
    let (mut pos_sample, mut ev_sample) = (vec![], vec![]);
    let p: f32 = POSITIONS_TO_USE as f32 / positions.len() as f32;
    for (pos, ev) in positions.iter().zip(evals.iter()) {
        let mut rng = rand::thread_rng();
        let x: f32 = rng.gen();

        if x < p {
            pos_sample.push(*pos);
            ev_sample.push(*ev);
        }
    }
    (pos_sample, ev_sample)
}

pub fn genetic_algorithm() -> Result<(), Box<dyn Error>> {
    let file_path = "/Users/seba/rs/Panda/data/chessData.csv";

    // Read the CSV file into a DataFrame
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(file_path.into()))?
        .finish()?;

    let str_column = df.get_columns()[0]
        .str()?
        .into_no_null_iter()
        .collect::<Vec<_>>();
    let i32_column = df.get_columns()[1]
        .str()?
        .into_no_null_iter()
        .collect::<Vec<_>>();

    let mut positions = Vec::new();
    let mut evals = Vec::new();

    for (string_value, i32_value) in str_column.iter().zip(i32_column.iter()) {
        match String::from(*i32_value).parse::<i32>() {
            Ok(x) => {
                positions.push(*string_value);
                evals.push(x);
            }
            Err(_) => continue, //skip mate evals
        }
    }

    println!("Successfully parsed data ✅ \n");

    let start = Individual::new();

    let mut population = vec![start.clone()];
    for _ in 0..POPULATION_SIZE - 1 {
        population.push(start.mutate());
    }

    for gen in 0..NUM_GENERATIONS {
        println!("Starting generation {} of {}! 🚀", gen + 1, NUM_GENERATIONS);
        let (pos_sample, ev_sample) = take_sample(&positions, &evals);
        let mut new_population = population.clone();
        for x in &population {
            let mut rng = rand::thread_rng();
            let n1 = rng.gen_range(0..POPULATION_SIZE);
            let child1 = x.combine(&population[n1 as usize]);

            let n2 = rng.gen_range(0..POPULATION_SIZE);
            let child2 = x.combine(&population[n2 as usize]);

            let n3 = rng.gen_range(0..POPULATION_SIZE);
            let child3 = x.combine(&population[n3 as usize]);

            let n4 = rng.gen_range(0..POPULATION_SIZE);
            let child4 = x.combine(&population[n4 as usize]);

            let child5 = x.mutate();
            let child6 = x.mutate();
            let child7 = x.mutate();
            let child8 = x.mutate();
            let child9 = x.mutate();

            new_population.extend(vec![
                child1, child2, child3, child4, child5, child6, child7, child8, child9,
            ]);
        }

        let bar = ProgressBar::new(500);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );

        for x in new_population.iter_mut() {
            match x.get_cost(
                &pos_sample, /*[..1_000_000].to_vec()*/
                &ev_sample,  /*[..1_000_000].to_vec()*/
            ) {
                Ok(_) => {}
                Err(e) => panic!("{}", e),
            };
            bar.inc(1);
        }
        bar.finish();
        new_population.sort_by_key(|x| x.cost); //ascending sort (which is what we want)
        population = new_population[..POPULATION_SIZE as usize].to_vec();

        println!(
            "Generation {} of {}, average error {}! 🌟 \n",
            gen + 1,
            NUM_GENERATIONS,
            population[0].cost as f32 / pos_sample.len() as f32
        );
    }

    for w in population[0].clone().weights {
        println!("{:?}", w);
    }

    Ok(())
}

fn acceptance_probability(delta_e: f32, temp: f32) -> f32 {
    (1.0 / delta_e) * temp
}

pub fn simulated_annealing() -> Result<(), Box<dyn Error>> {
    let mut temp: f32 = MAX_TEMP;

    let file_path = "/Users/seba/rs/Panda/data/chessData.csv";

    // Read the CSV file into a DataFrame
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(file_path.into()))?
        .finish()?;

    let str_column = df.get_columns()[0]
        .str()?
        .into_no_null_iter()
        .collect::<Vec<_>>();
    let i32_column = df.get_columns()[1]
        .str()?
        .into_no_null_iter()
        .collect::<Vec<_>>();

    let mut positions = Vec::new();
    let mut evals = Vec::new();

    for (string_value, i32_value) in str_column.iter().zip(i32_column.iter()) {
        match String::from(*i32_value).parse::<i32>() {
            Ok(x) => {
                positions.push(*string_value);
                evals.push(x);
            }
            Err(_) => continue, //skip mate evals
        }
    }

    println!("Successfully parsed data ✅ \n");

    let mut old = Individual::new();
    let mut constant = 0;
    while constant < MAX_CONSTANT {
        let (pos_sample, ev_sample) = take_sample(&positions, &evals);
        let mut new = old.mutate();

        old.get_cost(&pos_sample, &ev_sample)?;
        new.get_cost(&pos_sample, &ev_sample)?;

        let s_old = 1.0;
        let s_new = new.cost as f32 / old.cost as f32;
        let delta_e = s_new / s_old;

        if delta_e < 1.0 {
            println!(
                "Accepted cost: {} vs old cost: {}",
                new.cost as f32 / pos_sample.len() as f32,
                old.cost as f32 / pos_sample.len() as f32
            );
            old = new;
            constant = 0;
        } else {
            let mut rng = rand::thread_rng();
            let x: f32 = rng.gen();
            let p = acceptance_probability(delta_e, temp);

            if x <= p {
                println!(
                    "Accepted cost: {} vs old cost: {}",
                    new.cost as f32 / pos_sample.len() as f32,
                    old.cost as f32 / pos_sample.len() as f32
                );
                old = new;
                constant = 0;
            } else {
                constant += 1;
            }
        }
        temp *= K;
    }

    for w in old.weights {
        println!("{:?}", w);
    }
    Ok(())
}

pub fn hill_climbing() -> Result<(), Box<dyn Error>> {
    let file_path = "/Users/seba/rs/Panda/data/chessData.csv";

    // Read the CSV file into a DataFrame
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(file_path.into()))?
        .finish()?;

    let str_column = df.get_columns()[0]
        .str()?
        .into_no_null_iter()
        .collect::<Vec<_>>();
    let i32_column = df.get_columns()[1]
        .str()?
        .into_no_null_iter()
        .collect::<Vec<_>>();

    let mut positions = Vec::new();
    let mut evals = Vec::new();

    for (string_value, i32_value) in str_column.iter().zip(i32_column.iter()) {
        match String::from(*i32_value).parse::<i32>() {
            Ok(x) => {
                positions.push(*string_value);
                evals.push(x);
            }
            Err(_) => continue, //skip mate evals
        }
    }

    println!("Successfully parsed data ✅ \n");

    let mut old = Individual::new();
    let mut constant = 0;
    while constant < MAX_CONSTANT {
        let (pos_sample, ev_sample) = take_sample(&positions, &evals);
        let mut new = old.mutate();

        old.get_cost(&pos_sample, &ev_sample)?;
        new.get_cost(&pos_sample, &ev_sample)?;

        if new.cost < old.cost {
            println!(
                "Accepted cost: {} vs old cost: {}",
                new.cost as f32 / pos_sample.len() as f32,
                old.cost as f32 / pos_sample.len() as f32
            );
            old = new;
            constant = 0;
        }
    }

    Ok(())
}

//TODO: pick 100000 randoms from the whole thing every time!
