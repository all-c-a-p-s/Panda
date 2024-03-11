pub mod board;
pub mod helper;
pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod perft;
pub mod rng;

use crate::board::*;
use crate::magic::*;
use crate::movegen::*;
use crate::perft::*;
use crate::r#move::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

fn main() {

    init_all();    
    let pos = fen_to_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

/*
    let moves = gen_legal(pos);
    7
    
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        moves.moves[i].print_move();
    }
*/
    println!("{}", perft(START_DEPTH, pos));
}
