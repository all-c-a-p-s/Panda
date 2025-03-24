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
use crate::magic::*;
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
    pub pinned: u64,
    pub checkers: u64,
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

//this gets the longest line in the direction of two squares
//except for those two squares themselves
//used for detecting pins in movegen
const fn get_line_rays(sq1: usize, sq2: usize) -> u64 {
    let rays = BISHOP_EDGE_RAYS[sq1];
    if rays & set_bit(sq2, 0) > 0 {
        return (rays | set_bit(sq1, 0)) & (BISHOP_EDGE_RAYS[sq2] | set_bit(sq2, 0))
            ^ set_bit(sq1, 0)
            ^ set_bit(sq2, 0);
    }

    let rays = ROOK_EDGE_RAYS[sq1];
    if rays & set_bit(sq2, 0) > 0 {
        return (rays | set_bit(sq1, 0)) & (ROOK_EDGE_RAYS[sq2] | set_bit(sq2, 0))
            ^ set_bit(sq1, 0)
            ^ set_bit(sq2, 0);
    }
    0
}

const LINE_RAYS: [[u64; 64]; 64] = {
    let mut r = [[0u64; 64]; 64];
    let mut a = 0;
    while a < 64 {
        let mut b = 0;
        while b < 64 {
            r[a][b] = get_line_rays(a, b);
            b += 1;
        }
        a += 1;
    }
    r
};

impl Board {
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
                _ => unreachable!(),
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

                _ => unreachable!(),
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
        self.pinned = c.pinned;
        self.checkers = c.checkers;
        self.ply -= 1;

        unsafe {
            REPETITION_TABLE[self.ply + 1] = 0u64;
        }
    }
}

impl Board {
    pub fn try_move(&mut self, m: Move) -> Result<Commit, ()> {
        if !self.is_legal(m) {
            return Err(());
        }
        Ok(self.play_unchecked(m))
    }

