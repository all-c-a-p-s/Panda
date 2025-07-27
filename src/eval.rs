use crate::board::{Board, Colour};
use crate::helper::{count, rank};

use crate::types::{Piece, Square};

pub const MIRROR: [usize; 64] = {
    const fn relative_psqt_square(square: usize, c: Colour) -> usize {
        match c {
            Colour::White => {
                //piece-square tables have a1 on bottom left -> a8 at index 0
                let relative_rank = 7 - rank(unsafe { Square::from(square as u8) });
                let file = square % 8;
                relative_rank * 8 + file
            }
            Colour::Black => square,
        }
    }
    let mut mirror = [0usize; 64];
    let mut square = 0;
    while square < 64 {
        mirror[square] = relative_psqt_square(square, Colour::White);
        square += 1;
    }
    mirror
};

fn side_has_sufficient_matieral(b: &Board, side: Colour) -> bool {
    match side {
        Colour::White => {
            count(b.bitboards[Piece::WP]) > 0
                || count(b.bitboards[Piece::WR]) > 0
                || count(b.bitboards[Piece::WQ]) > 0
                || count(b.bitboards[Piece::WB]) > 1
                || count(b.bitboards[Piece::WN]) > 2
            //NOTE: atm this ignores exception of 2N vs P but I don't think we or our opponents can
            //win that anyway
        }
        Colour::Black => {
            count(b.bitboards[Piece::BP]) > 0
                || count(b.bitboards[Piece::BR]) > 0
                || count(b.bitboards[Piece::BQ]) > 0
                || count(b.bitboards[Piece::BB]) > 1
                || count(b.bitboards[Piece::BN]) > 2
        }
    }
}

#[must_use] pub fn evaluate(b: &Board) -> i32 {
    let s = b.nnue.evaluate(b.side_to_move);

    //TODO: endgame tablebase for better draw detection
    let side_sm = side_has_sufficient_matieral(b, b.side_to_move);
    let opp_sm = side_has_sufficient_matieral(b, b.side_to_move.opponent());

    if side_sm && opp_sm {
        s
    } else if side_sm {
        std::cmp::max(s, 0)
    } else {
        std::cmp::min(s, 0)
    }
}
