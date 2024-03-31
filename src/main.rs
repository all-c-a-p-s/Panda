pub mod board;
pub mod eval;
pub mod helper;
pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod perft;
pub mod rng;
pub mod search;
pub mod uci;
pub mod zobrist;

use crate::board::*;
use crate::eval::evaluate;
use crate::helper::*;
use crate::magic::*;
use crate::movegen::*;
use crate::perft::full_perft;
use crate::r#move::*;
use crate::search::*;
use crate::uci::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

fn main() {
    init_all();

    let debug = false;
    if debug {
        let pos = Board::from(STARTPOS);
        println!("{}", evaluate(&pos));
        full_perft();
        return;
    }
    uci_loop();
}