    pub fn play_unchecked(&mut self, m: Move) -> Commit {
        let mut commit = Commit {
            castling_reset: self.castling,
            ep_reset: self.en_passant,
            fifty_move_reset: self.fifty_move,
            piece_captured: NO_PIECE,
            hash_key: self.hash_key,
            made_move: true,
            pinned: self.pinned,
            checkers: self.checkers,
        };

        self.hash_key = hash_update(self.hash_key, &m, &self);
        //MUST be done before any changes made on the board

        (self.checkers, self.pinned) = (0, 0);

        let (from, to) = (m.square_from(), m.square_to());
        let piece_moved = self.pieces_array[from];
        let victim = self.pieces_array[to];

        let colour = self.side_to_move;

        let enemy_king = match colour {
            Colour::White => lsfb(self.bitboards[BK]),
            Colour::Black => lsfb(self.bitboards[WK]),
        };

        if piece_type(piece_moved) == PAWN || victim != NO_PIECE {
            self.fifty_move = 0;
        } else {
            self.fifty_move += 1;
        }

        self.en_passant = NO_SQUARE;

        if m.is_castling() {
            //update king and rook for castling
            match to {
                C1 => {
                    self.bitboards[WK] = set_bit(C1, 0); //works bc only 1 wk
                    self.bitboards[WR] = pop_bit(A1, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(D1, self.bitboards[WR]);

                    self.pieces_array[E1] = NO_PIECE;
                    self.pieces_array[A1] = NO_PIECE;
                    self.pieces_array[C1] = WK;
                    self.pieces_array[D1] = WR;

                    self.castling &= 0b0000_1100;
                }
                G1 => {
                    self.bitboards[WK] = set_bit(G1, 0);
                    self.bitboards[WR] = pop_bit(H1, self.bitboards[WR]);
                    self.bitboards[WR] = set_bit(F1, self.bitboards[WR]);

                    self.pieces_array[E1] = NO_PIECE;
                    self.pieces_array[H1] = NO_PIECE;
                    self.pieces_array[G1] = WK;
                    self.pieces_array[F1] = WR;

                    self.castling &= 0b0000_1100;
                }
                C8 => {
                    self.bitboards[BK] = set_bit(C8, 0);
                    self.bitboards[BR] = pop_bit(A8, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(D8, self.bitboards[BR]);

                    self.pieces_array[E8] = NO_PIECE;
                    self.pieces_array[A8] = NO_PIECE;
                    self.pieces_array[C8] = BK;
                    self.pieces_array[D8] = BR;

                    self.castling &= 0b0000_0011;
                }
                G8 => {
                    self.bitboards[BK] = set_bit(G8, 0);
                    self.bitboards[BR] = pop_bit(H8, self.bitboards[BR]);
                    self.bitboards[BR] = set_bit(F8, self.bitboards[BR]);

                    self.pieces_array[E8] = NO_PIECE;
                    self.pieces_array[H8] = NO_PIECE;
                    self.pieces_array[G8] = BK;
                    self.pieces_array[F8] = BR;

                    self.castling &= 0b0000_0011;
                }
                _ => unreachable!(),
            }
        } else {
            self.bitboards[piece_moved] ^= set_bit(from, 0);
            self.bitboards[piece_moved] ^= set_bit(to, 0);

            self.pieces_array[from] = NO_PIECE;
            self.pieces_array[to] = piece_moved;

            if victim != NO_PIECE {
                commit.piece_captured = victim;
                self.bitboards[victim] ^= set_bit(to, 0);
                match m.square_to() {
                    //remove castling rights is a1/h1/a8/h8 is captured on
                    A1 => self.castling &= 0b0000_1101,
                    H1 => self.castling &= 0b0000_1110,
                    A8 => self.castling &= 0b0000_0111,
                    H8 => self.castling &= 0b0000_1011,
                    _ => {}
                }
            }

            match piece_moved {
                WN | BN => self.checkers |= N_ATTACKS[enemy_king] & set_bit(to, 0),
                WP | BP => {
                    if m.is_promotion() {
                        self.bitboards[piece_moved] ^= set_bit(to, 0);

                        let promoted_piece = if colour == Colour::White {
                            m.promoted_piece()
                        } else {
                            m.promoted_piece() + 6
                        };

                        self.bitboards[promoted_piece] ^= set_bit(to, 0);
                        self.pieces_array[to] = promoted_piece;

                        if piece_type(promoted_piece) == KNIGHT {
                            self.checkers |= N_ATTACKS[enemy_king] & set_bit(to, 0);
                        }
                    } else {
                        if rank(from).abs_diff(rank(to)) == 2 {
                            match colour {
                                Colour::White => self.en_passant = to - 8,
                                Colour::Black => self.en_passant = to + 8,
                            }
                        } else if m.is_en_passant() {
                            match colour {
                                Colour::White => {
                                    self.bitboards[BP] ^= set_bit(to - 8, 0);
                                    self.pieces_array[to - 8] = NO_PIECE;
                                }
                                Colour::Black => {
                                    self.bitboards[WP] ^= set_bit(to + 8, 0);
                                    self.pieces_array[to + 8] = NO_PIECE;
                                }
                            }
                        }

                        self.checkers |= match colour {
                            Colour::White => BP_ATTACKS,
                            Colour::Black => WP_ATTACKS,
                        }[enemy_king]
                            & set_bit(to, 0);
                    }
                }
                WK => self.castling &= 0b0000_1100,
                BK => self.castling &= 0b0000_0011,
                WR | BR => match from {
                    A1 => self.castling &= 0b0000_1101,
                    H1 => self.castling &= 0b0000_1110,
                    A8 => self.castling &= 0b0000_0111,
                    H8 => self.castling &= 0b0000_1011,
                    _ => {}
                },
                _ => {}
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
        //update occupancies

        let mut our_attackers = if colour == Colour::White {
            self.occupancies[WHITE]
                & ((BISHOP_EDGE_RAYS[enemy_king] & (self.bitboards[WB] | self.bitboards[WQ]))
                    | ROOK_EDGE_RAYS[enemy_king] & (self.bitboards[WR] | self.bitboards[WQ]))
        } else {
            self.occupancies[BLACK]
                & ((BISHOP_EDGE_RAYS[enemy_king] & (self.bitboards[BB] | self.bitboards[BQ]))
                    | ROOK_EDGE_RAYS[enemy_king] & (self.bitboards[BR] | self.bitboards[BQ]))
        };

        while our_attackers > 0 {
            let sq = lsfb(our_attackers);
            let ray_between = RAY_BETWEEN[sq][enemy_king] & self.occupancies[BOTH];
            match count(ray_between) {
                0 => self.checkers |= set_bit(sq, 0),
                1 => self.pinned |= ray_between,
                _ => {}
            }
            our_attackers = pop_bit(sq, our_attackers);
        }

        self.side_to_move = self.side_to_move.opponent();
        self.ply += 1;

        unsafe {
            REPETITION_TABLE[self.ply] = self.hash_key;
        }

        commit
    }

    //NOTE: this assumes that the move is pseudo-legal
    pub fn is_legal(&mut self, m: Move) -> bool {
        let king_sq = lsfb(
            self.bitboards[match self.side_to_move {
                Colour::White => WK,
                Colour::Black => BK,
            }],
        );

        let from = m.square_from();
        let to = m.square_to();

        if m.square_from() == king_sq {
            return self.legal_king_move(m);
        }

        if self.pinned & set_bit(from, 0) > 0 && LINE_RAYS[from][king_sq] & set_bit(to, 0) == 0 {
            return false;
        }

        let target_squares = match count(self.checkers) {
            0 => self.target_squares(false),
            1 => self.target_squares(true),
            _ => return false,
        };

        let piece_moved = self.pieces_array[from];

        match piece_type(piece_moved) {
            PAWN => {
                if m.is_en_passant() {
                    let taken = if piece_moved == WP { to - 8 } else { to + 8 };
                    //exception here since you can take the pawn giving check en passant
                    (target_squares & set_bit(to, 0) > 0 || lsfb(self.checkers) == taken)
                        && check_en_passant(m, &self)
                } else {
                    target_squares & set_bit(to, 0) > 0
                }
            }
            _ => target_squares & set_bit(to, 0) > 0,
        }
    }

    fn legal_king_move(&mut self, m: Move) -> bool {
        let king_sq = lsfb(
            self.bitboards[match self.side_to_move {
                Colour::White => WK,
                Colour::Black => BK,
            }],
        );
        self.occupancies[BOTH] ^= set_bit(king_sq, 0);
        let ok = !is_attacked(m.square_to(), self.side_to_move.opponent(), &self);
        self.occupancies[BOTH] ^= set_bit(king_sq, 0);

        ok
    }

    //NOTE: assumes at most one checker (double check case handled elsewhere)
    fn target_squares(&self, in_check: bool) -> u64 {
        let colour = self.side_to_move;
        let targets = if in_check {
            let checker = lsfb(self.checkers);
            let our_king = lsfb(
                self.bitboards[match colour {
                    Colour::White => WK,
                    Colour::Black => BK,
                }],
            );
            RAY_BETWEEN[checker][our_king] | set_bit(checker, 0)
        } else {
            !0u64
        };

        targets
            & !self.occupancies[match colour {
                Colour::White => WHITE,
                Colour::Black => BLACK,
            }]
    }
}
