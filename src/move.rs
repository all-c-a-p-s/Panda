/// represent move as u16          hex constant
/// 0000 0000 0011 1111 square from        0x3f
/// 0000 1111 1100 0000 square to          0xfc0
/// 0011 0000 0000 0000 promoted piece     0xf000
/// 0100 0000 0000 0000 en passant flag    0x4000
/// 1000 0000 0000 0000 castling flag      0x8000
/// 1100 0000 0000 0000 promotion flag     0xc000
use crate::board::*;
use crate::helper::*;
use crate::magic::*;
use crate::movegen::*;
use crate::zobrist::*;
use crate::REPETITION_TABLE;

use crate::types::*;

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
}

#[derive(Default)]
pub struct Commit {
    //resets for irreversible fields of Board struct
    pub castling_reset: u8,
    pub ep_reset: Option<Square>,
    pub fifty_move_reset: u8,
    pub piece_captured: Option<Piece>,
    pub hash_key: u64,
    pub pinned: BitBoard,
    pub checkers: BitBoard,
}

impl Move {
    pub fn from_promotion(from: Square, to: Square, promoted_piece: PieceType) -> Self {
        Self {
            data: from as u16
                | (to as u16) << 6
                | (promoted_piece as u16 - 1) << 12
                | PROMOTION_FLAG,
        }
    }

    pub fn from_flags(from: Square, to: Square, flags: u16) -> Self {
        Self {
            data: from as u16 | (to as u16) << 6 | flags,
        }
    }

    pub fn square_from(self) -> Square {
        unsafe { Square::from((self.data & SQUARE_FROM_MASK) as u8) }
    }

    pub fn square_to(self) -> Square {
        unsafe { Square::from(((self.data & SQUARE_TO_MASK) >> 6) as u8) }
    }

