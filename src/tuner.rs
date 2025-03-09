use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use polars::prelude::*;
use rand::Rng;
use rayon::prelude::*;
use rusqlite::Connection;
use std::error::Error;

use crate::eval::*;

use crate::*;

/* Genetic Algorithm Parameters */
const POSITIONS_TO_USE: usize = 100_000;
const START_MUTATION_RATE: i32 = 100;
const MUTATION_RATE: i32 = 20; //out of 1000
const POPULATION_SIZE: i32 = 100;
const NUM_GENERATIONS: i32 = 50;

/* Simulated Annealing Parameters */
const MAX_ITERATIONS: usize = 2_000;
const MAX_CONSTANT: usize = 500;
const MAX_TEMP: f32 = 1.0;
const K: f32 = 0.99;

//need to tune parameters at a time because otherwise too many chromosomes
//for ga to be effective
pub const INDICES_TO_TUNE: &'static [usize] = &[10];

pub const PAWN_VALUE_IDX: usize = 0;
pub const KNIGHT_VALUE_IDX: usize = 1;
pub const BISHOP_VALUE_IDX: usize = 2;
pub const ROOK_VALUE_IDX: usize = 3;
pub const QUEEN_VALUE_IDX: usize = 4;
pub const PAWN_SAME_SIDE_TABLE_IDX: usize = 5;
pub const PAWN_OTHER_SIDE_TABLE_IDX: usize = 6;
pub const KNIGHT_TABLE_IDX: usize = 7;
pub const BISHOP_TABLE_IDX: usize = 8;
pub const ROOK_TABLE_IDX: usize = 9;
pub const QUEEN_TABLE_IDX: usize = 10;
pub const KING_TABLE_IDX: usize = 11;
pub const BISHOP_PAIR_IDX: usize = 12;
pub const ROOK_OPEN_FILE_IDX: usize = 13;
pub const ROOK_SEMI_OPEN_FILE_IDX: usize = 14;
pub const KING_SHIELD_BONUS_IDX: usize = 15;
pub const KING_OPEN_FILE_PENALTY_IDX: usize = 16;
pub const KING_SEMI_OPEN_FILE_PENALTY_IDX: usize = 17;
pub const KING_VIRTUAL_MOBILITY_IDX: usize = 18;
pub const BISHOP_MOBILITY_SCORE_IDX: usize = 19;
pub const ROOK_MOBILITY_SCORE_IDX: usize = 20;
pub const QUEEN_MOBILITY_SCORE_IDX: usize = 21;
pub const KNIGHT_MOBILITY_SCORE_IDX: usize = 22;
pub const PASSED_PAWN_BONUS_IDX: usize = 23;
pub const ISOLATED_PAWN_PENALTY_IDX: usize = 24;
pub const DOUBLED_PAWN_PENALTY_IDX: usize = 25;
pub const TEMPO_WEIGHT_IDX: usize = 26;
pub const ROOK_ON_SEVENTH_IDX: usize = 27;

