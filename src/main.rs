pub mod board;
pub mod helper;
pub mod magic;
pub mod movegen;
pub mod rng;

use crate::board::*;
use crate::magic::*;
use crate::movegen::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

fn main() {
    init_all();
    let pos = fen_to_board("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    gen_moves(pos);
}
