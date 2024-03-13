use crate::board::*;
use crate::magic::*;
use crate::helper::*;
use crate::is_attacked;
use crate::search::INFINITY;

const PAWN_VALUE: i32 = 95;
const KNIGHT_VALUE: i32 = 310;
const BISHOP_VALUE: i32 = 325;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

#[rustfmt::skip]
const WP_TABLE: [i32; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
    0,  10,  10, -20, -20,  15,  10,   0,
    5,  -5, -10,   0,   0, -15,  -5,   5,
    0,   0,   0,  15,  15,   0,   0,   0,
    5,   5,  10,  25,  25,  10,   5,  15,
   10,  10,  20,  40,  40,  20,  10,  20,
    0,  60,  60,  70,  70,  60,  60,  60,
    0,   0,   0,   0,   0,   0,   0,   0
];

#[rustfmt::skip]
const WN_TABLE: [i32; 64] = [
    -25, -20, -15, -15, -15, -15, -20, -25,
    -15, -20,   0  , 0,   0,   0, -20, -15,
    -10,   0,  10,   5,   5,  10,   0, -10,
    -10,   5,  15,  20,  20,  15,   5, -10,
     -5,   0,  15,  20,  20,  15,   0,  -5,
    -15,   5,  20,  25,  25,  20,   5, -15,
    -20, -20,   0,   5,   5,   0, -20, -20,
    -50, -40, -30, -30, -30, -30, -40, -50
];

#[rustfmt::skip]
const WB_TABLE: [i32; 64] = [
    -20, -10, -15, -10, -10, -15, -10, -20,
    -10,  15,   0,   0,   0,   0,  15, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
     10,   0,  15,  10,  10,  10,   0, -10,
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
    -20, -10, -10, -5, -5, -10, -10, -20,
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
   10, 10, 20, 40, 40, 20, 10, 20,
    5,  5, 10, 25, 25, 10,  5, 15,
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
    -10,   0,  10,   5,   5,  10,   0, -10,
    -15, -20,   0  , 0,   0,   0, -20, -15,
    -25, -20, -15, -15, -15, -15, -20, -25
];

#[rustfmt::skip]
const BB_TABLE: [i32; 64] = [
    -30, -30, -30, -30, -30, -30, -30, -30,
    -20, -10,  -5,  -5,  -5,  -5, -10, -20,
    -10,   0,   5,   0,   0,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,  10, -10,
    -10,   0,  15,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,  15,   0,   0,   0,   0,  15, -10,
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
    -20, -10, -10, -5, -5, -10, -10, -20
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

const BISHOP_BASE_MOBILITY: i32 = 4;
const ROOK_BASE_MOBILITY: i32 = 2;
const QUEEN_BASE_MOBILITY: i32 = 9;

const BISHOP_MOBILITY_UNIT: i32 = 4;
const ROOK_MOBILITY_UNIT: i32 = 3;
const QUEEN_MOBILITY_UNIT: i32 = 1;

const START_MATERIAL: i32 = PAWN_VALUE * 16 + KNIGHT_VALUE * 4 + BISHOP_VALUE * 4 + ROOK_VALUE * 4 + QUEEN_VALUE * 2;
//possible for promotions to in theory result in more material than this

pub fn game_phase_score(material_count: i32) -> f32 {
    material_count as f32 / START_MATERIAL as f32
}

pub fn evaluate(b: &Board) -> i32 {
    //TODO: pawn structure evaluation
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
                0 => eval += WP_TABLE[square],
                1 => eval += WN_TABLE[square],
                2 => {
                    eval += WB_TABLE[square];
                    eval += (count(get_bishop_attacks(square, b.occupancies[2])) as i32 - BISHOP_BASE_MOBILITY) as i32 * BISHOP_MOBILITY_UNIT;
                }
                3 => {
                    eval += WR_TABLE[square];
                    eval += (count(get_rook_attacks(square, b.occupancies[2])) as i32 - ROOK_BASE_MOBILITY) as i32 * ROOK_MOBILITY_UNIT;
                }
                4 => {
                    eval += WQ_TABLE[square];
                    eval += std::cmp::max((count(get_queen_attacks(square, b.occupancies[2])) as i32 - QUEEN_BASE_MOBILITY) as i32, 0) * QUEEN_MOBILITY_UNIT;
                }
                5 => eval += {
                    ((WK_TABLE[square] as f32 * phase_score + WK_ENDGAME[square] as f32 * (1f32 - phase_score)) / 2f32) as i32
                },

                6 => eval -= BP_TABLE[square],
                7 => eval -= BN_TABLE[square],
                8 => {
                    eval -= BB_TABLE[square];
                    eval -= (count(get_bishop_attacks(square, b.occupancies[2])) as i32 - BISHOP_BASE_MOBILITY) as i32 * BISHOP_MOBILITY_UNIT;
                }
                9 => {
                    eval -= BR_TABLE[square];
                    eval -= (count(get_rook_attacks(square, b.occupancies[2])) as i32 - ROOK_BASE_MOBILITY) as i32 * ROOK_MOBILITY_UNIT;
                }
                10 => {
                    eval -= BQ_TABLE[square];
                    eval -= std::cmp::max((count(get_queen_attacks(square, b.occupancies[2])) as i32 - QUEEN_BASE_MOBILITY) as i32, 0) * QUEEN_MOBILITY_UNIT;
                }
                11 => eval -= {
                    ((BK_TABLE[square] as f32 * phase_score + BK_ENDGAME[square] as f32 * (1f32 - phase_score)) / 2f32) as i32
                },
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
