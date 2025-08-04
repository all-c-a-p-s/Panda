use crate::movegen::get_attackers;
use crate::r#move::{Move, MoveList};
use crate::search::{params, INFINITY};
use crate::thread::Thread;
use crate::types::{OccupancyIndex, Piece, PieceType, BLACK_PIECES, WHITE_PIECES};
use crate::{
    get_bishop_attacks, get_rook_attacks, lsfb, piece_type, read_param, set_bit, Board, Colour,
    MAX_MOVES,
};

//taken from Carp
const MVV: [i32; 6] = [0, 2400, 2400, 4800, 9600, 0];

//same as MG evaluation weights (haven't updated these in a while)
pub const SEE_VALUES: [i32; 6] = [85, 306, 322, 490, 925, INFINITY];

impl Move {
    /// Static Exchange Evalutaion (SEE):
    /// If we already have an advantage of 'threshold', and play this move, will we still be ahead
    /// after the dust settles?
    #[must_use]
    pub fn see(self, b: &Board, threshold: i32) -> bool {
        // Iterative approach to SEE inspired by Ethereal.
        let sq_from = self.square_from();
        let sq_to = self.square_to();

        let mut next_victim = if self.is_promotion() {
            match b.side_to_move {
                //only consider queen promotions
                Colour::White => Piece::WQ,
                Colour::Black => Piece::BQ,
            }
        } else {
            self.piece_moved(b)
        };

        let mut balance = match b.pieces_array[sq_to] {
            None => 0,
            Some(k) => SEE_VALUES[piece_type(k)],
        } + threshold;

        if self.is_promotion() {
            balance += SEE_VALUES[PieceType::Queen] - SEE_VALUES[PieceType::Pawn];
        }

        if balance < 0 {
            //bad even in best case
            return false;
        }

        balance -= SEE_VALUES[piece_type(next_victim)];

        if balance >= 0 {
            //good even in worst case
            return true;
        }

        let bishop_attackers = b.bitboards[Piece::WB]
            | b.bitboards[Piece::BB]
            | b.bitboards[Piece::WQ]
            | b.bitboards[Piece::BQ];
        let rook_attackers = b.bitboards[Piece::WR]
            | b.bitboards[Piece::BR]
            | b.bitboards[Piece::WQ]
            | b.bitboards[Piece::BQ];

        let mut occupancies = b.occupancies[OccupancyIndex::BothOccupancies]
            ^ (set_bit(sq_from, 0) | set_bit(sq_to, 0));

        let mut attackers = get_attackers(sq_to, Colour::White, b, occupancies)
            | get_attackers(sq_to, Colour::Black, b, occupancies);

        let mut colour = match b.side_to_move {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };

        loop {
            let side_attackers = attackers
                & b.occupancies[match colour {
                    Colour::White => OccupancyIndex::WhiteOccupancies,
                    Colour::Black => OccupancyIndex::BlackOccupancies,
                }];
            //doesn't matter that actual board struct isn't getting updated because attackers
            //that get traded off will get popped from the attackers bitboard

            if side_attackers == 0 {
                break;
            }

            let pieces = match colour {
                Colour::White => WHITE_PIECES,
                Colour::Black => BLACK_PIECES,
            };

            for piece in pieces {
                if side_attackers & b.bitboards[piece] > 0 {
                    next_victim = piece;
                    break;
                }
            }

            //SAFETY: if this was zero we would have broken above
            occupancies ^= set_bit(
                unsafe { lsfb(side_attackers & b.bitboards[next_victim]).unwrap_unchecked() },
                0,
            );

            if piece_type(next_victim) == PieceType::Pawn
                || piece_type(next_victim) == PieceType::Bishop
                || piece_type(next_victim) == PieceType::Queen
            {
                //only diagonal moves can reveal new diagonal attackers
                attackers |= get_bishop_attacks(sq_to as usize, occupancies) & bishop_attackers;
            }

            if piece_type(next_victim) == PieceType::Rook
                || piece_type(next_victim) == PieceType::Queen
            {
                //same for rook attacks
                attackers |= get_rook_attacks(sq_to as usize, occupancies) & rook_attackers;
            }

            attackers &= occupancies;
            colour = match colour {
                Colour::White => Colour::Black,
                Colour::Black => Colour::White,
            };

            balance = -balance - 1 - SEE_VALUES[piece_type(next_victim)];

            if balance >= 0 {
                //if last move was king move and opponent still has attackers, the move
                //must have been illegal
                if next_victim == Piece::WK
                    && (attackers
                        & b.occupancies[match colour {
                            Colour::White => OccupancyIndex::WhiteOccupancies,
                            Colour::Black => OccupancyIndex::BlackOccupancies,
                        }])
                        > 0
                {
                    colour = match colour {
                        Colour::White => Colour::Black,
                        Colour::Black => Colour::White,
                    };
                }
                break;
            }
        }

        //side to move after the loop loses
        b.side_to_move != colour
    }

