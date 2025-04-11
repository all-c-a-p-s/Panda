use crate::movegen::*;
use crate::r#move::*;
use crate::search::*;
use crate::types::*;
use crate::*;

const HASH_MOVE_SCORE: i32 = 1_000_000;
const PV_MOVE_SCORE: i32 = 500_000;
const QUEEN_PROMOTION: i32 = 400_000;
const WINNING_CAPTURE: i32 = 300_000;
const FIRST_KILLER_MOVE: i32 = 200_000;
const SECOND_KILLER_MOVE: i32 = 100_000;
const LOSING_CAPTURE: i32 = -100_000;
const UNDER_PROMOTION: i32 = -200_000;

const MVV_LVA: [[i32; 6]; 6] = [
    //most valuable victim least valuable attacker
    [205, 204, 203, 202, 201, 200], //victim pawn
    [305, 304, 303, 302, 301, 300], //victim knight
    [405, 404, 403, 402, 401, 400], //victim bishop
    [505, 504, 503, 502, 501, 500], //victim rook
    [605, 604, 603, 602, 601, 600], //victim queen
    [0, 0, 0, 0, 0, 0],             //victim king
];

pub fn see_test() {
    let position1 = Board::from("8/7k/8/4p3/8/5N2/K7/8 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res1 = m.static_exchange_evaluation(&position1, 0);
    assert!(res1, "first see test position failed");

    let position2 = Board::from("8/2b4k/8/4p3/8/5N2/K7/8 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res2 = m.static_exchange_evaluation(&position2, 0);
    assert!(!res2, "second see test position failed");

    let position3 = Board::from("8/2b4k/8/4p3/8/5N2/K7/4R3 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res3 = m.static_exchange_evaluation(&position3, 0);
    assert!(res3, "third see test position failed");

    let position4 = Board::from("4q3/2b4k/8/4p3/8/5N2/K7/4R3 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res4 = m.static_exchange_evaluation(&position4, 0);
    assert!(!res4, "fourth see test position failed");

    let position5 = Board::from("4q3/2b4k/8/4p3/8/5N2/K7/Q3R3 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res5 = m.static_exchange_evaluation(&position5, 0);
    assert!(res5, "fifth see test position failed");

    //test start position with no captures
    let position6 = Board::from(STARTPOS);
    let m = encode_move(Square::E2, Square::E4, None, NO_FLAG);
    let res6 = m.static_exchange_evaluation(&position6, 0);
    assert!(res6, "sixth see test position failed");

    let position7 = Board::from("4k3/8/2n2b2/8/3P4/2P5/8/3K4 b - - 0 1");
    let m = encode_move(Square::C6, Square::D4, None, NO_FLAG);
    let res7 = m.static_exchange_evaluation(&position7, 0);
    assert!(!res7, "seventh see test position failed");

    //test sliding attack updates
    let position8 = Board::from("3q3k/3r4/3r4/3p4/8/3R4/3R4/3Q3K w - - 0 1");
    let m = encode_move(Square::D3, Square::D5, None, NO_FLAG);
    let res8 = m.static_exchange_evaluation(&position8, 0);
    assert!(!res8, "eighth see test position failed");

    let position9 = Board::from("7k/8/3r4/3p4/4P3/5B2/8/7K w - - 0 1");
    let m = encode_move(Square::E4, Square::D5, None, NO_FLAG);
    let res9 = m.static_exchange_evaluation(&position9, 0);
    assert!(res9, "ninth see test position failed");

    println!("see test passed");
}

//same as MG evaluation weights (haven't updated these in a while)
pub const SEE_VALUES: [i32; 6] = [85, 306, 322, 490, 925, INFINITY];

impl Move {
    pub fn static_exchange_evaluation(self, b: &Board, threshold: i32) -> bool {
        /*
         Iterative approach to SEE inspired by engine Ethereal. This is much faster
         than the recursive implementation I tried to make becuase most of the attack
         bitboards won't change during the SEE search so it's faster to keep them and
         only update slider attack bitboards when it's possible that they changed.
         This also avoids using make_move() and undo_move().
        */
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

    pub fn score_move(self, b: &mut Board, s: &Searcher, hash_move: &Move) -> i32 {
        /*
          MOVE ORDER:
        - TT Move
        - PV Move
        - Queen Promotion
        - Winning Capture + E.P.
        - Killers
        - History
        - Losing Capture
        - Underpromotion
         */

        if self.is_null() {
            -INFINITY
            //important for this to come before checking hash move
            //otherwise null move can get given hash move score
        } else if self == *hash_move {
            HASH_MOVE_SCORE
            //before pv move because this has been verified by >= search depth
        } else if self == s.pv[0][s.ply] {
            PV_MOVE_SCORE
        } else if self.is_capture(b) {
            let victim_type =
                piece_type(unsafe { b.pieces_array[self.square_to()].unwrap_unchecked() });
            let attacker_type = piece_type(self.piece_moved(b));
            let winning_capture = self.static_exchange_evaluation(b, 0);
            match winning_capture {
                true => WINNING_CAPTURE + MVV_LVA[victim_type][attacker_type],
                false => LOSING_CAPTURE + MVV_LVA[victim_type][attacker_type],
            }
        } else if self.is_promotion() {
            //maybe this should fo before checking if capture
            //because of promotions that are also captures
            match self.promoted_piece() {
                //promotions sorted by likelihood to be good
                PieceType::Queen => QUEEN_PROMOTION,
                PieceType::Knight => UNDER_PROMOTION,
                PieceType::Rook => UNDER_PROMOTION,
                PieceType::Bishop => UNDER_PROMOTION,
                _ => unreachable!(),
            }
        } else if self.is_en_passant() {
            MVV_LVA[PieceType::Pawn][PieceType::Pawn]
        } else if s.info.killer_moves[0][s.ply] == self {
            FIRST_KILLER_MOVE //after captures
        } else if s.info.killer_moves[1][s.ply] == self {
            SECOND_KILLER_MOVE
        } else {
            s.info.history_table[self.piece_moved(b)][self.square_to()]
        }
    }
}

#[derive(Copy, Clone)]
pub struct MoveOrderEntry<'a> {
    m: &'a Move,
    score: i32,
}

impl MoveList {
    pub fn order_moves(&mut self, board: &mut Board, s: &Searcher, best_move: &Move) {
        let mut ordered_moves = [MoveOrderEntry {
            m: &NULL_MOVE,
            score: -INFINITY,
        }; MAX_MOVES];

        for (i, m) in self.moves.iter().enumerate() {
            if m.is_null() {
                break;
            }
            ordered_moves[i].m = m;
            ordered_moves[i].score = m.score_move(board, s, best_move);
        }

        ordered_moves.sort_by(|a, b| b.score.cmp(&a.score));

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
