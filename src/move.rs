/*
*
* represent move as u16          hex constant
* 0000 0000 0011 1111 square from        0x3f
* 0000 1111 1100 0000 square to          0xfc0
* 0011 0000 0000 0000 promoted piece     0xf000
* 0100 0000 0000 0000 en passant flag    0x4000
* 1000 0000 0000 0000 castling flag      0x8000
* 1100 0000 0000 0000 promotion flag     0xc000
*
*/

use crate::board::*;
use crate::helper::*;
use crate::movegen::*;
use crate::zobrist::*;
use crate::REPETITION_TABLE;

pub const SQUARE_FROM_MASK: u16 = 0b0000_0000_0011_1111;
pub const SQUARE_TO_MASK: u16 = 0b0000_1111_1100_0000;
pub const PROMOTION_MASK: u16 = 0b0011_0000_0000_0000; // != PROMOTION_FLAG

pub const NO_FLAG: u16 = 0;
pub const EN_PASSANT_FLAG: u16 = 0b0100_0000_0000_0000;
pub const CASTLING_FLAG: u16 = 0b1000_0000_0000_0000;
pub const PROMOTION_FLAG: u16 = 0b1100_0000_0000_0000;

pub const NULL_MOVE: Move = Move { data: 0u16 };

#[derive(Debug)]
pub struct MoveList {
    pub moves: [Move; MAX_MOVES], //max number of legal moves in a chess position
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Move {
    pub data: u16,
    //    pub score: i32, < storing an i32 in move struct slows down performance significantly
}

#[derive(Default)]
pub struct Commit {
    //resets for irreversible fields of Board struct
    pub castling_reset: u8,
    pub ep_reset: usize,
    pub fifty_move_reset: u8,
    pub piece_captured: usize,
    pub hash_key: u64,
    pub made_move: bool,
}

impl Move {
    pub fn from_promotion(from: usize, to: usize, promoted_piece: usize) -> Self {
        Self {
            data: from as u16
                | (to as u16) << 6
                | (promoted_piece as u16 - 1) << 12
                | PROMOTION_FLAG,
        }
    }

    pub fn from_flags(from: usize, to: usize, flags: u16) -> Self {
        Self {
            data: from as u16 | (to as u16) << 6 | flags,
        }
    }

    pub fn square_from(self) -> usize {
        (self.data & SQUARE_FROM_MASK) as usize
    }

    pub fn square_to(self) -> usize {
        ((self.data & SQUARE_TO_MASK) >> 6) as usize
    }

    pub fn promoted_piece(self) -> usize {
        //must be called only if self.is_promotion() is true
        //also only given piece type, not colour
        (((self.data & PROMOTION_MASK) >> 12) + 1) as usize
    }

    pub fn is_promotion(self) -> bool {
        self.data & PROMOTION_FLAG == PROMOTION_FLAG
    }

    pub fn is_castling(self) -> bool {
        (self.data & CASTLING_FLAG) > 0 && (self.data & EN_PASSANT_FLAG == 0)
    }

    pub fn is_en_passant(self) -> bool {
        (self.data & EN_PASSANT_FLAG) > 0 && (self.data & CASTLING_FLAG == 0)
    }

    pub fn is_null(self) -> bool {
        self.data == 0
    }

    pub fn piece_moved(self, b: &Board) -> usize {
        b.pieces_array[self.square_from()]
    }

    pub fn is_capture(self, b: &Board) -> bool {
        b.pieces_array[self.square_to()] != NO_PIECE
    }

    pub fn is_double_push(self, b: &Board) -> bool {
        if rank(self.square_to()) != 3 && rank(self.square_to()) != 4
            || rank(self.square_from()) != 1 && rank(self.square_from()) != 6
        {
            return false;
        }
        piece_type(self.piece_moved(b)) == PAWN
    }

    pub fn piece_captured(self, b: &Board) -> usize {
        b.pieces_array[self.square_to()]
    }

