use std::time::{Duration, Instant};

use crate::*;

pub const PERFT_DEPTH: usize = 5;

pub fn perft(depth: usize, b: &mut Board) -> usize {
    let mut total = 0;
    let moves = MoveList::gen_legal(b);
    let mut added = 0;
    for i in 0..MAX_MOVES {
        if moves.moves[i].is_null() {
            if depth == 1 {
                return i;
            }
            break;
        }
        if depth != 1 {
            let commit = b.make_move(moves.moves[i]);
            added = perft(depth - 1, b);
            b.undo_move(moves.moves[i], commit);
        }
        /* uncomment for perft debug info
        if depth == PERFT_DEPTH {
            println!(
                "{}{}: {}",
                coordinate(moves.moves[i].square_from()),
                coordinate(moves.moves[i].square_to()),
                added
            );
        }
        */
        total += added;
    }

    total
}

pub fn full_perft() {
    let start = Instant::now();
    let res1 = perft(5, &mut Board::from(STARTPOS));
    assert_eq!(res1, 4_865_609);
    println!("Test 1 - Passed");
    let res2 = perft(
        5,
        &mut Board::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
    );
    assert_eq!(res2, 193_690_690);
    println!("Test 2 - Passed");
    let res3 = perft(
        5,
        &mut Board::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    );
    assert_eq!(res3, 674_624);
    println!("Test 3 - Passed");
    let res4 = perft(
        5,
        &mut Board::from("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
    );
    assert_eq!(res4, 15_833_292);
    println!("Test 4 - Passed");
    let res5 = perft(
        5,
        &mut Board::from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
    );
    assert_eq!(res5, 89_941_194);
    println!("Test 5 - Passed");
    let duration: Duration = start.elapsed();
    println!("Test completed in: {:?}", duration);
}
