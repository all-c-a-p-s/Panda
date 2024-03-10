pub mod board;
pub mod helper;
pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod perft;
pub mod rng;

use crate::board::*;
use crate::magic::*;
use crate::movegen::*;
use crate::perft::*;
use crate::r#move::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

fn main() {
    init_all();    
    let pos = fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!("{}", perft(4, pos));
}
