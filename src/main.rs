pub mod board;
pub mod datagen;
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
pub mod uncertainty;
pub mod zobrist;

use std::error::Error;
use std::time::Instant;

use crate::board::*;
use crate::datagen::*;
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
    Uncertainty,
    Datagen,
    Db,
    Uci,
}

const MODE: Mode = Mode::Uci;
const TUNING_METHOD: TuneType = TuneType::Genetic;

#[allow(unused)]
const ONE_HOUR: u64 = 3600;
const DATAGEN_PATH: &'static str = "/Users/seba/rs/Panda/marlinflow/trainer/data/data230325.txt";
//running entry count: 24.3M
//this comment is here so I don't have to load the whole file into a string to count entries
//instead I keep track of the number of entries added each session
//for reference, 1M entries ~= 78MB

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
        Mode::Uncertainty => uncertainty::tune_one_by_one(),
        Mode::Profile => full_perft(),
        Mode::Datagen => gen_data(DATAGEN_PATH, std::time::Duration::from_secs(ONE_HOUR * 20))?,
        Mode::Debug => {}
        Mode::Db => inspect_db("/Users/seba/rs/Panda/data/2021-07-31-lichess-evaluations-37MM.db")?,
    };

    Ok(())
}
