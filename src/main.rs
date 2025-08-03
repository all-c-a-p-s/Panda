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
pub mod thread;
pub mod transposition;
pub mod types;
pub mod uci;
pub mod zobrist;

use std::error::Error;

use crate::board::{BitBoard, Board, Colour};
use crate::datagen::gen_data;
use crate::helper::{coordinate, lsfb, piece_type, pop_bit, set_bit, square, MAX_MOVES};
use crate::magic::{get_bishop_attacks, get_rook_attacks, init_slider_attacks};
use crate::perft::{full_perft, perft};
use crate::r#move::{
    encode_move, Move, MoveList, CASTLING_FLAG, EN_PASSANT_FLAG, NO_FLAG, NULL_MOVE, PROMOTION_FLAG,
};
use crate::search::{iterative_deepening, MoveData, INFINITY, MAX_GAME_PLY, MAX_PLY};
use crate::uci::{uci_loop, STARTPOS};

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
const DATAGEN_PATH: &str = "/Users/seba/rs/bullet/datagen/set-backtracking-002.txt";
//running entry count: 0
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
        Mode::Datagen => gen_data(DATAGEN_PATH, std::time::Duration::from_secs(ONE_HOUR * 100))?,
        Mode::Debug => {}
    }

    Ok(())
}