    /// Scores a move based on this order
    /// - TT Move
    /// - Queen Promotion
    /// - Good Captures (sorted by MVV-caphist)
    /// - Killers     |
    /// - Quiets      |----- these are also subject to continuation bonuses
    /// - Losing Captures
    /// - Underpromotion
    /// To me it seems intuitive that en passant should be considered a "good capture", but doing
    /// this loses elo. At the moment, en passant just gets the MVV bonus for capturing a pawn.
    pub fn score_move(self, b: &mut Board, s: &Thread, hash_move: &Move) -> i32 {
        if self.is_null() {
            -INFINITY
            //important for this to come before checking hash move
            //otherwise null move can get given hash move score
        } else if self == *hash_move {
            read_param!(HASH_MOVE_SCORE)
            //before pv move because this has been verified by >= search depth
        } else if self.is_capture(b) {
            let victim_type = piece_type(self.piece_captured(b));
            let pc = self.piece_moved(b);
            let good_capture = self.see(b, 0);

            let hist = s.info.caphist_table[pc][self.square_to()][victim_type];

            hist + MVV[victim_type]
                + if good_capture {
                    read_param!(WINNING_CAPTURE)
                } else {
                    read_param!(LOSING_CAPTURE)
                }
        } else if self.is_promotion() {
            //maybe this should fo before checking if capture
            //because of promotions that are also captures
            match self.promoted_piece() {
                //promotions sorted by likelihood to be good
                PieceType::Queen => read_param!(QUEEN_PROMOTION),
                PieceType::Knight => read_param!(UNDER_PROMOTION),
                PieceType::Rook => read_param!(UNDER_PROMOTION),
                PieceType::Bishop => read_param!(UNDER_PROMOTION),
                _ => unreachable!(),
            }
        } else if self.is_en_passant() {
            MVV[PieceType::Pawn]
        } else {
            let cont_bonus = {
                let mut bonus = 0;
                s.info.ss[s.ply].previous_piece.inspect(|x| {
                    s.info.ss[s.ply].previous_square.inspect(|y| {
                        if self == s.info.counter_moves[*x][*y] {
                            bonus = read_param!(COUNTERMOVE_BONUS);
                        }
                    });
                });

                if s.ply >= 1 {
                    s.info.ss[s.ply - 1].previous_piece.inspect(|x| {
                        s.info.ss[s.ply - 1].previous_square.inspect(|y| {
                            if self == s.info.followup_moves[*x][*y] {
                                bonus += read_param!(FOLLOWUP_BONUS);
                            }
                        });
                    });
                }
                bonus
            };

            cont_bonus
                + if s.info.killer_moves[s.ply] == Some(self) {
                    read_param!(FIRST_KILLER_MOVE)
                } else {
                    s.info.history_table[self.piece_moved(b)][self.square_to()]
                }
        }
    }
}

impl MoveList {
    const ALREADY_SEARCHED: i32 = -INFINITY / 2;
    pub fn get_scores(&self, s: &Thread, b: &mut Board, best_move: &Move) -> [i32; MAX_MOVES] {
        let mut scores = [Self::ALREADY_SEARCHED; MAX_MOVES];
        for (i, &m) in self.moves.iter().take_while(|m| !m.is_null()).enumerate() {
            scores[i] = m.score_move(b, s, best_move);
        }

        scores
    }

    /// "Sorts" the moves using insertion sort
    /// In practice this is expected to be faster than O(n log n) sorting since in most cases we
    /// will only have to find a few of the highest scoring moves
    pub fn get_next(&mut self, scores: &mut [i32; MAX_MOVES]) -> Option<(Move, i32)> {
        if scores[0] == Self::ALREADY_SEARCHED {
            return None;
        }

        let (mut best, mut choice, mut idx) = (scores[0], self.moves[0], 0);
        let mut count = 0;
        for (i, &m) in self
            .moves
            .iter()
            .enumerate()
            .take_while(|&(i, _)| scores[i] > Self::ALREADY_SEARCHED)
            .skip(1)
        {
            if scores[i] > best {
                idx = i;
                best = scores[i];
                choice = m;
            }
            count = i;
        }

        self.moves.swap(idx, count);
        scores.swap(idx, count);
        scores[count] = Self::ALREADY_SEARCHED;

        Some((choice, best))
    }
}
