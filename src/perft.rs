use std::time::{Duration, Instant};

use crate::*;

pub fn perft<const BULK: bool>(
    depth: usize,
    b: &mut Board,
    reporting_depth: Option<usize>,
) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut total = 0;
    let moves = MoveList::gen_moves::<false>(b);

    if depth == 1 && BULK {
        let legal = MoveList::gen_legal(b);
        let mut count = 0;
        for m in legal.moves {
            if m.is_null() {
                break;
            }
            count += 1;
            if Some(1) == reporting_depth {
                println!("{}: 1", m.uci());
            }
        }
        return count;
    }

    for m in moves.moves {
        if m.is_null() {
            break;
        }

        let Ok(commit) = b.try_move(m) else {
            continue;
        };

        let added = perft::<BULK>(depth - 1, b, reporting_depth);
        total += added;

        b.undo_move(m, &commit);

        if let Some(d) = reporting_depth {
            if depth == d {
                println!("{}: {}", m.uci(), added);
            }
        }
    }

    total
}

pub fn full_perft() {
    //all positions from CPW perft results page
    let start = Instant::now();
    let res1 = perft::<true>(5, &mut Board::from(STARTPOS), None);
    assert_eq!(res1, 4_865_609);
    println!("Test 1 - Passed");
    let res2 = perft::<true>(
        5,
        &mut Board::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
        None,
    );
    assert_eq!(res2, 193_690_690);
    println!("Test 2 - Passed");
    let res3 = perft::<true>(
        5,
        &mut Board::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
        None,
    );
    assert_eq!(res3, 674_624);
    println!("Test 3 - Passed");
    let res4 = perft::<true>(
        5,
        &mut Board::from("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
        None,
    );
    assert_eq!(res4, 15_833_292);
    println!("Test 4 - Passed");
    let res5 = perft::<true>(
        5,
        &mut Board::from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
        None,
    );
    assert_eq!(res5, 89_941_194);
    println!("Test 5 - Passed");

    let res6 = perft::<true>(
        5,
        &mut Board::from(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        ),
        None,
    );
    assert_eq!(res6, 164_075_551);
    println!("Test 6 - Passed");
    let duration: Duration = start.elapsed();
    println!("Test completed in: {:?}", duration);
}
