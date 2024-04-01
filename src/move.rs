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
    pub piece_captured: usize,
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
                WP => "white pawn",
                WN => "white knight",
                WB => "white bishop",
                WR => "white rook",
                WQ => "white queen",
                WK => "white king",
                BP => "black pawn",
                BN => "black knight",
                BB => "black bishop",
                BR => "black rook",
                BQ => "black queen",
                BK => "black king",
                _ => "impossible",
            }
        );

        println!(
            "promoted to {}",
            match self.promoted_piece() {
                WN => "white knight",
                WB => "white bishop",
                WR => "white rook",
                WQ => "white queen",
                BN => "black knight",
                BB => "black bishop",
                BR => "black rook",
                BQ => "black queen",
                NO_PIECE => "NONE",
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
        //PERF: speed this up?
        if get_bit(sq_from, board.bitboards[i]) > 0 {
            piece = i;
            break;
        }
    }

    let capture = get_bit(sq_to, board.occupancies[BOTH]) == 1;
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
            piece_captured: NO_PIECE,
        };
        let sq_from = m.square_from();
        let sq_to = m.square_to();
        let piece = m.piece_moved();
        if m.is_capture() {
            //PERF: speed this up?
            //remove captured piece from bitboard
            for i in 0..12 {
                if get_bit(sq_to, self.bitboards[i]) > 0 {
                    self.bitboards[i] = pop_bit(sq_to, self.bitboards[i]);
                    commit.piece_captured = i;
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
                WP => self.bitboards[BP] = pop_bit(sq_to - 8, self.bitboards[BP]),
                BP => self.bitboards[WP] = pop_bit(sq_to + 8, self.bitboards[WP]),
                _ => panic!("non-pawn is capturing en passant 🤔"),
            }
        }

        if m.promoted_piece() != NO_PIECE {
            self.bitboards[piece] = pop_bit(sq_from, self.bitboards[piece]);
            self.bitboards[m.promoted_piece()] = set_bit(sq_to, self.bitboards[m.promoted_piece()]);
            //remove pawn and add promoted piece
        } else if m.is_castling() {
            //update king and rook for castling
            match sq_to {
                2 => {
                    self.bitboards[WK] = set_bit(2, 0); //works bc only 1 wk
                    self.bitboards[WR] = pop_bit(0, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(3, self.bitboards[WR])
                }
                6 => {
                    self.bitboards[WK] = set_bit(6, 0);
                    self.bitboards[WR] = pop_bit(7, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(5, self.bitboards[WR])
                }
                58 => {
                    self.bitboards[BK] = set_bit(58, 0);
                    self.bitboards[BR] = pop_bit(56, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(59, self.bitboards[BR])
                }
                62 => {
                    self.bitboards[BK] = set_bit(62, 0);
                    self.bitboards[BR] = pop_bit(63, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(61, self.bitboards[BR])
                }
                _ => panic!("castling to a square that is not c1 g1 c8 or g8 🤔"),
            }
        } else {
            self.bitboards[piece] = pop_bit(sq_from, self.bitboards[piece]); //pop bit from bitboard
            self.bitboards[piece] = set_bit(sq_to, self.bitboards[piece]);
            //set new bit on bitboard
        }

        self.occupancies[WHITE] = self.bitboards[WP]
            | self.bitboards[WN]
            | self.bitboards[WB]
            | self.bitboards[WR]
            | self.bitboards[WQ]
            | self.bitboards[WK];

        self.occupancies[BLACK] = self.bitboards[BP]
            | self.bitboards[BN]
            | self.bitboards[BB]
            | self.bitboards[BR]
            | self.bitboards[BQ]
            | self.bitboards[BK];

        self.occupancies[BOTH] = self.occupancies[WHITE] | self.occupancies[BLACK];
        //update occupancies

        if m.is_double_push() {
            self.en_passant = match piece {
                WP => sq_from + 8,
                BP => sq_from - 8,
                _ => panic!("non-pawn is making a double push 🤔"),
            }
        } else {
            self.en_passant = NO_SQUARE;
        }

        if (piece % 6 == 0) || m.is_capture() {
            self.fifty_move = 0;
        } else {
            self.fifty_move += 1;
        }

        if piece == WK {
            //wk moved
            self.castling &= 0b0000_1100;
        } else if piece == 11 {
            self.castling &= 0b0000_0011;
        }

        if piece == WR && sq_from == 0 {
            //rw leaves a1
            self.castling &= 0b0000_1101;
        } else if piece == WR && sq_from == 7 {
            self.castling &= 0b0000_1110;
        } else if piece == BR && sq_from == 56 {
            self.castling &= 0b0000_0111;
        } else if piece == BR && sq_from == 63 {
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
            //NOTE: updated at the beginning of the function
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };
        let piece = m.piece_moved();
        if m.promoted_piece() != NO_PIECE {
            self.bitboards[m.promoted_piece()] =
                pop_bit(m.square_to(), self.bitboards[m.promoted_piece()]);
            match m.piece_moved() {
                0 => self.bitboards[WP] = set_bit(m.square_from(), self.bitboards[WP]),
                6 => self.bitboards[BP] = set_bit(m.square_from(), self.bitboards[BP]),
                _ => panic!("non-pawn is promoting lol"),
            }
            //remove promoted piece from bitboard
        } else {
            self.bitboards[piece] = pop_bit(m.square_to(), self.bitboards[piece]);
            self.bitboards[piece] = set_bit(m.square_from(), self.bitboards[piece]);
        }

        if c.piece_captured != NO_PIECE {
            //put captured piece back onto bitboard
            self.bitboards[c.piece_captured] =
                set_bit(m.square_to(), self.bitboards[c.piece_captured]);
        }

        if m.is_castling() {
            //reset rooks after castling
            match m.square_to() {
                2 => {
                    self.bitboards[WR] = pop_bit(3, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(0, self.bitboards[WR]);
                }
                6 => {
                    self.bitboards[WR] = pop_bit(5, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(7, self.bitboards[WR]);
                }
                58 => {
                    self.bitboards[BR] = pop_bit(59, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(56, self.bitboards[BR]);
                }
                62 => {
                    self.bitboards[BR] = pop_bit(61, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(63, self.bitboards[BR]);
                }

                _ => panic!("castling to invalid square in undo_move()"),
            }
        }

        if m.is_en_passant() {
            match self.side_to_move {
                Colour::White => {
                    //white to move before move was made
                    self.bitboards[BP] = set_bit(m.square_to() - 8, self.bitboards[BP])
                }
                Colour::Black => {
                    self.bitboards[WP] = set_bit(m.square_to() + 8, self.bitboards[WP])
                }
            };
        }

        self.occupancies[WHITE] = self.bitboards[WP]
            | self.bitboards[WN]
            | self.bitboards[WB]
            | self.bitboards[WR]
            | self.bitboards[WQ]
            | self.bitboards[WK];

        self.occupancies[BLACK] = self.bitboards[BP]
            | self.bitboards[BN]
            | self.bitboards[BB]
            | self.bitboards[BR]
            | self.bitboards[BQ]
            | self.bitboards[BK];

        self.occupancies[BOTH] = self.occupancies[WHITE] | self.occupancies[BLACK];

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
        Colour::White => !is_attacked(lsfb(b.bitboards[BK]), Colour::White, b),
        Colour::Black => !is_attacked(lsfb(b.bitboards[WK]), Colour::Black, b),
    };
    b.undo_move(m, commit);
    legal
}
