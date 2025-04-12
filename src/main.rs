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

use std::io::Write;

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

const MODE: Mode = Mode::Uci;

#[allow(unused)]
const ONE_HOUR: u64 = 3600;
const DATAGEN_PATH: &str = "/Users/seba/rs/bullet/datagen/set0003.txt";
//running entry count: 27.3M
//this comment is here so I don't have to load the whole file into a string to count entries
//instead I keep track of the number of entries added each session
//for reference, 1M entries ~= 78MB (txt format)

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    init_all();

    match MODE {
        Mode::Uci => uci_loop(),
        Mode::Profile => full_perft(),
        Mode::Datagen => gen_data(DATAGEN_PATH, std::time::Duration::from_secs(ONE_HOUR * 40))?,
        Mode::Debug => {
            fn parse_line(line: &str) -> Option<(String, i16, f32)> {
                let parts = line.split("|").map(|x| x.trim()).collect::<Vec<_>>();
                let fen = parts[0].to_string();
                let eval = match parts[1].parse::<i16>() {
                    Ok(k) => k,
                    Err(_) => return None,
                };
                let wdl = match parts[2].parse::<f32>() {
                    Ok(k) => k,
                    Err(_) => return None,
                };

                Some((fen, eval, wdl))
            }

            let path = "/Users/seba/rs/bullet/datagen/set0003_fixed.txt";
            let mut file = if let Ok(f) = std::fs::OpenOptions::new().append(true).open(path) {
                f
            } else {
                std::fs::File::create(path)?
            };

            let s = std::fs::read_to_string(DATAGEN_PATH)?;

            let data = s.lines().filter_map(parse_line);

            let mut count = 0;
            for x in data {
                writeln!(file, "{} | {} | {:.1}", x.0, x.1, x.2)?;
                count += 1;
                if count % 100_000 == 0 {
                    println!("{}", count);
                }
            }
        }
    };

    Ok(())
}
