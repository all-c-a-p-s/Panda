pub mod board;
pub mod datagen;
pub mod eval;
pub mod helper;
pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod nnue;
pub mod ordering;
pub mod perft;
pub mod rng;
pub mod search;
pub mod transposition;
pub mod types;
pub mod uci;
pub mod zobrist;

use std::error::Error;

use crate::board::*;
use crate::datagen::*;
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

#[allow(dead_code)]
enum Mode {
    Profile,
    Debug,
    Datagen,
    Uci,
}

#[allow(unused)]
const ONE_HOUR: u64 = 3600;
#[allow(unused)]
const DATAGEN_PATH: &str = "/Users/seba/rs/bullet/datagen/set0004.txt";
//running entry count: 27.3M
//this comment is here so I don't have to load the whole file into a string to count entries
//instead I keep track of the number of entries added each session
//for reference, 1M entries ~= 78MB (txt format)

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    init_all();

    let args: Vec<String> = std::env::args().collect();
    let mode_command = args.last().unwrap();

    let mode = match mode_command.as_str() {
        "datagen" => Mode::Datagen,
        "profile" => Mode::Profile,
        "debug" => Mode::Debug,
        _ => Mode::Uci,
    };

    match mode {
        Mode::Uci => uci_loop(),
        Mode::Profile => full_perft(),
        Mode::Datagen => gen_data(DATAGEN_PATH, std::time::Duration::from_secs(ONE_HOUR / 3))?,
        Mode::Debug => {
            let mut pos =
                Board::from("r2qk2r/2pbppbp/1pn2np1/pB4B1/P2Pp3/1NP3NP/1P3PP1/R2QK2R b KQkq - 3 2");
            let mut searcher = Searcher::new(
                std::time::Instant::now() + std::time::Duration::from_millis(10),
                8192,
            );
            let move_data = iterative_deepening(&mut pos, 0, 0, 0, 10, &mut searcher, false);
            println!("{}", move_data.m.uci());
        }
    };

    Ok(())
}
