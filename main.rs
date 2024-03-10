pub mod board;
pub mod helper;
pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod rng;

use crate::board::*;
use crate::magic::*;
use crate::movegen::*;
use crate::r#move::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

fn main() {
    init_all();
    let pos = fen_to_board("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    let moves = gen_moves(&pos);
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        moves.moves[i].print_move();
        println!();
    }
}
