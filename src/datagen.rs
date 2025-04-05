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

#[allow(unused)]
fn is_terminal(eval: i32) -> bool {
    eval.abs() > INFINITY / 2
}

fn game_result(found_move: bool, board: &Board, history: &Vec<u64>) -> Option<f32> {
    if !found_move {
        match board.side_to_move {
            Colour::White => return Some(if board.checkers != 0 { 0.0 } else { 0.5 }),
            Colour::Black => return Some(if board.checkers != 0 { 1.0 } else { 0.5 }),
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
    let mut selected_fens = vec![];

    //suggested by creator of bullet that you can add all positions that pass through basic filter
    let first_pick = OPENING_PLIES + 1;
    let pick_interval = 1;

    let mut history = vec![];

    #[allow(unused)]
    let mut moves = MoveList::empty();
    #[allow(unused)]
    let mut result = UNKNOWN_RESULT;

    loop {
        moves = MoveList::gen_moves::<false>(&mut board);
        let mut found_move = false;

        let (mut s, mut chosen_move) = (0, NULL_MOVE);

        let mut best_score = -INFINITY;
        let mut best = NULL_MOVE;

        if board.ply < OPENING_PLIES {
            let mut scores = vec![];

            for m in moves.moves {
                history.push(board.hash_key);
                if m.is_null() {
                    break;
                }

                let Ok(commit) = board.try_move(m) else {
                    continue;
                };

                found_move = true;

                let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(50), 5000);
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
            let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(10), 5000);
            let move_data = best_move(&mut board, 0, 0, 0, 10, &mut searcher, false);

            s = move_data.eval;
            chosen_move = move_data.m;

            if !chosen_move.is_null() {
                found_move = true;
            }
        }

        if let Some(r) = game_result(found_move, &board, &history) {
            result = r;
            break;
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
            && s.abs() < i16::MAX as i32
            && board.checkers == 0
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

        board.play_unchecked(chosen_move);
        assert_eq!(board.nnue, nnue::Accumulator::from_board(&board));
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

fn next_checkpoint(path: &str, duration: Duration) -> Result<i32, std::io::Error> {
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

    Ok(added)
}

//generate data for a set amount of time so that I can leave it generating data when I can
//(for example overnight) and then resume on the same file later instead of having to do it
//all in one go.
pub fn gen_data(path: &str, duration: Duration) -> std::io::Result<()> {
    let mut remaining = duration;

    let mut added = 0;

    //run it in 10 minute chunks so that if I need to exit with <Ctrl-C>
    //I don't lose hours of work
    while remaining > Duration::from_secs(0) {
        let t = std::cmp::min(Duration::from_secs(60 * 10), remaining);
        let added_this_checkpoint = next_checkpoint(path, t)?;

        added += added_this_checkpoint;
        remaining -= t;

        println!(
            "Checkpoint Entries: {}\nAdded so far: {}\nTime remaining: {:?}\n",
            added_this_checkpoint, added, remaining
        );
    }

    println!("Done generating data. {} entries added in total.", added);
    Ok(())
}
