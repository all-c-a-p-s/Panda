use crate::board::*;
use crate::helper::*;
use crate::magic::*;

pub const PAWN_VALUE: (i32, i32) = (83, 95);
pub const KNIGHT_VALUE: (i32, i32) = (306, 311);
pub const BISHOP_VALUE: (i32, i32) = (322, 350);
pub const ROOK_VALUE: (i32, i32) = (490, 542);
pub const QUEEN_VALUE: (i32, i32) = (925, 940);

//all PSQT have a1 on bottom left as viewing the code
//currently picked pretty arbitrarily

//meant to give idea that more central pawns are more valuable in mg
#[rustfmt::skip]
const PAWN_TABLE: [(i32, i32); 64] = [
    (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),
  (11,132), (78,110), (69,103),  (91,95), (80, 90),  (95,93), (79,108),  (8,125),
  (-1, 63),  (11,53),  (13,51),  (39,43), (35, 42),  (25,48),  (11,50), (-5, 59),
  (-5, 17),   (9, 8),   (3, 6),  (22, 4),  (18, 3),   (8, 4),   (3, 7), (-8, 16),
   (-9, 4), (-5, -4),   (8, 2),  (14,-2),  (11,-3),  (-2,-5),  (-8,-4),  (-9, 4),
   (-5, 3),  (-3,-2),  (1, -2),   (5,-2),   (4, 0),  (-5,-2),  (-3, 1), (-10, 3),
   (-8, 3),   (1, 4),  (-3, 4), (-11, 5),  (-9, 6),  (11, 5),   (9, 4),  (-6, 3),
    (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),
];

#[rustfmt::skip]
const KNIGHT_TABLE: [(i32, i32); 64] = [
    (-70,-35),(-51,-23),(-41,-15),(-26,-12),(-24,-12),(-37,-14),(-49,-21),(-60,-31),
    (-22,-14),(-19,-12),  (20,-8),  (22,-5),  (23,-5),  (28,-7),  (-6,-6),(-19,-13),
      (-5,-7),   (3,-1),  (17, 3),  (28, 8),  (34, 8),  (42, 5),  (30,-1),  (8,-10),
      (-4,-5),   (2, 0),  (10, 7),  (15,12),  (14,11),  (16, 7),   (9, 0),  (-4,-7),
      (-5,-6),  (-1, 0),   (9, 6),  (10, 8),  (11, 8),   (8, 6),  (-2, 0),  (-7,-9),
      (-8,-6),   (3, 2),   (9, 2),   (2, 1),   (4, 3),  (10, 3),   (4, 1), (-9,-11),
     (-10,-6), (-10,-8),  (-7,-2),  (-1,-3),  (-2,-3), (-12,-9), (-10,-8), (-9,-10),
     (-22,-7),(-10,-10), (-11,-6), (-10,-5),  (-6,-4),  (-6,-5),(-12,-12), (-9,-14),
];

#[rustfmt::skip]
const BISHOP_TABLE: [(i32, i32); 64] = [
    (-15,-7), (-8,-10),(-18,-10), (-14,-9), (-14,-9),(-10,-10), (-8,-14), (-15,-8),
    (-15,-8), (-10,-4),  (-4,-2), (-3, -3),  (-2,-3),  (40,-6), (-10,-6), (-18,-8),
     (-5, 0),  (10, 1),  (20, 3),  (32, 3),  (30, 3),   (39,4),  (13, 1),   (1, 0),
      (0,-2),   (4, 1),  (12, 3),  (26,10),  (23,10),   (14,6),   (8, 0),   (0,-2),
    (-1, -3),   (3, 1),  (11, 3),  (18, 8),  (20, 8),   (9, 3),   (2, 1),   (2,-3),
     (2, -4),   (9, 0),   (7, 0),   (7, 3),   (4, 3),   (8, 1),   (6, 2),  (-2,-4),
     (-8,-5),  (10, 0),   (1, 0),   (0, 0),   (2, 1),   (0,-2),  (16, 4),   (1,-4),
     (-7,-5),  (-8,-6),  (-4,-4), (-8, -6), (-8, -6),  (-5,-4),  (-9,-6),  (-5,-3),
];

