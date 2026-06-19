use std::time::{Duration, Instant};

use crate::{Board, MoveList, STARTPOS, board::movegen::MovegenMode};

pub fn perft<const BULK: bool, const TEST_PSEUDOLEGAL: bool, const MODES: bool>(
    depth: usize,
    b: &mut Board,
    reporting_depth: Option<usize>,
) -> usize {
    if depth == 0 {
        return 1;
    }

    if depth == 1 && BULK {
        let legal = MoveList::gen_legal(b);
        return legal.used;
    }

    let mut total = 0;
    let mut moves = MoveList::empty();
    if MODES {
        moves.gen_moves(b, MovegenMode::NoisyQueenPromotions);
        moves.gen_moves(b, MovegenMode::QuietQueenPromotions);
        moves.gen_moves(b, MovegenMode::CapsOnly);
        moves.gen_moves(b, MovegenMode::QuietsOnly);
        moves.gen_moves(b, MovegenMode::NoisyUnderpromotions);
        moves.gen_moves(b, MovegenMode::QuietUnderpromotions);
    } else {
        moves.gen_moves(b, MovegenMode::All);
    }

    for &mv in moves.moves.iter().take(moves.used) {
        if TEST_PSEUDOLEGAL && !b.is_pseudo_legal(mv) {
            mv.print_move();
            b.print_board();
            panic!("pseudo-legal move wasn't pseudo-legal :(");
        }

        let Ok(commit) = b.try_move(mv, None) else {
            continue;
        };

        let added = perft::<BULK, TEST_PSEUDOLEGAL, MODES>(depth - 1, b, reporting_depth);
        total += added;

        b.undo_move(mv, &commit, None);

        if let Some(d) = reporting_depth
            && depth == d
        {
            println!("{}: {}", mv.uci(), added);
        }
    }

    total
}

macro_rules! perft {
    ($fen: expr, $depth: expr, $tgt: expr, $idx: expr, $plegal: expr, $modes: expr) => {
        let mut b = Board::from($fen);
        let res = perft::<true, $plegal, $modes>($depth, &mut b, None);
        assert_eq!(res, $tgt);
        println!("Perft Position {}: Passed", $idx);
    };
}

