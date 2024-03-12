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
use crate::movegen::is_attacked;

pub const NULL_MOVE: Move = Move { data: 0u32 };
pub const MAX_MOVES: usize = 218;

#[derive(Debug)]
pub struct MoveList {
    pub moves: [Move; MAX_MOVES], //max number of legal moves in a chess position
}

#[derive(PartialEq, Clone, Copy, Debug)]
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
        piece == 0 && (sq_from + 16 == sq_to) || piece == 6 && (sq_from - 16 == sq_to);

    let ep = (sq_to == board.en_passant) && (piece % 6 == 0);
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

pub fn make_move(m: Move, b: Board) -> Board {
    let mut updated_board = b;
    let sq_from = m.square_from();
    let sq_to = m.square_to();
    let piece = m.piece_moved();
    if m.is_capture() {
        //remove captured piece from bitboard
        for i in 0..12 {
            if get_bit(sq_to, b.bitboards[i]) > 0 {
                updated_board.bitboards[i] = pop_bit(sq_to, b.bitboards[i]);
                break;
            }
        }
        match m.square_to() { //remove castling rights is a1/h1/a8/h8 is captured on
            0 => updated_board.castling &= 0b0000_1101,
            7 => updated_board.castling &= 0b0000_1110,
            56 => updated_board.castling &= 0b0000_0111,
            63 => updated_board.castling &= 0b0000_1011,
            _ => {},
        }
    } else if m.is_en_passant() {
        match piece {
            0 => updated_board.bitboards[6] = pop_bit(sq_to - 8, updated_board.bitboards[6]),
            6 => updated_board.bitboards[0] = pop_bit(sq_to + 8, updated_board.bitboards[0]),
            _ => panic!("non-pawn is capturing en passant 🤔"),
        }
    }

    if m.promoted_piece() != 15 {
        updated_board.bitboards[piece] = pop_bit(sq_from, updated_board.bitboards[piece]);
        updated_board.bitboards[m.promoted_piece()] =
            set_bit(sq_to, updated_board.bitboards[m.promoted_piece()]);
        //remove pawn and add promoted piece
    } else if m.is_castling() {
        //update king and rook for castling
        match sq_to {
            2 => {
                updated_board.bitboards[5] = set_bit(2, 0); //works bc only 1 wk
                updated_board.bitboards[3] = pop_bit(0, updated_board.bitboards[3]);
                updated_board.bitboards[3] = set_bit(3, updated_board.bitboards[3])
            }
            6 => {
                updated_board.bitboards[5] = set_bit(6, 0);
                updated_board.bitboards[3] = pop_bit(7, updated_board.bitboards[3]);
                updated_board.bitboards[3] = set_bit(5, updated_board.bitboards[3])
            }
            58 => {
                updated_board.bitboards[11] = set_bit(58, 0);
                updated_board.bitboards[9] = pop_bit(56, updated_board.bitboards[9]);
                updated_board.bitboards[9] = set_bit(59, updated_board.bitboards[9])
            }
            62 => {
                updated_board.bitboards[11] = set_bit(62, 0);
                updated_board.bitboards[9] = pop_bit(63, updated_board.bitboards[9]);
                updated_board.bitboards[9] = set_bit(61, updated_board.bitboards[9])
            }
            _ => panic!("castling to a square that is not c1 g1 c8 or g8 🤔"),
        }
    } else {
        updated_board.bitboards[piece] = pop_bit(sq_from, updated_board.bitboards[piece]); //pop bit from bitboard
        updated_board.bitboards[piece] = set_bit(sq_to, updated_board.bitboards[piece]); //set new bit on bitboard
    }

    updated_board.occupancies[0] = updated_board.bitboards[0]
            | updated_board.bitboards[1]
            | updated_board.bitboards[2]
            | updated_board.bitboards[3]
            | updated_board.bitboards[4]
            | updated_board.bitboards[5];

        updated_board.occupancies[1] = updated_board.bitboards[6]
            | updated_board.bitboards[7]
            | updated_board.bitboards[8]
            | updated_board.bitboards[9]
            | updated_board.bitboards[10]
            | updated_board.bitboards[11];

        updated_board.occupancies[2] = updated_board.occupancies[0] | updated_board.occupancies[1];
        //update occupancies

    if m.is_double_push() {
        updated_board.en_passant = match piece {
            0 => sq_from + 8,
            6 => sq_from - 8,
            _ => panic!("non-pawn is making a double push 🤔"),
        }
    } else {
        updated_board.en_passant = 64;
    }

    if (piece % 6 == 0) || m.is_capture() {
        updated_board.fifty_move = 0;
    } else {
        updated_board.fifty_move += 1;
    }

    if piece == 5 {
        updated_board.castling &= 0b0000_1100;
    } else if piece == 11 {
        updated_board.castling &= 0b0000_0011;
    }

    if piece == 3 && sq_from == 0 {
        updated_board.castling &= 0b0000_1101;
    } else if piece == 3 && sq_from == 7 {
        updated_board.castling &= 0b0000_1110;
    } else if piece == 9 && sq_from == 56 {
        updated_board.castling &= 0b0000_0111;
    } else if piece == 9 && sq_from == 63 {
        updated_board.castling &= 0b0000_1011;
    } //update castling rights

    updated_board.ply += 1;

    updated_board.side_to_move = match b.side_to_move {
        Colour::White => Colour::Black,
        Colour::Black => Colour::White,
    };

    updated_board
}

pub fn is_legal(m: Move, b: &Board) -> bool {
    let updated_board = make_move(m, *b);
    match updated_board.side_to_move {
        // AFTER move has been made
        Colour::White => !is_attacked(lsfb(updated_board.bitboards[11]).unwrap(), Colour::White, &updated_board),
        Colour::Black => !is_attacked(lsfb(updated_board.bitboards[5]).unwrap(), Colour::Black, &updated_board),
    }
}
