pub mod board;
pub mod eval;
pub mod helper;
pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod perft;
pub mod rng;
pub mod search;
pub mod transposition;
pub mod uci;
pub mod zobrist;

use crate::board::*;
use crate::eval::evaluate;
use crate::helper::*;
use crate::magic::*;
use crate::perft::*;
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
        let pos = Board::from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        println!("{}", evaluate(&pos));
        //full_hash_test();
        //let pos = Board::from(STARTPOS);
        //println!("{}", evaluate(&pos));
        see_test();
        full_perft();
    } else {
        uci_loop();
    }
}
