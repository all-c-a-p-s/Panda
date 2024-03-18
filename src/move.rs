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
use crate::INFINITY;

pub const NULL_MOVE: Move = Move {
    data: 0u32,
    move_order_score: -INFINITY,
};
pub const MAX_MOVES: usize = 218;

#[derive(Debug)]
pub struct MoveList {
    pub moves: [Move; MAX_MOVES], //max number of legal moves in a chess position
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Move {
    pub data: u32,
    pub move_order_score: i32,
}

pub struct Commit {
    //resets for irreversible fields of Board struct
    pub castling_reset: u8,
    pub ep_reset: usize,
    pub fifty_move_reset: u8,
    pub piece_captured: Option<u8>,
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
            move_order_score: 0, //update later
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

impl Board {
    pub fn make_move(&mut self, m: Move) -> Commit {
        let mut commit = Commit {
            castling_reset: self.castling,
            ep_reset: self.en_passant,
            fifty_move_reset: self.fifty_move,
            piece_captured: None,
        };
        let sq_from = m.square_from();
        let sq_to = m.square_to();
        let piece = m.piece_moved();
        if m.is_capture() {
            //remove captured piece from bitboard
            for i in 0..12 {
                if get_bit(sq_to, self.bitboards[i]) > 0 {
                    self.bitboards[i] = pop_bit(sq_to, self.bitboards[i]);
                    commit.piece_captured = Some(i as u8);
                    break;
                }
            }
            match m.square_to() {
                //remove castling rights is a1/h1/a8/h8 is captured on
                0 => self.castling &= 0b0000_1101,
                7 => self.castling &= 0b0000_1110,
                56 => self.castling &= 0b0000_0111,
                63 => self.castling &= 0b0000_1011,
                _ => {}
            }
        } else if m.is_en_passant() {
            match piece {
                0 => self.bitboards[6] = pop_bit(sq_to - 8, self.bitboards[6]),
                6 => self.bitboards[0] = pop_bit(sq_to + 8, self.bitboards[0]),
                _ => panic!("non-pawn is capturing en passant 🤔"),
            }
        }

        if m.promoted_piece() != 15 {
            self.bitboards[piece] = pop_bit(sq_from, self.bitboards[piece]);
            self.bitboards[m.promoted_piece()] = set_bit(sq_to, self.bitboards[m.promoted_piece()]);
            //remove pawn and add promoted piece
        } else if m.is_castling() {
            //update king and rook for castling
            match sq_to {
                2 => {
                    self.bitboards[5] = set_bit(2, 0); //works bc only 1 wk
                    self.bitboards[3] = pop_bit(0, self.bitboards[3]);
                    self.bitboards[3] = set_bit(3, self.bitboards[3])
                }
                6 => {
                    self.bitboards[5] = set_bit(6, 0);
                    self.bitboards[3] = pop_bit(7, self.bitboards[3]);
                    self.bitboards[3] = set_bit(5, self.bitboards[3])
                }
                58 => {
                    self.bitboards[11] = set_bit(58, 0);
                    self.bitboards[9] = pop_bit(56, self.bitboards[9]);
                    self.bitboards[9] = set_bit(59, self.bitboards[9])
                }
                62 => {
                    self.bitboards[11] = set_bit(62, 0);
                    self.bitboards[9] = pop_bit(63, self.bitboards[9]);
                    self.bitboards[9] = set_bit(61, self.bitboards[9])
                }
                _ => panic!("castling to a square that is not c1 g1 c8 or g8 🤔"),
            }
        } else {
            self.bitboards[piece] = pop_bit(sq_from, self.bitboards[piece]); //pop bit from bitboard
            self.bitboards[piece] = set_bit(sq_to, self.bitboards[piece]);
            //set new bit on bitboard
        }

        self.occupancies[0] = self.bitboards[0]
            | self.bitboards[1]
            | self.bitboards[2]
            | self.bitboards[3]
            | self.bitboards[4]
            | self.bitboards[5];

        self.occupancies[1] = self.bitboards[6]
            | self.bitboards[7]
            | self.bitboards[8]
            | self.bitboards[9]
            | self.bitboards[10]
            | self.bitboards[11];

        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];
        //update occupancies

        if m.is_double_push() {
            self.en_passant = match piece {
                0 => sq_from + 8,
                6 => sq_from - 8,
                _ => panic!("non-pawn is making a double push 🤔"),
            }
        } else {
            self.en_passant = 64;
        }