#[rustfmt::skip]
pub fn full_perft() {
    //positions mostly taken from CPW and Ethereal test suite
    let start = Instant::now();

    perft!(STARTPOS, 5, 4_865_609, 1, false, false);
    perft!("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690, 2, false, false);
    perft!("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674_624, 3, false, false);
    perft!("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5, 15_833_292, 4, false, false);
    perft!("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194, 5, false, false);
    perft!("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551, 6, false, false);
    perft!("2bqr3/rp2ppbk/2p2np1/p1Pp3p/N2P1B1P/6P1/PPQRPPB1/4R1K1 b - - 8 21", 5, 40_034_887, 7, false, false);
    perft!("r2q1rk1/1p2npb1/p1n1b1pp/2ppp3/PP2P3/2PP1N1P/2N2PP1/R1BQRBK1 b - b3 0 13", 5, 74_778_465, 8, false, false);
    perft!("4k3/8/8/8/8/8/8/4K2R w K - 0 1", 6, 764_643, 9, false, false);
    perft!("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", 6, 179_862_938, 10, false, false);
    perft!("4k3/8/8/8/8/8/8/4K2R b K - 0 1", 6, 899_442, 11, false, false);
    perft!("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", 6, 199_002, 12, false, false);
    perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", 6, 37_665_329, 13, false, false);
    perft!("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", 6, 28_859_283, 14, false, false);
    perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", 6, 71_179_139, 15, false, false);
    perft!("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", 6, 1_030_499, 16, false, false);
    perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", 6, 37_665_329, 17, false, false);
    perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6, 71_179_139, 18, false, false);
    perft!("k7/8/3p4/8/8/4P3/8/7K w - - 0 1", 6, 28_662, 19, false, false);
    perft!("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", 6, 157_093, 20, false, false);
    perft!("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", 6, 3_370_175, 21, false, false);
    perft!("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", 6, 524_966_748, 22, false, false);
    perft!("7k/RR6/8/8/8/8/rr6/7K w - - 0 1", 6, 44_956_585, 23, false, false);
    perft!("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", 6, 22_823_890, 24, false, false);
    perft!("8/4K3/5P2/1p6/4N2p/1k3n2/6p1/8 w - - 0 53", 6, 19_350_596, 25, false, false);

    let duration: Duration = start.elapsed();
    println!("Perft completed in: {duration:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[test]
    pub fn perft_wrapper() {
        init_all();
        // I want to be able to call the full_perft() function on its own for profiling
        // so this is a wrapper function for unit tests
        println!("Starting Perft...");
        full_perft();
    }

    #[test]
    #[rustfmt::skip]
    pub fn full_perft_with_pseudo_legal_tests() {
        init_all();

        perft!(STARTPOS, 5, 4_865_609, 1, true, false);
        perft!("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690, 2, true, false);
        perft!("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674_624, 3, true, false);
        perft!("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5, 15_833_292, 4, true, false);
        perft!("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194, 5, true, false);
        perft!("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551, 6, true, false);
        perft!("2bqr3/rp2ppbk/2p2np1/p1Pp3p/N2P1B1P/6P1/PPQRPPB1/4R1K1 b - - 8 21", 5, 40_034_887, 7, true, false);
        perft!("r2q1rk1/1p2npb1/p1n1b1pp/2ppp3/PP2P3/2PP1N1P/2N2PP1/R1BQRBK1 b - b3 0 13", 5, 74_778_465, 8, true, false);
        perft!("4k3/8/8/8/8/8/8/4K2R w K - 0 1", 6, 764_643, 9, true, false);
        perft!("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", 6, 179_862_938, 10, true, false);
        perft!("4k3/8/8/8/8/8/8/4K2R b K - 0 1", 6, 899_442, 11, true, false);
        perft!("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", 6, 199_002, 12, true, false);
        perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", 6, 37_665_329, 13, true, false);
        perft!("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", 6, 28_859_283, 14, true, false);
        perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", 6, 71_179_139, 15, true, false);
        perft!("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", 6, 1_030_499, 16, true, false);
        perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", 6, 37_665_329, 17, true, false);
        perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6, 71_179_139, 18, true, false);
        perft!("k7/8/3p4/8/8/4P3/8/7K w - - 0 1", 6, 28_662, 19, true, false);
        perft!("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", 6, 157_093, 20, true, false);
        perft!("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", 6, 3_370_175, 21, true, false);
        perft!("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", 6, 524_966_748, 22, true, false);
        perft!("7k/RR6/8/8/8/8/rr6/7K w - - 0 1", 6, 44_956_585, 23, true, false);
        perft!("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", 6, 22_823_890, 24, true, false);
        perft!("8/4K3/5P2/1p6/4N2p/1k3n2/6p1/8 w - - 0 53", 6, 19_350_596, 25, true, false);
    }

    #[test]
    #[rustfmt::skip]
    pub fn full_perft_with_modes() {
        init_all();

        perft!(STARTPOS, 5, 4_865_609, 1, true, true);
        perft!("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690, 2, true, true);
        perft!("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674_624, 3, true, true);
        perft!("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5, 15_833_292, 4, true, true);
        perft!("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194, 5, true, true);
        perft!("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551, 6, true, true);
        perft!("2bqr3/rp2ppbk/2p2np1/p1Pp3p/N2P1B1P/6P1/PPQRPPB1/4R1K1 b - - 8 21", 5, 40_034_887, 7, true, true);
        perft!("r2q1rk1/1p2npb1/p1n1b1pp/2ppp3/PP2P3/2PP1N1P/2N2PP1/R1BQRBK1 b - b3 0 13", 5, 74_778_465, 8, true, true);
        perft!("4k3/8/8/8/8/8/8/4K2R w K - 0 1", 6, 764_643, 9, true, true);
        perft!("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", 6, 179_862_938, 10, true, true);
        perft!("4k3/8/8/8/8/8/8/4K2R b K - 0 1", 6, 899_442, 11, true, true);
        perft!("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", 6, 199_002, 12, true, true);
        perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", 6, 37_665_329, 13, true, true);
        perft!("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", 6, 28_859_283, 14, true, true);
        perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", 6, 71_179_139, 15, true, true);
        perft!("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", 6, 1_030_499, 16, true, true);
        perft!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", 6, 37_665_329, 17, true, true);
        perft!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6, 71_179_139, 18, true, true);
        perft!("k7/8/3p4/8/8/4P3/8/7K w - - 0 1", 6, 28_662, 19, true, true);
        perft!("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", 6, 157_093, 20, true, true);
        perft!("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", 6, 3_370_175, 21, true, true);
        perft!("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", 6, 524_966_748, 22, true, true);
        perft!("7k/RR6/8/8/8/8/rr6/7K w - - 0 1", 6, 44_956_585, 23, true, true);
        perft!("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", 6, 22_823_890, 24, true, true);
        perft!("8/4K3/5P2/1p6/4N2p/1k3n2/6p1/8 w - - 0 53", 6, 19_350_596, 25, true, true);
    }
}
