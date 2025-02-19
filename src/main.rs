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
pub mod tuner;
pub mod uci;
pub mod zobrist;

use std::error::Error;
use std::time::Instant;

use crate::board::*;
use crate::helper::*;
use crate::magic::*;
use crate::perft::*;
use crate::r#move::*;
use crate::search::*;
use crate::tuner::*;
use crate::uci::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

fn main() -> Result<(), Box<dyn Error>> {
    init_all();

    let debug = false;
    let tune = true;
    let genetic = false;
    let anneal = false;

    if debug {
        //full_hash_test();
        let mut pos = Board::from("r2k1b1r/pp2pppp/8/1B1p4/1q3B2/2n2Q2/P4PPP/2R2RK1 w - - 0 15");
        pos.hash_key = 1;
        let mut s = Searcher::new(Instant::now());
        let res = s.quiescence_search(&mut pos, -INFINITY, INFINITY);
        println!("{}", res);
        see_test();
        full_perft();
    } else if tune {
        if genetic {
            genetic_algorithm()?;
        } else if anneal {
            simulated_annealing()?;
        } else {
            hill_climbing()?;
        }
    } else {
        uci_loop();
    }
    Ok(())
}
