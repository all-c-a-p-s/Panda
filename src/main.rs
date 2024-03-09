pub mod board;
pub mod helper;
pub mod magic;
pub mod rng;

use board::fen_to_board;

use crate::helper::*;
use crate::magic::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

fn main() {
    init_all();

    let mut blockers: u64 = 0;
    blockers = set_bit(square("e5"), blockers);
    let sq = square("e4");
    let rook_attacks = get_rook_attacks(sq, blockers);
    print_bitboard(rook_attacks);
    let pos = fen_to_board("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    println!("{:?}", pos);
}
