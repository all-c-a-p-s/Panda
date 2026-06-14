pub mod board;
pub mod eval;
pub mod search;
pub mod util;

use std::error::Error;

use crate::board::magic::{get_bishop_attacks, get_rook_attacks, init_slider_attacks};
use crate::board::r#move::{
    CASTLING_FLAG, EN_PASSANT_FLAG, Move, MoveList, NO_FLAG, NULL_MOVE, PROMOTION_FLAG, encode_move,
};
use crate::board::perft::{full_perft, perft};
use crate::board::{BitBoard, Board, Colour};
use crate::search::{INFINITY, MAX_DEPTH, MoveData, iterative_deepening};
use crate::util::bench::prepare_bench;
use crate::util::datagen::gen_data;
use crate::util::helper::{MAX_MOVES, coordinate, lsfb, piece_type, pop_bit, set_bit, square};
use crate::util::uci::{STARTPOS, uci_loop};

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

#[allow(dead_code)]
enum Mode {
    Profile,
    Prep,
    Debug,
    Datagen,
    Uci,
}

#[allow(unused)]
const ONE_HOUR: u64 = 3600;
#[allow(unused)]
const DATAGEN_PATH: &str = "/Users/seba/rs/Panda/set-backtracking-003.txt";

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    init_all();

    let args: Vec<String> = std::env::args().collect();
    let mode_command = args.last().unwrap();

    let mode = match mode_command.as_str() {
        "datagen" => Mode::Datagen,
        "profile" => Mode::Profile,
        "debug" => Mode::Debug,
        "prep" => Mode::Prep,
        _ => Mode::Uci,
    };

    match mode {
        Mode::Uci => uci_loop(),
        Mode::Profile => full_perft(),
        Mode::Datagen => gen_data(DATAGEN_PATH, std::time::Duration::from_secs(ONE_HOUR * 1000))?,
        Mode::Prep => prepare_bench()?,
        Mode::Debug => {}
    }

    Ok(())
}
