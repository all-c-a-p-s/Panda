pub mod nnue;

pub use nnue::*;

use crate::board::{Board, Colour};
use crate::search::MATE;
use crate::util::helper::rank;

use crate::util::types::{Piece, Square};

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

// technically misses some edge cases like 2 bishops of the same colour from promotions
// or 2 knights vs pawn, in the cases that it's winning
pub fn side_has_sufficient_material(b: &Board, side: Colour) -> bool {
    let (pawns, knights, bishops, rooks, queens) = match side {
        Colour::White => (
            b.bitboards[Piece::WP],
            b.bitboards[Piece::WN],
            b.bitboards[Piece::WB],
            b.bitboards[Piece::WR],
            b.bitboards[Piece::WQ],
        ),
        Colour::Black => (
            b.bitboards[Piece::BP],
            b.bitboards[Piece::BN],
            b.bitboards[Piece::BB],
            b.bitboards[Piece::BR],
            b.bitboards[Piece::BQ],
        ),
    };

    if pawns | rooks | queens > 0 {
        return true;
    }

    if bishops.count_ones() >= 2 || (knights | bishops).count_ones() >= 3 {
        return true;
    }

    bishops > 0 && knights > 0
}

#[must_use]
pub fn evaluate(b: &Board, acc: &Accumulator) -> i32 {
    let s = acc.evaluate(b.side_to_move);

    let side_sm = side_has_sufficient_material(b, b.side_to_move);
    let opp_sm = side_has_sufficient_material(b, b.side_to_move.opponent());

    let r = if side_sm && opp_sm {
        s
    } else if side_sm {
        s.max(0)
    } else {
        s.min(0)
    };

    r.clamp(-MATE, MATE)
}