#[rustfmt::skip]
fn init_weights() -> Vec<Vec<(i32, i32)>> {

    vec![
        // PAWN_VALUE_IDX = 0
        vec![(145, 170)],
        // KNIGHT_VALUE_IDX = 1
        vec![(538, 515)],
        // BISHOP_VALUE_IDX = 2
        vec![(585, 543)],
        // ROOK_VALUE_IDX = 3
        vec![(713, 879)],
        // QUEEN_VALUE_IDX = 4
        vec![(1380, 1649)],
        // PAWN_SAME_SIDE_TABLE_IDX = 5
vec![
    (30, 30), (38, 48), (0, 2), (-10, -13), (10, 8), (18, 21), (-29, -38), (-10, -10),
    (219, 254), (22, 22), (152, 257), (168, 187), (93, 100), (214, 244), (73, 138), (44, 228),
    (22, 64), (65, 64), (40, 35), (34, 33), (46, 51), (127, 87), (44, 51), (62, 63),
    (22, 23), (1, 3), (8, 6), (17, 16), (53, 12), (34, 25), (31, 35), (13, 15),
    (-9, -8), (-24, -2), (-2, -1), (12, 14), (26, -6), (25, 8), (9, 10), (-4, -3),
    (-4, -2), (-19, -6), (-8, -2), (-31, -9), (16, 14), (13, 16), (40, 8), (17, 2),
    (-5, -6), (1, 2), (4, 3), (-15, -14), (-8, -8), (14, 14), (41, 5), (2, -12),
    (0, 2), (-26, -23), (-21, -24), (-5, -7), (-21, -18), (0, -4), (19, 16), (-3, -4),
],

// PAWN_OTHER_SIDE_TABLE_IDX = 6
vec![
    (27, 24), (32, 31), (-3, -4), (-10, -9), (21, 21), (16, 16), (-34, -29), (-8, -6),
    (172, 185), (15, 24), (152, 173), (177, 167), (66, 68), (80, 79), (84, 75), (49, 60),
    (26, 64), (42, 35), (42, 35), (44, 38), (62, 52), (75, 84), (26, 24), (46, 38),
    (12, 9), (3, 5), (6, 6), (32, 7), (47, 12), (26, 22), (19, 21), (8, 8),
    (-24, -7), (-25, -2), (2, 0), (32, 14), (30, 25), (18, 18), (-3, -4), (-4, -6),
    (-15, -15), (-28, -6), (-7, -9), (-2, -1), (8, 11), (15, 14), (8, 13), (2, 3),
    (-23, -10), (-28, -1), (-33, -12), (-20, -22), (-15, -14), (-3, -1), (-2, 3), (-14, -8),
    (4, 2), (-26, -24), (-12, -22), (-3, -5), (-14, -18), (-1, -4), (19, 19), (-6, -6),
],
        // KNIGHT_TABLE_IDX = 6
        vec![
    (-43, -47), (-106, -84), (-10, -11), (-32, -33), (-25, -24), (-13, -11), (-40, -40), (-43, -52),
    (-26, -22), (4, 4), (33, 25), (25, 23), (58, 40), (13, 13), (1, 3), (-11, -10),
    (11, 10), (13, 11), (28, 29), (116, 32), (21, 21), (-6, -5), (72, 49), (-3, -1),
    (4, 3), (-7, -6), (11, 12), (61, 68), (18, 18), (35, 30), (2, 13), (-9, -6),
    (-17, -18), (-5, -6), (13, 11), (3, 16), (21, 20), (17, 16), (-2, 0), (-10, -8),
    (-57, -26), (-10, -12), (-3, -7), (8, 8), (10, 10), (9, -1), (5, 6), (-27, -28),
    (-57, -43), (-22, -22), (-23, -23), (5, 3), (4, 1), (-11, -11), (-19, -20), (-21, -20),
    (-18, -20), (-19, -23), (-10, -10), (-16, -15), (-14, -11), (-14, -13), (-30, -34), (2, 3),
],
        // BISHOP_TABLE_IDX = 7
        vec![
    (-29, -29), (-12, -12), (-10, -14), (-21, -24), (25, 26), (2, 2), (5, 4), (-1, 0),
    (5, 6), (0, 2), (2, 1), (-2, -1), (3, 0), (7, 7), (-1, 1), (-20, -26),
    (3, 5), (-9, -10), (-1, 7), (12, 10), (29, 24), (120, 72), (2, 2), (4, 6),
    (3, 2), (-10, -3), (7, 11), (27, 28), (24, 27), (0, 1), (-11, -7), (-11, -9),
    (-12, -14), (-3, -6), (-16, -17), (26, 30), (10, 11), (-6, -7), (-18, -19), (6, 7),
    (-12, -10), (8, 9), (0, 2), (2, 4), (-3, -1), (3, 4), (3, 0), (3, 2),
    (5, 6), (7, -6), (16, 5), (-20, 1), (-3, -3), (-2, 0), (25, 9), (4, 6),
    (-6, -7), (-1, -2), (-20, -21), (-3, -3), (-9, -7), (-40, -11), (-12, -9), (-15, -14),
],
        // ROOK_TABLE_IDX = 8
        vec![(0, 4), (13, 9), (19, 18), (30, 34), (30, 30), (-34, -18), (22, 24), (14, 15), (9, 9), (-2, 2), (5, 5), (24, 25), (11, 15), (17, 17), (38, 28), (4, 6), (14, 14), (23, 24), (-7, -6), (9, 10), (31, 36), (23, 28), (47, 39), (27, 27), (-1, 0), (
1, -1), (-1, -2), (4, 4), (2, 0), (17, 17), (17, 18), (17, 18), (-8, -6), (-15, -11), (5, 6), (3, 3), (-16, -11), (-14, -
11), (3, 3), (-16, -17), (-6, -7), (-13, -15), (-17, -16), (-15, -13), (-6, -8), (-17, -17), (-7, -8), (-21, -14), (-16, 
-16), (-1, 2), (-1, -1), (-4, -5), (-2, -1), (0, -1), (-21, -21), (-22, -19), (-22, -19), (-9, -9), (6, 1), (22, -3), (16
, 2), (-4, -5), (-1, 0), (-37, -19)],
        // QUEEN_TABLE_IDX = 9
        vec![(-3, -4), (6, 3), (-5, 0), (25, 29), (12, 18), (12, 12), (28, 26), (12, 8), (2, 4), (-5, -4), (26, 28), (15, 11), (30, 26), (166, 132), (23, 27), (22, 25), (0, 0), (2, -2), (7, 10), (-4, -3), (79, 88), (138, 156), (188, 169), (63, 71), (-13,
 -11), (-12, -8), (20, 20), (13, 55), (25, 37), (29, 35), (11, 9), (27, 30), (-20, -18), (-10, -11), (-6, 0), (1, 17), (3
, 2), (-2, -1), (10, 16), (9, 9), (5, 2), (-2, -7), (-1, 0), (-7, -5), (-5, -6), (-5, 8), (13, 10), (-8, -5), (-26, -29),
 (-4, -5), (9, -4), (17, 4), (8, 4), (5, 2), (-11, -12), (4, 2), (7, 8), (-15, -13), (-12, -12), (6, 5), (-1, -2), (-24, 
-23), (-5, -4), (-48, -46)],
        // KING_TABLE_IDX = 10
        vec![(-55, -61), (-37, -41), (3, 0), (-6, -4), (-1, -2), (0, 1), (-17, -21), (-56, -64), (-14, -19), (-14, -15), (8, 10), (-9
, -10), (6, 8), (9, 14), (-38, -27), (-5, -7), (-48, -49), (-28, -21), (-19, -14), (-14, -13), (-4, -4), (11, 13), (17, 24), (-17, -15), (-11, -8), (-12, -13), (-21, -14), (9, 17), (57, 64), (22, 22), (-2, -3), (-37, -39), (-27, -26), (-7, -8
), (-7, -1), (2, 7), (-2, 0), (-1, 1), (3, 2), (-23, -25), (-6, -7), (3, 4), (-5, -6), (-7, -5), (-17, -11), (3, 4), (-15
, -18), (-35, -41), (-2, -7), (-2, -4), (-12, -13), (-27, -13), (-36, -17), (-36, -17), (-17, -17), (-25, -29), (-14, -11
), (6, -6), (5, 3), (-76, -26), (-9, -21), (-91, -32), (1, -10), (-17, -27)],
        // BISHOP_PAIR_IDX = 11
        vec![(37, 43)],
        // ROOK_OPEN_FILE_IDX = 12
        vec![(11, 13)],
        // ROOK_SEMI_OPEN_FILE_IDX = 13
        vec![(14, 11)],
        // KING_SHIELD_BONUS_IDX = 14
        vec![(6, 6)],
        // KING_OPEN_FILE_PENALTY_IDX = 15
        vec![(-1, 1)],
        // KING_SEMI_OPEN_FILE_PENALTY_IDX = 16
        vec![(-6, -6)],
        // KING_VIRTUAL_MOBILITY_IDX = 17
        vec![(5, 3), (0, -2), (-1, 0), (-7, 0), (-18, -13), (-15, 30), (-16, 29), (-23, 38), (-33, 41), (-45, 43), (-44, 40), (-68, 43), (-56, 43), (-70, 44), (-86, 46), (-94, 41), (-101, 41), (-75, 37), (-91, 34), (-79, 29), (-114, 28), (-93, 26), (-129
, 23), (-77, 9), (-114, -8), (-124, -26), (-145, -32), (-108, -34)],
        // BISHOP_MOBILITY_SCORE_IDX = 18
        vec![(-37, -41), (-27, -27), (-13, -19), (-12, -14), (-3, -4), (3, 5), (15, 19), (13, 20), (13, 26), (16, 34), (19, 27), (28,
 32), (27, 24), (3, 3)],
        // ROOK_MOBILITY_SCORE_IDX = 19
        vec![(-40, -42), (-22, -22), (-19, -22), (-9, -14), (-10, -9), (0, 1), (6, 4), (15, 16), (27, 20), (30, 33), (38, 39), (39, 39), (46, 47), (56, 63), (44, 51)],
        // QUEEN_MOBILITY_SCORE_IDX = 20
        vec![(-33, -46), (-25, -27), (-26, -22), (-24, -43), (-22, -26), (-15, -29), (-15, -14), (-10, -14), (-11, -9), (-6, -7), (-6
, -9), (-5, -4), (-4, -1), (-1, -2), (1, 10), (7, 5), (7, 15), (1, 15), (12, 15), (2, 2), (15, 19), (3, 2), (2, 3), (-2, 
1), (-33, -27), (-18, -17), (-1, -1), (-56, -57)],
        // KNIGHT_MOBILITY_SCORE_IDX = 21
        vec![(-62, -49), (-20, -18), (-4, -10), (3, -5), (13, 6), (12, 12), (14, 19), (24, 27), (28, 28)],
        // PASSED_PAWN_BONUS_IDX = 22
        vec![(18, 17), (0, 0), (-9, -1), (2, 6), (1, 0), (3, 14), (4, 9), (12, 47), (29, 34), (57, 69), (69, 58), (107, 137), (26, 29
), (197, 331), (8, 9), (-12, -11)],
        // ISOLATED_PAWN_PENALTY_IDX = 23
        vec![(-31, -24)],
        // DOUBLED_PAWN_PENALTY_IDX = 24
        vec![(-10, -12)],
        //TEMPO_IDX + 25
        vec![(27, 14)],
        //ROOK_ON_SEVENTH_IDX = 26
        vec![(41, 34)],
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

    let king_kside = lsfb(match colour {
        Colour::White => b.bitboards[WK],
        Colour::Black => b.bitboards[BK],
    }) % 8
        > 3;

    while temp_pawns > 0 {
        let square = lsfb(temp_pawns);
        let is_kside = square % 8 > 3;

        #[allow(non_snake_case)]
        let PAWN_TABLE_IDX = if is_kside == king_kside {
            PAWN_SAME_SIDE_TABLE_IDX
        } else {
            PAWN_OTHER_SIDE_TABLE_IDX
        };
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

        if rank(square) == 6 && colour == Colour::White
            || rank(square) == 1 && colour == Colour::Black
        {
            rook_eval += tapered_score(weights[ROOK_ON_SEVENTH_IDX][0], phase_score);
        }
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

    tapered_score(weights[TEMPO_WEIGHT_IDX][0], phase_score)
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

        let mut captures = MoveList::gen_captures(position); //in tuner I don't think we care
                                                             //whether or not its check
        captures.order_moves(position, &Searcher::new(Instant::now()), &NULL_MOVE);

        //let pin_rays = movegen::get_pin_rays(&position);

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

            let (commit, ok) = position.try_move(c /*, &pin_rays*/);

            if !ok {
                if !commit.made_move {
                    continue;
                }
                position.undo_move(c, &commit);
                continue;
            }

            let eval = -self.quiescence_search(position, -beta, -alpha);
            position.undo_move(c, &commit);
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
            let eval = match b.side_to_move {
                Colour::White => self.quiescence_search(&mut b, -INFINITY, INFINITY),
                Colour::Black => -self.quiescence_search(&mut b, -INFINITY, INFINITY),
            };

            let s =
                (*sf_eval as f32).abs().sqrt() * if sf_eval.is_negative() { -1 } else { 1 } as f32;
            let e = (eval as f32).abs().sqrt() * if eval.is_negative() { -1 } else { 1 } as f32;

            total_error += (s - e).abs() as u32;
            //idea here is that the difference between -0.5 and +0.5 is a lot more important than
            //the difference between +5 and +6
        }
        self.cost = total_error; //want to minimise this value
        Ok(())
    }

    fn mutate(&self, start: bool) -> Self {
        let mut n = Self {
            weights: self.weights.clone(),
            cost: 0,
        };

        let p = if start {
            START_MUTATION_RATE
        } else {
            MUTATION_RATE
        };
        //much more mutation for generation 0 to get bigger variation in initial population

        for i in INDICES_TO_TUNE {
            for (j, (v1, v2)) in self.weights[*i].iter().enumerate() {
                let mut rng = rand::thread_rng();

                //tune first / mg parameter
                let r = rng.gen_range(1..=1000);

                if r <= p {
                    let delta = rng.gen_range(5..=15);
                    let change = (*v1 * delta) / 100;

                    let noise = rng.gen_range(-2..=2);

                    let up = rng.gen_bool(0.5);
                    let new_value = if up { *v1 + change } else { *v1 - change } + noise;
                    n.weights[*i][j].0 = new_value;
                }

                //tune second / eg parameter
                let r = rng.gen_range(1..=1000);

                if r <= p {
                    let delta = rng.gen_range(5..=15);
                    let change = (*v2 * delta) / 100;

                    //percentage change
                    let noise = rng.gen_range(-2..=2);

                    let up = rng.gen_bool(0.5);
                    let new_value = if up { *v1 + change } else { *v1 - change } + noise;
                    n.weights[*i][j].1 = new_value;
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

        x.mutate(false)
    }

    #[allow(unused)]
    fn combine_chunks(&self, other: &Self) -> Self {
        let mut x = Self {
            cost: 0,
            weights: self.weights.clone(),
        };

        let mut rng = rand::thread_rng();
        for i in 0..self.weights.len() {
            let b = rng.gen_bool(0.5);
            if b {
                x.weights[i] = other.weights[i].clone();
            }
        }

        x.mutate(false)
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

fn load_data(db_path: &str) -> Result<(Vec<String>, Vec<i32>), Box<dyn std::error::Error>> {
    println!("{} Loading data from .db file", "INFO:".green().bold(),);
    let start = Instant::now();

    let conn = Connection::open(db_path)?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM evaluations", [], |row| row.get(0))?;

    let mut fens = Vec::with_capacity(count as usize);
    let mut evals = Vec::with_capacity(count as usize);

    let mut stmt = conn.prepare("SELECT fen, eval FROM evaluations")?;
    let mut rows = stmt.query([])?;

    let bar = ProgressBar::new(count as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    while let Some(row) = rows.next()? {
        let fen: String = row.get(0)?;
        let eval: f64 = row.get(1)?;

        if eval.abs() > 50.0 {
            //skip mate evals
            continue;
        }

        fens.push(fen);
        evals.push((eval * 100.0) as i32);

        bar.inc(1);
    }

    bar.finish();

    let duration = start.elapsed();
    println!(
        "{} Loaded {} positions in {:.2?}\n",
        "INFO:".green().bold(),
        count,
        duration
    );

    Ok((fens, evals))
}

pub fn genetic_algorithm() -> Result<(), Box<dyn Error>> {
    let (positions, evals) =
        load_data("/Users/seba/rs/Panda/data/2021-07-31-lichess-evaluations-37MM.db")?;

    let positions = positions.iter().map(|x| x.as_str()).collect();

    let start = Individual::new();

    let mut population = vec![start.clone()];
    for _ in 0..POPULATION_SIZE - 1 {
        population.push(start.mutate(true));
    }

    for gen in 0..NUM_GENERATIONS {
        println!(
            "{} Starting generation {} of {}! 🚀",
            "INFO:".green().bold(),
            gen + 1,
            NUM_GENERATIONS
        );
        let (pos_sample, ev_sample) = take_sample(&positions, &evals);
        let mut new_population = population.clone();
        //use elitism to avoid "throwing away" a good solution
        for x in &population {
            let mut rng = rand::thread_rng();
            let n1 = rng.gen_range(0..POPULATION_SIZE);
            let child1 = x.combine(&population[n1 as usize]);

            let n2 = rng.gen_range(0..POPULATION_SIZE);
            let child2 = x.combine(&population[n2 as usize]);

            let n3 = rng.gen_range(0..POPULATION_SIZE);
            let child3 = x.combine(&population[n3 as usize]);
            /*

                        let n4 = rng.gen_range(0..POPULATION_SIZE);
                        let child4 = x.combine_chunks(&population[n4 as usize]);

                        let n5 = rng.gen_range(0..POPULATION_SIZE);
                        let child5 = x.combine_chunks(&population[n5 as usize]);

                        let n6 = rng.gen_range(0..POPULATION_SIZE);
                        let child6 = x.combine_chunks(&population[n6 as usize]);

            */
            let child7 = x.mutate(false);
            let child8 = x.mutate(false);
            let child9 = x.mutate(false);

            new_population.extend(vec![
                child1, child2, child3, /*child4, child5, child6,*/ child7, child8, child9,
            ]);
        }

        let bar = ProgressBar::new(POPULATION_SIZE as u64 * 7);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );

        new_population.par_iter_mut().for_each(|x| {
            let _ = x.get_cost(&pos_sample, &ev_sample);
            bar.inc(1);
        });
        bar.finish();
        new_population.sort_by_key(|x| x.cost); //ascending sort (which is what we want)
        population = new_population[..POPULATION_SIZE as usize].to_vec();

        println!(
            "{} Generation {} of {}: average cost {}! \n",
            "INFO:".green().bold(),
            gen + 1,
            NUM_GENERATIONS,
            (population[0].cost as f32 / pos_sample.len() as f32)
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

//I don't think that simulated annealing is that good because it's almost impossible to get stuck
//in local minima, at which point annealing isn't likely to be useful
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

    let bar = ProgressBar::new(MAX_ITERATIONS as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let mut old = Individual::new();
    let mut constant = 0;
    let mut iterations = 0;
    while constant < MAX_CONSTANT && iterations < MAX_ITERATIONS {
        bar.inc(1);
        iterations += 1;
        let (pos_sample, ev_sample) = take_sample(&positions, &evals);
        let mut new = old.mutate(false);

        old.get_cost(&pos_sample, &ev_sample)?;
        new.get_cost(&pos_sample, &ev_sample)?;

        let s_old = 1.0;
        let s_new = new.cost as f32 / old.cost as f32;
        let delta_e = s_new / s_old;

        if delta_e < 1.0 {
            println!(
                "{} Iteration {}: Accepted cost {} vs old cost {}",
                "INFO:".green().bold(),
                iterations,
                new.cost as f32 / pos_sample.len() as f32,
                old.cost as f32 / pos_sample.len() as f32,
            );
            old = new;
            constant = 0;
        } else {
            let mut rng = rand::thread_rng();
            let x: f32 = rng.gen();
            let p = acceptance_probability(delta_e, temp);

            if x <= p {
                println!(
                    "{} Iteration {}: Accepted cost {} vs old cost {}",
                    "INFO:".green().bold(),
                    iterations,
                    new.cost as f32 / pos_sample.len() as f32,
                    old.cost as f32 / pos_sample.len() as f32,
                );
                old = new;
                constant = 0;
            } else {
                constant += 1;
            }
        }
        temp *= K;
    }
    bar.finish();

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

    let bar = ProgressBar::new(MAX_ITERATIONS as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let mut old = Individual::new();
    let mut constant = 0;
    let mut iterations = 0;
    while constant < MAX_CONSTANT && iterations < MAX_ITERATIONS {
        bar.inc(1);
        iterations += 1;
        let (pos_sample, ev_sample) = take_sample(&positions, &evals);
        let mut new = old.mutate(false);

        old.get_cost(&pos_sample, &ev_sample)?;
        new.get_cost(&pos_sample, &ev_sample)?;

        if new.cost < old.cost {
            println!(
                "{} Iteration {}: Accepted cost {} vs old cost {}",
                "INFO:".green().bold(),
                iterations,
                new.cost as f32 / pos_sample.len() as f32,
                old.cost as f32 / pos_sample.len() as f32,
            );
            old = new;
            constant = 0;
        } else {
            constant += 1;
        }
    }

    bar.finish();

    for w in old.clone().weights {
        println!("{:?}", w);
    }
    Ok(())
}
