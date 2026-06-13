use std::io::Write;

use crate::{
    board::{Board, Colour},
    search::INFINITY,
    thread::{SearchInfo, Searcher},
    transposition::TranspositionTable,
};

/// Play some games from kinda unbalanced exits and record to a file for bench.
/// Don't play an actual pair to get more of a mix of positions, but record FENs once from good
/// side, once from bad side and once as white, once as black.
pub fn prepare_bench() -> Result<(), std::io::Error> {
    let mut file = if let Ok(f) = std::fs::OpenOptions::new().append(true).open("bench.txt") {
        f
    } else {
        std::fs::File::create("bench.txt")?
    };

    let tt = TranspositionTable::in_megabytes(16);
    let mut info = SearchInfo::default();

    let pirc = "r2q1rk1/pp2ppbp/n2p1np1/2pP4/4PPb1/2NB1N2/PPP3PP/R1BQ1RK1 w - - 1 9";
    let dutch = "rnbq1rk1/ppp1b1pp/3pp3/8/2PPp3/5NP1/PP2PPBP/R1BQ1RK1 w - - 0 9";

    let mut b = Board::from(pirc);

    writeln!(file, "uci")?;
    writeln!(file, "isready")?;
    writeln!(file, "ucinewgame\n")?;

    while !b.is_drawn() {
        let mut s = Searcher::new(&tt, &mut info);
        let r = s.start_search(&mut b, 0, 0, 0, 1000, INFINITY as usize, 1);

        if r.m.is_null() {
            break;
        }

        if b.side_to_move == Colour::White {
            writeln!(file, "position fen {}", b.fen())?;
            writeln!(file, "go movetime 1000\n")?;
        }

        b.play_unchecked(r.m);
        b.pretty_print_board();
    }

    b = Board::from(dutch);

    writeln!(file, "isready")?;
    writeln!(file, "ucinewgame\n")?;

    tt.clear();
    info = SearchInfo::default();

    while !b.is_drawn() {
        let mut s = Searcher::new(&tt, &mut info);
        let r = s.start_search(&mut b, 0, 0, 0, 1000, INFINITY as usize, 1);

        if r.m.is_null() {
            break;
        }

        if b.side_to_move == Colour::Black {
            writeln!(file, "position fen {}", b.fen())?;
            writeln!(file, "go movetime 1000\n")?;
        }

        b.play_unchecked(r.m);
        b.pretty_print_board();
    }

    writeln!(file, "quit")?;

    Ok(())
}
