use crate::{
    board::Board,
    search::INFINITY,
    thread::{SearchInfo, Searcher},
    transposition::TranspositionTable,
};

#[rustfmt::skip]
const POSITIONS: [&'static str; 36] = [
    // Kaufman Test positions from CPW (doesn't give 50mr/ply so I just added 0 1 to each)
    "1rbq1rk1/p1b1nppp/1p2p3/8/1B1pN3/P2B4/1P3PPP/2RQ1R1K w - - 0 1",
    "3r2k1/p2r1p1p/1p2p1p1/q4n2/3P4/PQ5P/1P1RNPP1/3R2K1 b - - 0 1",
    "3r2k1/1p3ppp/2pq4/p1n5/P6P/1P6/1PB2QP1/1K2R3 w - - 0 1",
    "r1b1r1k1/1ppn1p1p/3pnqp1/8/p1P1P3/5P2/PbNQNBPP/1R2RB1K w - - 0 1",
    "2r4k/pB4bp/1p4p1/6q1/1P1n4/2N5/P4PPP/2R1Q1K1 b - - 0 1",
    "r5k1/3n1ppp/1p6/3p1p2/3P1B2/r3P2P/PR3PP1/2R3K1 b - - 0 1",
    "2r2rk1/1bqnbpp1/1p1ppn1p/pP6/N1P1P3/P2B1N1P/1B2QPP1/R2R2K1 b - - 0 1",
    "5r1k/6pp/1n2Q3/4p3/8/7P/PP4PK/R1B1q3 b - - 0 1",
    "r3k2r/pbn2ppp/8/1P1pP3/P1qP4/5B2/3Q1PPP/R3K2R w KQkq - 0 1",
    "3r2k1/ppq2pp1/4p2p/3n3P/3N2P1/2P5/PP2QP2/K2R4 b - - 0 1",
    "q3rn1k/2QR4/pp2pp2/8/P1P5/1P4N1/6n1/6K1 w - - 0 1",
    "6k1/p3q2p/1nr3pB/8/3Q1P2/6P1/PP5P/3R2K1 b - - 0 1",
    "1r4k1/7p/5np1/3p3n/8/2NB4/7P/3N1RK1 w - - 0 1",
    "1r2r1k1/p4p1p/6pB/q7/8/3Q2P1/PbP2PKP/1R3R2 w - - 0 1",
    "r2q1r1k/pb3p1p/2n1p2Q/5p2/8/3B2N1/PP3PPP/R3R1K1 w - - 0 1",
    "8/4p3/p2p4/2pP4/2P1P3/1P4k1/1P1K4/8 w - - 0 1",
    "1r1q1rk1/p1p2pbp/2pp1np1/6B1/4P3/2NQ4/PPP2PPP/3R1RK1 w - - 0 1",
    "q4rk1/1n1Qbppp/2p5/1p2p3/1P2P3/2P4P/6P1/2B1NRK1 b - - 0 1",
    "r2q1r1k/1b1nN2p/pp3pp1/8/Q7/PP5P/1BP2RPN/7K w - - 0 1",
    "8/5p2/pk2p3/4P2p/2b1pP1P/P3P2B/8/7K w - - 0 1",
    "8/2k5/4p3/1nb2p2/2K5/8/6B1/8 w - - 0 1",
    "1B1b4/7K/1p6/1k6/8/8/8/8 w - - 0 1",
    "rn1q1rk1/1b2bppp/1pn1p3/p2pP3/3P4/P2BBN1P/1P1N1PP1/R2Q1RK1 b - - 0 1",
    "8/p1ppk1p1/2n2p2/8/4B3/2P1KPP1/1P5P/8 w - - 0 1",
    "8/3nk3/3pp3/1B6/8/3PPP2/4K3/8 w - - 0 1",

    // Zugzwang positions
    "8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w - - 0 1",
    "1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w - - 0 1",

    // A few positions from my own games, just for fun :)
    "2kr3r/2qn2bp/b1p1N3/p1Ppp3/1P1B4/2P5/P2N1PPP/R2QK2R b KQ - 3 16",
    "r2q1rk1/1p2npb1/p1n1b1pp/2ppp3/P3P3/2PP1N1P/1PN2PP1/R1BQRBK1 w - - 1 13",
    "4rnk1/1r2qp1p/4p1p1/3pP2P/Pp3PQ1/3B4/1P4P1/4RR1K b - - 2 26",
    "r3kbnr/1p1b1ppp/p1n1p1q1/2p5/3P1B2/2N2NP1/PPP2PBP/R2Q1K1R w kq - 0 10",
    "4r2k/p1q2pp1/7p/1p6/6P1/P1p2R1P/1bB1r1P1/1Q1R3K b - - 1 36",
    "8/5k2/1n2p2p/1K3p2/8/2B2P2/7P/8 b - - 4 45",
    "8/5p2/4pk1p/8/p1r2PPK/8/PR6/8 w - - 1 41",
    "r1k5/1pb1q3/p2R4/3Q2p1/1PP5/7P/P5P1/7K w - - 1 46",
    "6k1/1p5p/4K3/3R2PP/1r6/8/1P6/8 w - - 1 50",
];

pub fn bench() {
    println!("Starting bench suite for profiling (should take ~40s)");
    let tt = TranspositionTable::in_megabytes(16);
    let mut info = SearchInfo::default();

    for (i, p) in POSITIONS.iter().enumerate() {
        let mut b = Board::from(p);

        let mut s = Searcher::new(&tt, &mut info);
        let _ = s.start_search(&mut b, 0, 0, 0, 1000, INFINITY as usize, 1);
        println!("completed position {}", i + 1);

        tt.clear();
        info = SearchInfo::default();
    }
    println!("Done :)");
}
