use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use crate::*;

const OPENING_CP_MARGIN: i32 = 20;
const OPENING_PLIES: usize = 16;

const BATCH_SIZE: usize = 64;

const UNKNOWN_RESULT: f32 = -1.0;
//if we find this in the data file we know there's an error

//assuming branching factor 5:
// 5^10 = 10e7 different openings
// 5^12 = 10e8     "        "
// 5^14 = 10e10    "        "
// 5^16 = 10e12    "        "

// add the current fen to the list only if the best move in that position is not a capture
// and as long as the eval is not terminal
// this should be a good idea because:
// - its the job of the search, not the evaluation function to catch terminal evals
// - in qsearch we are meant to be evaluating QUIET positions + if we evaluate noisy positions then
//   I suspect there will be a bias against having pieces on squares where captures often occur (like
//   in the centre)

fn is_terminal(eval: i32) -> bool {
    eval.abs() > INFINITY / 2
}

fn game_result(legal_moves: &MoveList, board: &Board, history: &Vec<u64>) -> Option<f32> {
    if legal_moves.moves[0].is_null() {
        match board.side_to_move {
            Colour::White => return Some(if board.is_check() { 0.0 } else { 0.5 }),
            Colour::Black => return Some(if board.is_check() { 1.0 } else { 0.5 }),
        }
    }

    if board.fifty_move == 100 {
        Some(0.5)
    } else if history.iter().filter(|x| **x == board.hash_key).count() == 2 {
        Some(0.5)
    } else {
        None
    }
}

pub fn play_one_game() -> Vec<(String, i32, f32)> {
    let mut board = Board::from(STARTPOS);
    #[allow(unused_mut)]
    let mut selected_fens = vec![];

    let first_pick = OPENING_PLIES + 1;
    let pick_interval = 1;

    let mut history = vec![];

    #[allow(unused)]
    let mut legal_moves = MoveList::empty();
    #[allow(unused)]
    let mut result = UNKNOWN_RESULT;

    loop {
        legal_moves = MoveList::gen_legal(&mut board);

        if let Some(r) = game_result(&legal_moves, &board, &history) {
            result = r;
            break;
        }

        let (mut s, mut chosen_move) = (0, NULL_MOVE);

        let mut best_score = -INFINITY;
        let mut best = NULL_MOVE;

        if board.ply < OPENING_PLIES {
            let mut scores = vec![];

            for m in legal_moves.moves {
                history.push(board.hash_key);
                if m.is_null() {
                    break;
                }

                let (commit, ok) = board.try_move(m);

                if !ok {
                    board.undo_move(m, &commit);
                    continue;
                }

                let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(10));
                searcher.do_pruning = false;

                let score = -searcher.negamax(&mut board, 3, -INFINITY, INFINITY, false);
                scores.push((score, m));

                if score > best_score {
                    best_score = score;
                    best = m;
                }

                board.undo_move(m, &commit);

                (s, chosen_move) = {
                    let candidates = scores
                        .iter()
                        .filter(|x| best_score - OPENING_CP_MARGIN <= x.0)
                        .map(|x| *x)
                        .collect::<Vec<(i32, Move)>>();

                    let i = rand::random::<usize>() % candidates.len();
                    candidates[i]
                };
            }
        } else {
            let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(10));
            let move_data = best_move(&mut board, 0, 0, 0, 10, &mut searcher, false);

            s = move_data.eval;
            chosen_move = move_data.m;
        }

        if chosen_move.is_null() {
            return selected_fens;
        }

        if s > 1000 && count(board.occupancies[BOTH]) < 6 {
            //adjudicate endgames because it won't actually win many of these
            //if its searching to fixed low depth even though theyre completely winning

            result = if board.side_to_move == Colour::White {
                1.0
            } else {
                0.0
            };
            break;
        }

        if board.ply > first_pick
            && (board.ply - first_pick) % pick_interval == 0
            && !best.is_capture(&board)
            && !is_terminal(s)
            && !board.is_check()
        {
            let fen = board.fen();

            let eval = if board.side_to_move == Colour::White {
                s
            } else {
                -s
            };

            let wdl = UNKNOWN_RESULT; //unknown - update later once game finished
            selected_fens.push((fen, eval, wdl));
        }

        board.make_move(chosen_move);
    }

    for x in selected_fens.iter_mut() {
        (*x).2 = result;
    }

    selected_fens
}

pub fn play_multiple_games(num_games: usize, num_threads: usize) -> Vec<(String, i32, f32)> {
    let num_threads = std::cmp::min(num_threads, num_games);

    let games_per_thread = num_games / num_threads;
    let remainder = num_games % num_threads;

    let mut handles = vec![];

    for i in 0..num_threads {
        let thread_games = games_per_thread + if i < remainder { 1 } else { 0 };

        let handle = thread::spawn(move || {
            let mut results = Vec::new();

            for _ in 0..thread_games {
                match std::panic::catch_unwind(|| play_one_game()) {
                    Ok(game_results) => results.extend(game_results),
                    Err(_) => println!("ERROR: a game panicked (skipped)"),
                }
            }

            results
        });

        handles.push(handle);
    }

    let mut all_results = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(thread_results) => all_results.extend(thread_results),
            Err(_) => println!("ERROR: a thread panicked"),
        }
    }

    all_results
}

fn next_checkpoint(path: &str, duration: Duration) -> std::io::Result<()> {
    let mut file = if let Ok(f) = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
    {
        f
    } else {
        std::fs::File::create(path)?
    };

    let mut added = 0;

    let thread_count = match std::thread::available_parallelism() {
        Ok(n) if n.get() > 1 => n.get() - 1, // leave one core for the OS
        Ok(n) => n.get(),
        Err(_) => 1, // fallback to single thread
    };

    println!("Starting data generation with {} threads", thread_count);

    let start = Instant::now();

    let pb = ProgressBar::new(duration.as_secs() as u64);
    pb.set_position(0);

    // Set a nice style for the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    while start.elapsed() < duration {
        let results = play_multiple_games(BATCH_SIZE, thread_count);

        for result in &results {
            writeln!(file, "{} | {} | {:.1}", result.0, result.1, result.2)?;
            added += 1;
        }

        pb.set_position(start.elapsed().as_secs());

        std::thread::sleep(Duration::from_millis(10));
        //apprently this might help cpus to not overheat ... idk
    }
    pb.finish();

    println!("Finished checkpoint. Added {} entries.", added);
    Ok(())
}

//generate data for a set amount of time so that I can leave it generating data when I can
//(for example overnight) and then resume on the same file later instead of having to do it
//all in one go.
pub fn gen_data(path: &str, duration: Duration) -> std::io::Result<()> {
    let mut remaining = duration;

    //run it it
    while remaining > Duration::from_secs(0) {
        let t = std::cmp::min(Duration::from_secs(60 * 10), remaining);
        next_checkpoint(path, t)?;
        remaining -= t;
    }

    Ok(())
}