        if (piece % 6 == 0) || m.is_capture() {
            self.fifty_move = 0;
        } else {
            self.fifty_move += 1;
        }

        if piece == 5 {
            self.castling &= 0b0000_1100;
        } else if piece == 11 {
            self.castling &= 0b0000_0011;
        }

        if piece == 3 && sq_from == 0 {
            self.castling &= 0b0000_1101;
        } else if piece == 3 && sq_from == 7 {
            self.castling &= 0b0000_1110;
        } else if piece == 9 && sq_from == 56 {
            self.castling &= 0b0000_0111;
        } else if piece == 9 && sq_from == 63 {
            self.castling &= 0b0000_1011;
        } //update castling rights

        self.ply += 1;

        self.side_to_move = match self.side_to_move {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };
        commit
    }

    pub fn undo_move(&mut self, m: Move, c: Commit) {
        //incremental update should be faster than copying the whole board
        self.side_to_move = match self.side_to_move {
            //note updated at the beginning of the function
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };
        if m.promoted_piece() != 15 {
            self.bitboards[m.promoted_piece()] =
                pop_bit(m.square_to(), self.bitboards[m.promoted_piece()]);
            match m.piece_moved() {
                0 => self.bitboards[0] = set_bit(m.square_from(), self.bitboards[0]),
                6 => self.bitboards[6] = set_bit(m.square_from(), self.bitboards[6]),
                _ => panic!("non pawn is promoting lol"),
            }
            //remove promoted piece from bitboard
        } else {
            for i in 0..12 {
                if get_bit(m.square_to(), self.bitboards[i]) == 1 {
                    //piece that was moved to square
                    self.bitboards[i] = pop_bit(m.square_to(), self.bitboards[i]);
                    self.bitboards[i] = set_bit(m.square_from(), self.bitboards[i]);
                    break;
                }
            }
        }

        if c.piece_captured.is_some() {
            //put captured piece back onto bitboard
            self.bitboards[c.piece_captured.unwrap() as usize] = set_bit(
                m.square_to(),
                self.bitboards[c.piece_captured.unwrap() as usize],
            );
        }

        if m.is_castling() {
            //reset rooks after castling
            match m.square_to() {
                2 => {
                    self.bitboards[3] = pop_bit(3, self.bitboards[3]);
                    self.bitboards[3] = set_bit(0, self.bitboards[3]);
                }
                6 => {
                    self.bitboards[3] = pop_bit(5, self.bitboards[3]);
                    self.bitboards[3] = set_bit(7, self.bitboards[3]);
                }
                58 => {
                    self.bitboards[9] = pop_bit(59, self.bitboards[9]);
                    self.bitboards[9] = set_bit(56, self.bitboards[9]);
                }
                62 => {
                    self.bitboards[9] = pop_bit(61, self.bitboards[9]);
                    self.bitboards[9] = set_bit(63, self.bitboards[9]);
                }

                _ => panic!("castling to invalid square in undo_move()"),
            }
        }

        if m.is_en_passant() {
            match self.side_to_move {
                Colour::White => {
                    //white to move before move was made
                    self.bitboards[6] = set_bit(m.square_to() - 8, self.bitboards[6])
                }
                Colour::Black => self.bitboards[0] = set_bit(m.square_to() + 8, self.bitboards[0]),
            };
        }

        self.occupancies[0] = self.bitboards[0]
            | self.bitboards[1]
            | self.bitboards[2]
            | self.bitboards[3]
            | self.bitboards[4]
            | self.bitboards[5];

        self.occupancies[1] = self.bitboards[6]
            | self.bitboards[7]
            | self.bitboards[8]
            | self.bitboards[9]
            | self.bitboards[10]
            | self.bitboards[11];

        self.en_passant = c.ep_reset;
        self.castling = c.castling_reset;
        self.fifty_move = c.fifty_move_reset;
        self.ply -= 1;
    }
}

pub fn is_legal(m: Move, b: &mut Board) -> bool {
    let commit = b.make_move(m);

    let legal = match b.side_to_move {
        // AFTER move has been made
        Colour::White => !is_attacked(lsfb(b.bitboards[11]).unwrap(), Colour::White, b),
        Colour::Black => !is_attacked(lsfb(b.bitboards[5]).unwrap(), Colour::Black, b),
    };
    b.undo_move(m, commit);
    legal
}