    pub fn promoted_piece(self) -> PieceType {
        //must be called only if self.is_promotion() is true
        //also only given piece type, not colour
        unsafe { PieceType::from((((self.data & PROMOTION_MASK) >> 12) + 1) as u8) }
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

    pub fn piece_moved(self, b: &Board) -> Piece {
        b.get_piece_at(self.square_from())
    }

    pub fn is_capture(self, b: &Board) -> bool {
        b.pieces_array[self.square_to()].is_some()
    }

    pub fn is_double_push(self, b: &Board) -> bool {
        if rank(self.square_to()) != 3 && rank(self.square_to()) != 4
            || rank(self.square_from()) != 1 && rank(self.square_from()) != 6
        {
            return false;
        }
        piece_type(self.piece_moved(b)) == PieceType::Pawn
    }

    pub fn piece_captured(self, b: &Board) -> Piece {
        b.get_piece_at(self.square_to())
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
                    PieceType::Knight => "knight",
                    PieceType::Bishop => "bishop",
                    PieceType::Rook => "rook",
                    PieceType::Queen => "queen",
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

pub fn encode_move(from: Square, to: Square, promoted_piece: Option<PieceType>, flag: u16) -> Move {
    if flag & PROMOTION_FLAG == PROMOTION_FLAG {
        //move is a promotion
        Move::from_promotion(from, to, unsafe { promoted_piece.unwrap_unchecked() })
    } else {
        Move::from_flags(from, to, flag)
    }
}

//this gets the longest line in the direction of two squares
//except for those two squares themselves
//used for detecting pins in movegen
static LINE_RAYS: [[BitBoard; 64]; 64] = {
    const fn get_line_rays(sq1: Square, sq2: Square) -> BitBoard {
        let rays = BISHOP_EDGE_RAYS[sq1 as usize];
        if rays & set_bit(sq2, 0) > 0 {
            return (rays | set_bit(sq1, 0)) & (BISHOP_EDGE_RAYS[sq2 as usize] | set_bit(sq2, 0))
                ^ set_bit(sq1, 0)
                ^ set_bit(sq2, 0);
        }

        let rays = ROOK_EDGE_RAYS[sq1 as usize];
        if rays & set_bit(sq2, 0) > 0 {
            return (rays | set_bit(sq1, 0)) & (ROOK_EDGE_RAYS[sq2 as usize] | set_bit(sq2, 0))
                ^ set_bit(sq1, 0)
                ^ set_bit(sq2, 0);
        }
        0
    }

    let mut r = [[0u64; 64]; 64];
    let mut a = 0;
    while a < 64 {
        let mut b = 0;
        while b < 64 {
            r[a][b] = get_line_rays(unsafe { Square::from(a as u8) }, unsafe {
                Square::from(b as u8)
            });
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

        let to = m.square_to();
        let from = m.square_from();
        let piece = match m.is_promotion() {
            false => self.get_piece_at(to),
            true => match self.pieces_array[to] {
                Some(Piece::WN) | Some(Piece::WB) | Some(Piece::WR) | Some(Piece::WQ) => Piece::WP,
                Some(Piece::BN) | Some(Piece::BB) | Some(Piece::BR) | Some(Piece::BQ) => Piece::BP,
                _ => unreachable!(),
            },
        }; //not m.piece_moved(&self) because board has been mutated

        if m.is_promotion() {
            let promoted_piece = self.get_piece_at(to);
            self.bitboards[promoted_piece] = pop_bit(to, self.bitboards[promoted_piece]);
            self.pieces_array[to] = c.piece_captured; //remove promoted piece from pieces_array
            self.bitboards[piece] = set_bit(from, self.bitboards[piece]);
            self.pieces_array[from] = Some(piece);
            //remove promoted piece from bitboard
            self.nnue
                .undo_move(piece, c.piece_captured, Some(promoted_piece), from, to);
        } else {
            self.bitboards[piece] = pop_bit(to, self.bitboards[piece]);
            self.pieces_array[to] = c.piece_captured;
            self.bitboards[piece] = set_bit(from, self.bitboards[piece]);
            self.pieces_array[from] = Some(piece);
            self.nnue.undo_move(piece, c.piece_captured, None, from, to);
        }

        if let Some(victim) = c.piece_captured {
            //put captured piece back onto bitboard
            self.bitboards[victim] = set_bit(to, self.bitboards[victim]);
            self.pieces_array[to] = c.piece_captured;
        }

        if m.is_castling() {
            //reset rooks after castling (king done above)
            match to {
                Square::C1 => {
                    self.bitboards[Piece::WR] = pop_bit(Square::D1, self.bitboards[Piece::WR]);
                    self.bitboards[Piece::WR] = set_bit(Square::A1, self.bitboards[Piece::WR]);

                    self.pieces_array[Square::D1] = None;
                    self.pieces_array[Square::A1] = Some(Piece::WR);

                    self.pieces_array[Square::D1] = None;
                    self.pieces_array[Square::A1] = Some(Piece::WR);
                }
                Square::G1 => {
                    self.bitboards[Piece::WR] = pop_bit(Square::F1, self.bitboards[Piece::WR]);
                    self.bitboards[Piece::WR] = set_bit(Square::H1, self.bitboards[Piece::WR]);

                    self.pieces_array[Square::F1] = None;
                    self.pieces_array[Square::H1] = Some(Piece::WR);
                }
                Square::C8 => {
                    self.bitboards[Piece::BR] = pop_bit(Square::D8, self.bitboards[Piece::BR]);
                    self.bitboards[Piece::BR] = set_bit(Square::A8, self.bitboards[Piece::BR]);

                    self.pieces_array[Square::D8] = None;
                    self.pieces_array[Square::A8] = Some(Piece::BR);
                }
                Square::G8 => {
                    self.bitboards[Piece::BR] = pop_bit(Square::F8, self.bitboards[Piece::BR]);
                    self.bitboards[Piece::BR] = set_bit(Square::H8, self.bitboards[Piece::BR]);

                    self.pieces_array[Square::F8] = None;
                    self.pieces_array[Square::H8] = Some(Piece::BR);
                }

                _ => unreachable!(),
            }

            self.nnue.undo_castling(piece, from, to);
        }

        if m.is_en_passant() {
            match self.side_to_move {
                Colour::White => {
                    //white to move before move was made
                    self.bitboards[Piece::BP] =
                        set_bit(unsafe { to.sub_unchecked(8) }, self.bitboards[Piece::BP]);
                    self.pieces_array[unsafe { to.sub_unchecked(8) }] = Some(Piece::BP);
                    self.nnue.undo_ep(piece, Some(Piece::BP), from, to);
                }
                Colour::Black => {
                    self.bitboards[Piece::WP] =
                        set_bit(unsafe { to.add_unchecked(8) }, self.bitboards[Piece::WP]);
                    self.pieces_array[unsafe { to.add_unchecked(8) }] = Some(Piece::WP);
                    self.nnue.undo_ep(piece, Some(Piece::WP), from, to);
                }
            };
        }

        self.occupancies[WHITE] = self.bitboards[Piece::WP]
            | self.bitboards[Piece::WN]
            | self.bitboards[Piece::WB]
            | self.bitboards[Piece::WR]
            | self.bitboards[Piece::WQ]
            | self.bitboards[Piece::WK];

        self.occupancies[BLACK] = self.bitboards[Piece::BP]
            | self.bitboards[Piece::BN]
            | self.bitboards[Piece::BB]
            | self.bitboards[Piece::BR]
            | self.bitboards[Piece::BQ]
            | self.bitboards[Piece::BK];

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
    #[allow(clippy::result_unit_err)]
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
            piece_captured: None,
            hash_key: self.hash_key,
            pinned: self.pinned,
            checkers: self.checkers,
        };

        self.hash_key = hash_update(self.hash_key, &m, self);
        //MUST be done before any changes made on the board

        (self.checkers, self.pinned) = (0, 0);

        let (from, to) = (m.square_from(), m.square_to());
        let piece_moved = self.get_piece_at(from);
        let victim = self.pieces_array[to];

        let colour = self.side_to_move;

        //SAFETY: there MUST be a king on the board
        let enemy_king = unsafe {
            match colour {
                Colour::White => lsfb(self.bitboards[Piece::BK]).unwrap_unchecked(),
                Colour::Black => lsfb(self.bitboards[Piece::WK]).unwrap_unchecked(),
            }
        };

        if piece_type(piece_moved) == PieceType::Pawn || victim.is_some() {
            self.fifty_move = 0;
        } else {
            self.fifty_move += 1;
        }

        self.en_passant = None;

        if !m.is_promotion() {
            self.nnue.quiet_update(piece_moved, from, to);
        }

        if m.is_castling() {
            //update king and rook for castling
            match to {
                Square::C1 => {
                    self.bitboards[Piece::WK] = set_bit(Square::C1, 0); //works bc only 1 wk
                    self.bitboards[Piece::WR] = pop_bit(Square::A1, self.bitboards[Piece::WR]);
                    self.bitboards[Piece::WR] = set_bit(Square::D1, self.bitboards[Piece::WR]);

                    self.pieces_array[Square::E1] = None;
                    self.pieces_array[Square::A1] = None;
                    self.pieces_array[Square::C1] = Some(Piece::WK);
                    self.pieces_array[Square::D1] = Some(Piece::WR);

                    self.castling &= 0b0000_1100;
                }
                Square::G1 => {
                    self.bitboards[Piece::WK] = set_bit(Square::G1, 0);
                    self.bitboards[Piece::WR] = pop_bit(Square::H1, self.bitboards[Piece::WR]);
                    self.bitboards[Piece::WR] = set_bit(Square::F1, self.bitboards[Piece::WR]);

                    self.pieces_array[Square::E1] = None;
                    self.pieces_array[Square::H1] = None;
                    self.pieces_array[Square::G1] = Some(Piece::WK);
                    self.pieces_array[Square::F1] = Some(Piece::WR);

                    self.castling &= 0b0000_1100;
                }
                Square::C8 => {
                    self.bitboards[Piece::BK] = set_bit(Square::C8, 0);
                    self.bitboards[Piece::BR] = pop_bit(Square::A8, self.bitboards[Piece::BR]);
                    self.bitboards[Piece::BR] = set_bit(Square::D8, self.bitboards[Piece::BR]);

                    self.pieces_array[Square::E8] = None;
                    self.pieces_array[Square::A8] = None;
                    self.pieces_array[Square::C8] = Some(Piece::BK);
                    self.pieces_array[Square::D8] = Some(Piece::BR);

                    self.castling &= 0b0000_0011;
                }
                Square::G8 => {
                    self.bitboards[Piece::BK] = set_bit(Square::G8, 0);
                    self.bitboards[Piece::BR] = pop_bit(Square::H8, self.bitboards[Piece::BR]);
                    self.bitboards[Piece::BR] = set_bit(Square::F8, self.bitboards[Piece::BR]);

                    self.pieces_array[Square::E8] = None;
                    self.pieces_array[Square::H8] = None;
                    self.pieces_array[Square::G8] = Some(Piece::BK);
                    self.pieces_array[Square::F8] = Some(Piece::BR);

                    self.castling &= 0b0000_0011;
                }
                _ => unreachable!(),
            }

            self.nnue.castling_update(piece_moved, from, to);
        } else {
            self.bitboards[piece_moved] ^= set_bit(from, 0);
            self.bitboards[piece_moved] ^= set_bit(to, 0);

            self.pieces_array[from] = None;
            self.pieces_array[to] = Some(piece_moved);

            if let Some(victim) = victim {
                commit.piece_captured = Some(victim);
                self.bitboards[victim] ^= set_bit(to, 0);
                match m.square_to() {
                    //remove castling rights is a1/h1/a8/h8 is captured on
                    Square::A1 => self.castling &= 0b0000_1101,
                    Square::H1 => self.castling &= 0b0000_1110,
                    Square::A8 => self.castling &= 0b0000_0111,
                    Square::H8 => self.castling &= 0b0000_1011,
                    _ => {}
                }

                self.nnue.capture_update(piece_moved, victim, from, to);
            }

            match piece_moved {
                Piece::WN | Piece::BN => self.checkers |= N_ATTACKS[enemy_king] & set_bit(to, 0),
                Piece::WP | Piece::BP => {
                    if m.is_promotion() {
                        self.bitboards[piece_moved] ^= set_bit(to, 0);

                        let promoted_piece = if colour == Colour::White {
                            m.promoted_piece().to_white_piece()
                        } else {
                            m.promoted_piece().to_white_piece().opposite()
                        };

                        self.bitboards[promoted_piece] ^= set_bit(to, 0);
                        self.pieces_array[to] = Some(promoted_piece);

                        if piece_type(promoted_piece) == PieceType::Knight {
                            self.checkers |= N_ATTACKS[enemy_king] & set_bit(to, 0);
                        }

                        self.nnue
                            .promotion_update(piece_moved, Some(promoted_piece), from, to);
                    } else {
                        if rank(from).abs_diff(rank(to)) == 2 {
                            match colour {
                                Colour::White => {
                                    self.en_passant = Some(unsafe { to.sub_unchecked(8) })
                                }
                                Colour::Black => {
                                    self.en_passant = Some(unsafe { to.add_unchecked(8) })
                                }
                            }
                        } else if m.is_en_passant() {
                            match colour {
                                Colour::White => {
                                    self.bitboards[Piece::BP] ^=
                                        set_bit(unsafe { to.sub_unchecked(8) }, 0);
                                    self.pieces_array[unsafe { to.sub_unchecked(8) }] = None;
                                    self.nnue.ep_update(piece_moved, Piece::BP, from, to);
                                }
                                Colour::Black => {
                                    self.bitboards[Piece::WP] ^=
                                        set_bit(unsafe { to.add_unchecked(8) }, 0);
                                    self.pieces_array[unsafe { to.add_unchecked(8) }] = None;
                                    self.nnue.ep_update(piece_moved, Piece::WP, from, to);
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
                Piece::WK => self.castling &= 0b0000_1100,
                Piece::BK => self.castling &= 0b0000_0011,
                Piece::WR | Piece::BR => match from {
                    Square::A1 => self.castling &= 0b0000_1101,
                    Square::H1 => self.castling &= 0b0000_1110,
                    Square::A8 => self.castling &= 0b0000_0111,
                    Square::H8 => self.castling &= 0b0000_1011,
                    _ => {}
                },
                _ => {}
            };
        }

        self.occupancies[WHITE] = self.bitboards[Piece::WP]
            | self.bitboards[Piece::WN]
            | self.bitboards[Piece::WB]
            | self.bitboards[Piece::WR]
            | self.bitboards[Piece::WQ]
            | self.bitboards[Piece::WK];

        self.occupancies[BLACK] = self.bitboards[Piece::BP]
            | self.bitboards[Piece::BN]
            | self.bitboards[Piece::BB]
            | self.bitboards[Piece::BR]
            | self.bitboards[Piece::BQ]
            | self.bitboards[Piece::BK];

        self.occupancies[BOTH] = self.occupancies[WHITE] | self.occupancies[BLACK];
        //update occupancies

        let mut our_attackers = if colour == Colour::White {
            self.occupancies[WHITE]
                & ((BISHOP_EDGE_RAYS[enemy_king]
                    & (self.bitboards[Piece::WB] | self.bitboards[Piece::WQ]))
                    | ROOK_EDGE_RAYS[enemy_king]
                        & (self.bitboards[Piece::WR] | self.bitboards[Piece::WQ]))
        } else {
            self.occupancies[BLACK]
                & ((BISHOP_EDGE_RAYS[enemy_king]
                    & (self.bitboards[Piece::BB] | self.bitboards[Piece::BQ]))
                    | ROOK_EDGE_RAYS[enemy_king]
                        & (self.bitboards[Piece::BR] | self.bitboards[Piece::BQ]))
        };

        while let Some(sq) = lsfb(our_attackers) {
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
        //SAFETY: there MUST be a king on the board
        let king_sq = unsafe {
            lsfb(
                self.bitboards[match self.side_to_move {
                    Colour::White => Piece::WK,
                    Colour::Black => Piece::BK,
                }],
            )
            .unwrap_unchecked()
        };

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

        let piece_moved = self.get_piece_at(from);

        match piece_type(piece_moved) {
            PieceType::Pawn => {
                if m.is_en_passant() {
                    let taken = if piece_moved == Piece::WP {
                        unsafe { to.sub_unchecked(8) }
                    } else {
                        unsafe { to.add_unchecked(8) }
                    };
                    //exception here since you can take the pawn giving check en passant
                    (target_squares & set_bit(to, 0) > 0 || lsfb(self.checkers) == Some(taken))
                        && check_en_passant(m, self)
                } else {
                    target_squares & set_bit(to, 0) > 0
                }
            }
            _ => target_squares & set_bit(to, 0) > 0,
        }
    }

    fn legal_king_move(&mut self, m: Move) -> bool {
        //SAFETY: there MUST be a king on the board
        let king_sq = unsafe {
            lsfb(
                self.bitboards[match self.side_to_move {
                    Colour::White => Piece::WK,
                    Colour::Black => Piece::BK,
                }],
            )
            .unwrap_unchecked()
        };
        self.occupancies[BOTH] ^= set_bit(king_sq, 0);
        let ok = !is_attacked(m.square_to(), self.side_to_move.opponent(), self);
        self.occupancies[BOTH] ^= set_bit(king_sq, 0);

        ok
    }

    //NOTE: assumes at most one checker (double check case handled elsewhere)
    fn target_squares(&self, in_check: bool) -> BitBoard {
        let colour = self.side_to_move;
        let targets = if in_check {
            //SAFETY: there MUST be a checker since we know we are in check
            let checker = unsafe { lsfb(self.checkers).unwrap_unchecked() };
            //SAFETY: there MUST be a king on the board
            let our_king = unsafe {
                lsfb(
                    self.bitboards[match colour {
                        Colour::White => Piece::WK,
                        Colour::Black => Piece::BK,
                    }],
                )
                .unwrap_unchecked()
            };
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
