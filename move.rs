/*
*
* represent move as u32 with 24 bits used          hex constant
* 0000 0000 0000 0000 0011 1111 square from        0x3f
* 0000 0000 0000 1111 1100 0000 square to          0xfc0
* 0000 0000 1111 0000 0000 0000 piece moved        0xf000
* 0000 1111 0000 0000 0000 0000 promoted piece     0xf0000
* 0001 0000 0000 0000 0000 0000 capture flag       0x100000
* 0010 0000 0000 0000 0000 0000 pawn by 2 flag     0x200000
* 0100 0000 0000 0000 0000 0000 en passant flag    0x400000
* 1000 0000 0000 0000 0000 0000 castling flag      0x800000
*
*/

use crate::board::*;
use crate::helper::*;

pub const NULL_MOVE: Move = Move { data: 0u32 };
pub const MAX_MOVES: usize = 218;

pub struct MoveList {
    pub moves: [Move; MAX_MOVES], //max number of legal moves in a chess position
}

//#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub struct Move {
    pub data: u32,
}

#[allow(clippy::too_many_arguments)]
impl Move {
    pub fn from(
        sq_from: usize,
        sq_to: usize,
        piece_moved: usize,
        promoted_piece: usize,
        capture: bool,
        double_push: bool,
        en_passant: bool,
        castling: bool,
    ) -> Self {
        Move {
            data: (sq_from
                | sq_to << 6
                | piece_moved << 12
                | promoted_piece << 16
                | match capture {
                    false => 0,
                    true => 1 << 20,
                }
                | match double_push {
                    false => 0,
                    true => 1 << 21,
                }
                | match en_passant {
                    false => 0,
                    true => 1 << 22,
                }
                | match castling {
                    false => 0,
                    true => 1 << 23,
                }) as u32,
        }
    }

    pub fn square_from(&self) -> usize {
        (self.data & 0x3f) as usize
    }

    pub fn square_to(&self) -> usize {
        ((self.data & 0xfc0) >> 6) as usize
    }

    pub fn piece_moved(&self) -> usize {
        ((self.data & 0xf000) >> 12) as usize
    }

    pub fn promoted_piece(&self) -> usize {
        ((self.data & 0xf0000) >> 16) as usize
    }

    pub fn is_capture(&self) -> bool {
        self.data & 0x100000 > 0
    }

    pub fn is_double_push(&self) -> bool {
        self.data & 0x200000 > 0
    }

    pub fn is_en_passant(&self) -> bool {
        self.data & 0x400000 > 0
    }

    pub fn is_castling(&self) -> bool {
        self.data & 0x800000 > 0
    }

    pub fn print_move(&self) {
        println!(
            "{}{}",
            coordinate(self.square_from()),
            coordinate(self.square_to())
        );
        println!(
            "moved {}",
            match self.piece_moved() {
                0 => "white pawn",
                1 => "white knight",
                2 => "white bishop",
                3 => "white rook",
                4 => "white queen",
                5 => "white king",
                6 => "black pawn",
                7 => "black knight",
                8 => "black bishop",
                9 => "black rook",
                10 => "black queen",
                11 => "black king",
                _ => "impossible",
            }
        );

        println!(
            "promoted to {}",
            match self.promoted_piece() {
                0 => "white knight",
                1 => "white bishop",
                2 => "white rook",
                3 => "white queen",
                4 => "black knight",
                5 => "black bishop",
                6 => "black rook",
                7 => "black queen",
                15 => "NONE",
                _ => "impossible",
            }
        );

        println!("capture: {}", self.is_capture());
        println!("double push: {}", self.is_double_push());
        println!("en passant: {}", self.is_en_passant());
        println!("castling: {}", self.is_castling());
    }
}

pub fn encode_move(
    sq_from: usize,
    sq_to: usize,
    promoted_piece: usize,
    board: &Board,
    castling: bool,
) -> Move {
    let mut piece = 0;
    for i in 0..12 {
        if get_bit(sq_from, board.bitboards[i]) > 0 {
            piece = i;
            break;
        }
    }

    let capture = get_bit(sq_to, board.occupancies[2]) == 1;
    let double_push: bool =
        piece == 0 && (sq_from - 16 == sq_to) || piece == 6 && (sq_from + 16 == sq_to);

    let ep = sq_to == board.en_passant;
    Move::from(
        sq_from,
        sq_to,
        piece,
        promoted_piece,
        capture,
        double_push,
        ep,
        castling,
    )
}
