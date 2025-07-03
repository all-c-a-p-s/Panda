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

macro_rules! perft {
    ($fen: expr, $depth: expr, $tgt: expr, $idx: expr) => {
        let mut b = Board::from($fen);
        let res = perft::<true>($depth, &mut b, None);
        assert_eq!(res, $tgt);
        println!("Perft Position {}: Passed", $idx);
    };
}

#[rustfmt::skip]
pub fn full_perft() {
    //positions mostly taken from CPW and Ethereal test suite
    let start = Instant::now();

    perft!(STARTPOS, 5, 4_865_609, 1);
    perft!("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690, 2);
    perft!("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674_624, 3);
    perft!("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5, 15_833_292, 4);
    perft!("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194, 5);
    perft!("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551, 6);
    perft!("2bqr3/rp2ppbk/2p2np1/p1Pp3p/N2P1B1P/6P1/PPQRPPB1/4R1K1 b - - 8 21", 5, 40_034_887, 7);
    perft!("r2q1rk1/1p2npb1/p1n1b1pp/2ppp3/PP2P3/2PP1N1P/2N2PP1/R1BQRBK1 b - b3 0 13", 5, 74_778_465, 8);
    perft!("4k3/8/8/8/8/8/8/4K2R w K - 0 1", 6, 764_643, 9);
    perft!("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", 6, 179_862_938, 10);
    perft!("4k3/8/8/8/8/8/8/4K2R b K - 0 1", 6, 899_442, 11);
    perft!("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", 6, 199_002, 12);
    perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", 6, 37_665_329, 13);
    perft!("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", 6, 28_859_283, 14);
    perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", 6, 71_179_139, 15);
    perft!("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", 6, 1_030_499, 16);
    perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", 6, 37_665_329, 17);
    perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6, 71_179_139, 18);
    perft!("k7/8/3p4/8/8/4P3/8/7K w - - 0 1", 6, 28_662, 19);
    perft!("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", 6, 157_093, 20);
    perft!("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", 6, 3_370_175, 21);
    perft!("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", 6, 524_966_748, 22);
    perft!("7k/RR6/8/8/8/8/rr6/7K w - - 0 1", 6, 44_956_585, 23);
    perft!("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", 6, 22_823_890, 24);
    perft!("8/4K3/5P2/1p6/4N2p/1k3n2/6p1/8 w - - 0 53", 6, 19_350_596, 25);

    let duration: Duration = start.elapsed();
    println!("Perft completed in: {:?}", duration);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn perft_wrapper() {
        init_all();
        // I want to be able to call the full_perft() function on its own for profiling
        // so this is a wrapper function for unit tests
        println!("Starting Perft...");
        full_perft();
    }
}