#[rustfmt::skip]
const ROOK_TABLE: [(i32, i32); 64] = [
    (18, 5),  (20,5), (24,6),  (26, 6),  (29, 6),  (25, 5),  (20, 4),  (17, 3),
    (18, 8), (20, 8),(25,10),  (35,13), (44, 10),   (71,8),  (34, 6),  (17, 6),
     (0, 3),  (4, 1), (5, 4),  (20, 7),  (20, 6),  (11, 4),  (10, 1),  (-4, 3),
   (-10, 1),(-11, 0),(-10,2),  (14, 4),  (12, 4),  (10, 2),   (2, 0), (-12, 1),
   (-15,-2),(-19,-4),(-9, 1),  (-8, 3),  (-8, 3),  (-6, 1), (-6, -4), (-10,-5),
   (-14,-3),(-10,-2),(-11,0),  (-8, 0),  (-6, 0),  (0, -4),  (-6,-3), (-16,-6),
   (-30,-5),(-16, 0),(-8, 1),   (0, 1),   (0, 1),   (0, 0), (-10,-7), (-36,-1),
   (-6, -7),(-10,-5), (2, 0),   (9, 0),   (8, 0),  (3, -1), (-12,-5),  (-6,-7),
];

#[rustfmt::skip]
const QUEEN_TABLE: [(i32, i32); 64] = [
   (-23,-5),   (3,10), (10, 10),  (10,16), (36, 13),  (36,10),   (38,9),  (40, 3),
   (-18,-7), (-23,10), (-4, 14),   (1,20), (-3, 19),  (42,17),  (24,15),   (40,4),
   (-10,-4), (-12, 0),  (3, 10),   (4,28), (16, 25), (40, 13),  (35,10),  (42, 8),
   (-14, 1), (-14, 6),  (-8,18),  (-6,36),  (-2,34),  (10,23),  (-2,20),  (1, 18),
    (-2,-7), (-12,14),  (-3,14),  (-5,26),  (-5,24),  (-4,15),  (-6,12),  (-7,10),
   (-8,-10),   (3,-6),   (1, 3),   (1, 3),  (-1, 5),   (1, 6),   (5, 5),   (3, 3),
  (-13,-11),  (-4,-8),   (8,-5),   (3,-8),   (7,-5),   (0,-8), (-2,-10), (-2,-11),
   (-1,-15),(-10,-13), (-6,-11),  (6,-20),(-13,-10),(-14,-16),(-15,-10),(-25,-20),
];

#[rustfmt::skip]
const KING_TABLE: [(i32, i32); 64] = [
    (-40,-55), (-20,-30),(-21,-20),(-16,-20),(-35,-16), (-34,-6),(-20,-12),(-17,-49),
    (-25,-15),  (-31,12), (-34,14), (-42,15), (-41,16), (-30,22), (-30,15), (-29,-2),
     (-30, 4), (-32, 15),(-30, 14), (-39,15), (-40,15), (-37,19), (-33,15),(-30, -5),
    (-26,-11), (-15, 12), (-19,11),(-32, 12),(-30, 12), (-26,16), (-20,12), (-25, 2),
    (-21,-18),  (-12, 9), (-15, 8), (-28, 9), (-26, 9),(-22, 13), (-18, 9),(-15,-10),
    (-15,-19),  (-11,-1), (-14,-2), (-25,-1), (-22,-1), (-20, 3),(-10, -1),(-14,-12),
     (-3,-35),   (5,-12),  (2,-14),(-25,-12),(-20,-12), (-14,-8),  (8,-12),  (7,-30),
    (-10,-51),  (20,-36), (15,-20),(-23,-16),  (0,-24),(-18,-20), (21,-24),  (8,-45),
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

pub const FILES: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        table[square] = set_file(square);
        square += 1;
    }
    table
};

