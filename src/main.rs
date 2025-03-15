pub mod board;
pub mod db;
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
use crate::db::*;
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

#[allow(dead_code)]
enum Mode {
    Profile,
    Debug,
    Tune,
    Db,
    Uci,
}

const MODE: Mode = Mode::Uci;
const TUNING_METHOD: TuneType = TuneType::HillClimb;

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    init_all();

    match MODE {
        Mode::Uci => uci_loop(),
        Mode::Tune => {
            match TUNING_METHOD {
                TuneType::Genetic => genetic_algorithm()?,
                TuneType::Anneal => simulated_annealing()?,
                TuneType::HillClimb => hill_climbing()?,
            };
        }
        Mode::Profile => full_perft(),
        Mode::Debug => {}
        Mode::Db => inspect_db("/Users/seba/rs/Panda/data/2021-07-31-lichess-evaluations-37MM.db")?,
    };

    Ok(())
}
