use crate::movegen::*;
use crate::nmm::*;
use crate::r#move::*;
use crate::search::*;
use crate::thread::*;
use crate::types::*;
use crate::*;

//same as MG evaluation weights (haven't updated these in a while)
pub const SEE_VALUES: [i32; 6] = [85, 306, 322, 490, 925, INFINITY];

impl Move {
    pub fn see(self, b: &Board, threshold: i32) -> bool {
        // Iterative approach to SEE inspired by Ethereal.
        let sq_from = self.square_from();
        let sq_to = self.square_to();

        let mut next_victim = match self.is_promotion() {
            true => match b.side_to_move {
                //only consider queen promotions
                Colour::White => Piece::WQ,
                Colour::Black => Piece::BQ,
            },
            false => self.piece_moved(b),
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

        let mut occupancies = b.occupancies[BOTH] ^ (set_bit(sq_from, 0) | set_bit(sq_to, 0));

        let mut attackers = get_attackers(sq_to, Colour::White, b, occupancies)
            | get_attackers(sq_to, Colour::Black, b, occupancies);

        let mut colour = match b.side_to_move {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };

        loop {
            let side_attackers = attackers
                & b.occupancies[match colour {
                    Colour::White => WHITE,
                    Colour::Black => BLACK,
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
                            Colour::White => WHITE,
                            Colour::Black => BLACK,
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

    pub fn score_move(self, b: &mut Board, s: &Thread, hash_move: &Move) -> f32 {
        if self == NULL_MOVE {
            -1.0
        } else if self == *hash_move {
            1.0
        } else {
            let is = make_input_state(b, self);
            forward_prediction(&s.info.policy.net, &is).expect("failed to pass forward") as f32
        }
    }
}

#[derive(Copy, Clone)]
pub struct MoveOrderEntry<'a> {
    m: &'a Move,
    score: f32,
}

impl MoveList {
    pub fn order_moves(&mut self, board: &mut Board, s: &Thread, best_move: &Move) {
        let mut ordered_moves = [MoveOrderEntry {
            m: &NULL_MOVE,
            score: -1.0,
        }; MAX_MOVES];

        for (i, m) in self.moves.iter().enumerate() {
            if m.is_null() {
                break;
            }
            ordered_moves[i].m = m;
            ordered_moves[i].score = m.score_move(board, s, best_move);
        }

        ordered_moves.sort_by(|a, b| b.score.partial_cmp(&a.score).expect("no ordering"));

        let mut final_moves = [NULL_MOVE; MAX_MOVES];

        for i in 0..MAX_MOVES {
            if ordered_moves[i].m.is_null() {
                break;
            }
            final_moves[i] = *ordered_moves[i].m;
        }
        self.moves = final_moves
    }
}