const MIRROR: [usize; 64] = {
    let mut mirror = [0usize; 64];
    let mut square = 0;
    while square < 64 {
        mirror[square] = relative_psqt_square(square, Colour::White);
        square += 1;
    }
    mirror
};

//mobility scores worse than these give negative bonus
const BISHOP_BASE_MOBILITY: i32 = 4;
const ROOK_BASE_MOBILITY: i32 = 2;
const QUEEN_BASE_MOBILITY: i32 = 3;

const BISHOP_MOBILITY_UNIT: (i32, i32) = (2, 2);
const ROOK_MOBILITY_UNIT: (i32, i32) = (1, 2);
const QUEEN_MOBILITY_UNIT: (i32, i32) = (1, 1);

const BISHOP_PAIR: (i32, i32) = (21, 32);
const ROOK_OPEN_FILE: (i32, i32) = (22, 9);
const ROOK_SEMI_OPEN_FILE: (i32, i32) = (10, 6);

const KING_SHIELD_BONUS: (i32, i32) = (6, 1);
const KING_OPEN_FILE_PENALTY: (i32, i32) = (-12, -4);
const KING_SEMI_OPEN_FILE_PENALTY: (i32, i32) = (-6, -1);
const KING_VIRTUAL_MOBILITY: (i32, i32) = (-2, 0);

const START_PHASE_SCORE: i32 = 12;
//possible for promotions to in theory result in more material than this

// bonus[rank][can advance]
const PASSED_PAWN_BONUS: [[(i32, i32); 2]; 8] = [
    [(0, 0), (0, 0)],
    [(-30, -4), (5, 25)],
    [(-20, 6), (9, 28)],
    [(-5, 25), (20, 43)],
    [(10, 46), (43, 60)],
    [(42, 50), (62, 92)],
    [(62, 55), (90, 110)],
    [(0, 0), (0, 0)],
];

const ISOLATED_PAWN_PENALTY: (i32, i32) = (-2, -16);
const DOUBLED_PAWN_PENALTY: (i32, i32) = (-5, -22); //only given to the first pawn

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
        bishop_eval += (count(attacks) as i32 - BISHOP_BASE_MOBILITY)
            * tapered_score(BISHOP_MOBILITY_UNIT, phase_score);
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

fn above_rank(square: usize, c: Colour) -> u64 {
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
        let attacks = get_rook_attacks(square, b.occupancies[BOTH])
            & !b.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }];
        let attacks_up_file = attacks & above_rank(square, colour);
        let mut open_file = false;
        let mut semi_open_file = false;
        rook_eval += (count(attacks) as i32 - ROOK_BASE_MOBILITY)
            * tapered_score(ROOK_MOBILITY_UNIT, phase_score);
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
        queen_eval += (count(attacks) as i32 - QUEEN_BASE_MOBILITY)
            * tapered_score(QUEEN_MOBILITY_UNIT, phase_score);
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
    safety_score += count(attacks) as i32 * tapered_score(KING_VIRTUAL_MOBILITY, phase_score);

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
    /*
        println!(
            "pawn {}",
            evaluate_pawns(b, phase_score, Colour::White)
                - evaluate_pawns(b, phase_score, Colour::Black)
        );
        println!(
            "knight {}",
            evaluate_knights(b, phase_score, Colour::White)
                - evaluate_knights(b, phase_score, Colour::Black)
        );
        println!(
            "bishop {}",
            evaluate_bishops(b, phase_score, Colour::White)
                - evaluate_bishops(b, phase_score, Colour::Black)
        );
        println!(
            "rook {}",
            evaluate_rooks(b, phase_score, Colour::White)
                - evaluate_rooks(b, phase_score, Colour::Black)
        );
        println!(
            "queen {}",
            evaluate_queens(b, phase_score, Colour::White)
                - evaluate_queens(b, phase_score, Colour::Black)
        );
        println!(
            "king {}",
            evaluate_king(b, phase_score, Colour::White) - evaluate_king(b, phase_score, Colour::Black)
        );
    */
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

    match b.side_to_move {
        //return from perspective of side to move
        Colour::White => eval,
        Colour::Black => -eval,
    }
}