    pub fn is_tactical(self, b: &Board) -> bool {
        self.is_promotion() || self.is_capture(b) || self.is_en_passant()
    }
}

#[allow(clippy::too_many_arguments)]
impl Move {
    pub fn print_move(&self) {
        println!(
            "{}{}",
            coordinate(self.square_from()),
            coordinate(self.square_to())
        );

        println!(
            "promoted to {}",
            if self.is_promotion() {
                match self.promoted_piece() {
                    KNIGHT => "knight",
                    BISHOP => "bishop",
                    ROOK => "rook",
                    QUEEN => "queen",
                    _ => unreachable!(),
                }
            } else {
                "NONE"
            }
        );

        println!("en passant: {}", self.is_en_passant());
        println!("castling: {}", self.is_castling());
    }
}

pub fn encode_move(sq_from: usize, sq_to: usize, promoted_piece: usize, flag: u16) -> Move {
    if flag & PROMOTION_FLAG == PROMOTION_FLAG {
        //move is a promotion
        Move::from_promotion(sq_from, sq_to, promoted_piece)
    } else {
        Move::from_flags(sq_from, sq_to, flag)
    }
}

impl Board {
    pub fn make_move(&mut self, m: Move) -> Commit {
        let mut commit = Commit {
            castling_reset: self.castling,
            ep_reset: self.en_passant,
            fifty_move_reset: self.fifty_move,
            piece_captured: NO_PIECE,
            hash_key: self.hash_key,
            made_move: true,
        };

        self.hash_key = hash_update(self.hash_key, &m, &self);
        //MUST be done before any changes made on the board

        let sq_from = m.square_from();
        let sq_to = m.square_to();
        let piece = m.piece_moved(&self);
        let piece_captured = m.piece_captured(&self);
        let double_push = m.is_double_push(&self);
        let promoted_piece = match m.is_promotion() {
            false => NO_PIECE,
            true => match self.side_to_move {
                Colour::White => m.promoted_piece(),
                Colour::Black => m.promoted_piece() + 6,
            },
        };

        //NOTE: piece removed from sq_from below

        if m.is_capture(&self) {
            //remove captured piece from bitboard
            self.bitboards[piece_captured] = pop_bit(sq_to, self.bitboards[piece_captured]);
            self.pieces_array[sq_to] = piece;
            commit.piece_captured = piece_captured;
            match m.square_to() {
                //remove castling rights is a1/h1/a8/h8 is captured on
                A1 => self.castling &= 0b0000_1101,
                H1 => self.castling &= 0b0000_1110,
                A8 => self.castling &= 0b0000_0111,
                H8 => self.castling &= 0b0000_1011,
                _ => {}
            }
        } else if m.is_en_passant() {
            match piece {
                WP => {
                    self.bitboards[BP] = pop_bit(sq_to - 8, self.bitboards[BP]);
                    self.pieces_array[sq_to - 8] = NO_PIECE;
                }
                BP => {
                    self.bitboards[WP] = pop_bit(sq_to + 8, self.bitboards[WP]);
                    self.pieces_array[sq_to + 8] = NO_PIECE;
                }
                _ => panic!("non-pawn is capturing en passant 🤔"),
            }
        }

        if m.is_promotion() {
            self.bitboards[promoted_piece] = set_bit(sq_to, self.bitboards[promoted_piece]);
            self.pieces_array[sq_to] = promoted_piece;
            //remove pawn and add promoted piece
        } else if m.is_castling() {
            //update king and rook for castling
            match sq_to {
                C1 => {
                    self.bitboards[WK] = set_bit(C1, 0); //works bc only 1 wk
                    self.bitboards[WR] = pop_bit(A1, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(D1, self.bitboards[WR]);

                    self.pieces_array[E1] = NO_PIECE;
                    self.pieces_array[A1] = NO_PIECE;
                    self.pieces_array[C1] = WK;
                    self.pieces_array[D1] = WR;
                }
                G1 => {
                    self.bitboards[WK] = set_bit(G1, 0);
                    self.bitboards[WR] = pop_bit(H1, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(F1, self.bitboards[WR]);

                    self.pieces_array[E1] = NO_PIECE;
                    self.pieces_array[H1] = NO_PIECE;
                    self.pieces_array[G1] = WK;
                    self.pieces_array[F1] = WR;
                }
                C8 => {
                    self.bitboards[BK] = set_bit(C8, 0);
                    self.bitboards[BR] = pop_bit(A8, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(D8, self.bitboards[BR]);

                    self.pieces_array[E8] = NO_PIECE;
                    self.pieces_array[A8] = NO_PIECE;
                    self.pieces_array[C8] = BK;
                    self.pieces_array[D8] = BR;
                }
                G8 => {
                    self.bitboards[BK] = set_bit(G8, 0);
                    self.bitboards[BR] = pop_bit(H8, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(F8, self.bitboards[BR]);

                    self.pieces_array[E8] = NO_PIECE;
                    self.pieces_array[H8] = NO_PIECE;
                    self.pieces_array[G8] = BK;
                    self.pieces_array[F8] = BR;
                }
                _ => panic!("castling to a square that is not c1 g1 c8 or g8 🤔"),
            }
        } else {
            self.bitboards[piece] = set_bit(sq_to, self.bitboards[piece]);
            self.pieces_array[sq_to] = piece;
            //set new bit on bitboard
        }

        self.bitboards[piece] = pop_bit(sq_from, self.bitboards[piece]);
        self.pieces_array[sq_from] = NO_PIECE;
        //remove moved piece from sq_from in all cases

        if double_push {
            //must do before moving pieces on the board
            self.en_passant = match piece {
                WP => sq_from + 8,
                BP => sq_from - 8,
                _ => panic!("non-pawn is making a double push 🤔"),
            };
        } else {
            self.en_passant = NO_SQUARE;
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

        if piece_captured != NO_PIECE || piece_type(piece) == PAWN {
            self.fifty_move = 0;
        } else {
            self.fifty_move += 1;
        }

        if piece_type(piece) == KING {
            if piece == WK {
                //wk moved
                self.castling &= 0b0000_1100;
            } else if piece == BK {
                self.castling &= 0b0000_0011;
            }
        }

        if piece_type(piece) == ROOK {
            if sq_from == A1 {
                //wr leaves a1
                self.castling &= 0b0000_1101;
            } else if sq_from == H1 {
                self.castling &= 0b0000_1110;
            } else if sq_from == A8 {
                self.castling &= 0b0000_0111;
            } else if sq_from == H8 {
                self.castling &= 0b0000_1011;
            } //update castling rights
        }

        self.ply += 1;

        self.side_to_move = match self.side_to_move {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };

        unsafe {
            REPETITION_TABLE[self.ply] = self.hash_key;
        }

        commit
    }

    pub fn undo_move(&mut self, m: Move, c: &Commit) {
        //incremental update should be faster than copying the whole board
        self.side_to_move = match self.side_to_move {
            //NOTE: updated at the beginning of the function
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };
        self.hash_key = c.hash_key;

        let sq_to = m.square_to();
        let sq_from = m.square_from();
        let piece = match m.is_promotion() {
            false => self.pieces_array[sq_to],
            true => match self.pieces_array[sq_to] {
                WN..=WQ => WP,
                BN..=BQ => BP,
                _ => panic!("impossible n'est pas français"),
            },
        }; //not m.piece_moved(&self) because board has been mutated

        if m.is_promotion() {
            let promoted_piece = self.pieces_array[sq_to];
            self.bitboards[promoted_piece] = pop_bit(sq_to, self.bitboards[promoted_piece]);
            self.pieces_array[sq_to] = c.piece_captured; //remove promoted piece from pieces_array
            self.bitboards[piece] = set_bit(sq_from, self.bitboards[piece]);
            self.pieces_array[sq_from] = piece;
            //remove promoted piece from bitboard
        } else {
            self.bitboards[piece] = pop_bit(sq_to, self.bitboards[piece]);
            self.pieces_array[sq_to] = c.piece_captured;
            self.bitboards[piece] = set_bit(sq_from, self.bitboards[piece]);
            self.pieces_array[sq_from] = piece;
        }

        if c.piece_captured != NO_PIECE {
            //put captured piece back onto bitboard
            self.bitboards[c.piece_captured] = set_bit(sq_to, self.bitboards[c.piece_captured]);
            self.pieces_array[sq_to] = c.piece_captured;
        }

        if m.is_castling() {
            //reset rooks after castling (king done above)
            match sq_to {
                C1 => {
                    self.bitboards[WR] = pop_bit(D1, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(A1, self.bitboards[WR]);

                    self.pieces_array[D1] = NO_PIECE;
                    self.pieces_array[A1] = WR;
                }
                G1 => {
                    self.bitboards[WR] = pop_bit(F1, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(H1, self.bitboards[WR]);

                    self.pieces_array[F1] = NO_PIECE;
                    self.pieces_array[H1] = WR;
                }
                C8 => {
                    self.bitboards[BR] = pop_bit(D8, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(A8, self.bitboards[BR]);

                    self.pieces_array[D8] = NO_PIECE;
                    self.pieces_array[A8] = BR;
                }
                G8 => {
                    self.bitboards[BR] = pop_bit(F8, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(H8, self.bitboards[BR]);

                    self.pieces_array[F8] = NO_PIECE;
                    self.pieces_array[H8] = BR;
                }

                _ => panic!("castling to invalid square in undo_move()"),
            }
        }

        if m.is_en_passant() {
            match self.side_to_move {
                Colour::White => {
                    //white to move before move was made
                    self.bitboards[BP] = set_bit(sq_to - 8, self.bitboards[BP]);
                    self.pieces_array[sq_to - 8] = BP;
                }
                Colour::Black => {
                    self.bitboards[WP] = set_bit(sq_to + 8, self.bitboards[WP]);
                    self.pieces_array[sq_to + 8] = WP;
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

        unsafe {
            REPETITION_TABLE[self.ply + 1] = 0u64;
        }
    }
}

//NOTE: this doesn't actually work but I'm not using it rn
pub fn is_legal(m: Move, b: &mut Board) -> bool {
    let commit = b.make_move(m);

    let legal = match b.side_to_move {
        // AFTER move has been made
        Colour::White => !is_attacked(lsfb(b.bitboards[BK]), Colour::White, b),
        Colour::Black => !is_attacked(lsfb(b.bitboards[WK]), Colour::Black, b),
    };
    b.undo_move(m, &commit);
    return legal;
}
impl Board {
    /*pub fn try_move(&mut self, m: Move, pin_rays: &[u64]) -> (Commit, bool) {
        let mut commit = Commit::default();
        let mut ok = false;
        if self.is_check() {
            commit = self.make_move(m);
            ok = !self.is_still_check();
        } else if legal_non_check_evasion(m, &self, pin_rays) {
            commit = self.make_move(m);
            ok = true;
        }

        (commit, ok)
    }*/

    pub fn try_move(&mut self, m: Move) -> (Commit, bool) {
        let commit = self.make_move(m);
        let ok = !self.is_still_check();
        (commit, ok)
    }
}
