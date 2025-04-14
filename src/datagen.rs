use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::fmt::{self, Display};
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use crate::*;

const OPENING_CP_MARGIN: i32 = 20;
const OPENING_PLIES: usize = 16;

const BATCH_SIZE: usize = 64;

//if we find this in the data file we know there's an error

// assuming branching factor 5:
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

pub struct Game {
    positions: Vec<Node>,
}

// consider 3 nodes A, B, C
// let the value of a node denoted by some letter X (with respect to the side to move at X) be v_x
// we define A as
//      - definitely misevaluated is v_c < v_b < v_a
//      - maybe misevaluated if v_b < v_c < v_a
//
// if it is maybe misevaluated then we randomise based on WDLs to determine whether or not to class
// it as misevaluated so that P(will be misevaluated) = [wdl(v_a) - wdl(v_c)] / [wdl(v_a) - wdl(v_b)]
// where the wdl() function is the same as the one used for training the value network
//
// if A is misevaluated then we re-score it as follows:
//  - compute our second choice of move at A and play it on the board
//  - let S be the value of this position from our opponent's perspective
//  - reassign value of A to be max(-v_b, S)

#[derive(Clone, Copy)]
pub struct Node {
    position: Board,
    value: i32, //note these are from perspective of STM
    choice: Option<Move>,
    result: Option<f32>,
}

impl Node {
    pub fn from_position(pos: &Board) -> Self {
        Self {
            position: *pos,
            value: 0,
            choice: None,
            result: None,
        }
    }

    // this function merely needs to determine the value of the node, not of its moves
    fn value(&mut self) -> i32 {
        let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(10), 16384);
        let move_data = iterative_deepening(&mut self.position, 0, 0, 0, 10, &mut searcher, false);
        move_data.eval
    }

    pub fn choose_move(&mut self) {
        let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(10), 16384);
        let move_data = iterative_deepening(&mut self.position, 0, 0, 0, 10, &mut searcher, false);

        self.choice = Some(move_data.m);
        self.value = move_data.eval;
    }

    pub fn choose_opening_move(&mut self) {
        let movelist = MoveList::gen_moves::<false>(&self.position);

        let (mut s, mut chosen_move) = (0, NULL_MOVE);

        let mut best_score = -INFINITY;

        // in the opening do a very shallow search of all legal moves in the root
        // then pick from the moves which are within some margin from the best one
        let mut scores = vec![];

        for m in movelist.moves {
            if m.is_null() {
                break;
            }

            let Ok(commit) = self.position.try_move(m) else {
                continue;
            };

            // node limit should be a multiple of 4096
            let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(10), 8192);
            searcher.do_pruning = false;

            let score = -searcher.negamax(&mut self.position, 3, -INFINITY, INFINITY, false);
            scores.push((score, m));
            best_score = best_score.max(score);

            self.position.undo_move(m, &commit);

            (s, chosen_move) = {
                let candidates = scores
                    .iter()
                    .filter(|&x| best_score - OPENING_CP_MARGIN <= x.0)
                    .copied()
                    .collect::<Vec<(i32, Move)>>();

                let i = rand::random::<usize>() % candidates.len();
                candidates[i]
            };
        }

        self.value = s;
        self.choice = Some(chosen_move);
    }

    // must be called when choice is not None and when choice is not the only legal move
    pub fn choose_second(&mut self) {
        let m = self.choice.unwrap();

        let mut searcher = Searcher::new(Instant::now() + Duration::from_millis(10), 8192);
        searcher.info.excluded[0] = Some(m);
        let move_data = iterative_deepening(&mut self.position, 0, 0, 0, 10, &mut searcher, false);
        self.choice = Some(move_data.m);
    }
}

impl Game {
    fn new() -> Self {
        let b = Board::from(STARTPOS);
        let n = Node::from_position(&b);
        Self { positions: vec![n] }
    }

    fn next(&mut self) -> Result<bool, ()> {
        let mut pos = self.positions.last().unwrap().position;
        let movelist = MoveList::gen_legal(&mut pos);
        let repetition_table = self
            .positions
            .iter()
            .map(|x| x.position.hash_key)
            .collect::<Vec<u64>>();

        let found_move = !movelist.moves[0].is_null();
        let leaf = self.positions.last_mut().unwrap();
        if let Some(res) = game_result(found_move, &pos, &repetition_table) {
            leaf.result = Some(res);
            leaf.value = match res {
                0.0 => -INFINITY,
                0.5 => 0,
                1.0 => INFINITY,
                _ => unreachable!(),
            };
            return Ok(false);
        }

        if pos.ply < OPENING_PLIES {
            leaf.choose_opening_move();
        } else {
            leaf.choose_move();
        }

        if leaf.choice.unwrap().is_null() {
            return Err(());
        }

        pos.play_unchecked(leaf.choice.unwrap());
        let child = Node::from_position(&pos);

        self.positions.push(child);
        Ok(true)
    }

    pub fn generate() -> Option<Self> {
        let mut g = Self::new();
        loop {
            let r = g.next();
            if let Ok(q) = r {
                if q == false {
                    break;
                }
            } else {
                return None;
            }
        }
        g.backtrack();
        Some(g)
    }

    // the purpose of the backtracking algorithm is to try to use the information we gained by
    // playing the game to more accurately score the nodes in the game
    fn backtrack(&mut self) {
        use rand::Rng;
        fn wdl(x: i32) -> f32 {
            1.0 / (1.0 + ((-x as f32) * 2.55 / 400.0).exp())
        }

        for ply in (OPENING_PLIES..self.positions.len()).rev() {
            if ply != self.positions.len() - 1 {
                self.positions[ply].result = self.positions[ply + 1].result;
            }

            if ply >= self.positions.len() - 2 {
                continue;
            }

            let (a, b, c) = (
                self.positions[ply],
                self.positions[ply + 1],
                self.positions[ply + 2],
            );

            let (v_a, v_b, v_c) = (a.value, -b.value, c.value);

            let misevaluated = if v_b >= v_a || v_c >= v_a {
                false
            } else if v_a > v_b && v_b >= v_c {
                true
            } else {
                let delta_b = wdl(v_a) - wdl(v_b);
                let delta_c = wdl(v_a) - wdl(v_c);

                if delta_b >= delta_c {
                    false
                } else {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(0.0..delta_b) < delta_c
                }
            };

            if misevaluated {
                let p = self.positions.get_mut(ply).unwrap();

                let mut pos = p.position;

                let movelist = MoveList::gen_legal(&mut pos);
                let it = movelist.moves.iter().filter(|x| !x.is_null());

                if it.count() != 1 {
                    p.choose_second();
                    if !p.choice.unwrap().is_null() {
                        pos.play_unchecked(p.choice.unwrap());
                        let mut n = Node::from_position(&pos);
                        let s = -n.value();

                        p.value = std::cmp::max(v_b, s);
                    }
                }
            }
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.position.print_board();
        let mut s = format!("value: {}", self.value);
        s += format!("\nchoice: {}", {
            if let Some(m) = self.choice {
                m.uci()
            } else {
                "None".to_string()
            }
        })
        .as_str();
        s += format!("\nresult: {}", {
            if let Some(r) = self.result {
                format!("{}", r)
            } else {
                "Unknown".to_string()
            }
        })
        .as_str();

        write!(f, "{}\n", s)
    }
}

#[allow(unused)]
fn is_terminal(eval: i32) -> bool {
    eval.abs() > INFINITY / 2
}

fn game_result(found_move: bool, board: &Board, history: &[u64]) -> Option<f32> {
    if !found_move {
        match board.side_to_move {
            Colour::White => return Some(if board.checkers != 0 { 0.0 } else { 0.5 }),
            Colour::Black => return Some(if board.checkers != 0 { 1.0 } else { 0.5 }),
        }
    }

    if board.fifty_move == 100 || history.iter().filter(|x| **x == board.hash_key).count() == 2 {
        Some(0.5)
    } else {
        None
    }
}

pub fn play_one_game() -> Vec<(String, i32, f32)> {
    // **Very** occasionally the engine can fail to find a move in 10ms / within node limit which leads
    // it to not find a move to play. In this case we just throw away the game and try again until one works.
    // To make sure that there isn't some bigger problem if we somehow fail to generate 3 games in
    // a row, then panic!()

    let mut attempts = 0;

    let mut try_game = None;
    while try_game.is_none() {
        if attempts >= 3 {
            panic!("failing to find moves too often...");
        }
        try_game = Game::generate();
        attempts += 1;
    }

    let g = try_game.unwrap();

    let mut filtered = vec![];

    for n in g
        .positions
        .iter()
        .take(g.positions.len() - 1)
        .skip(OPENING_PLIES)
    {
        let quiet = n.position.checkers == 0 && !n.choice.unwrap().is_capture(&n.position);
        let within_bounds = n.value.abs() < i16::max_value() as i32;
        let enough_pieces = n.position.occupancies[BOTH].count_ones() > 3;

        let value = match n.position.side_to_move {
            Colour::White => n.value,
            Colour::Black => -n.value,
        };

        if quiet && within_bounds && enough_pieces {
            filtered.push((n.position.fen(), value, n.result.unwrap()))
        }
    }

    filtered
}

pub fn play_parallel_games(num_games: usize, num_threads: usize) -> Vec<(String, i32, f32)> {
    let num_threads = std::cmp::min(num_threads, num_games);

    let games_per_thread = num_games / num_threads;
    let remainder = num_games % num_threads;

    let mut handles = vec![];

    for i in 0..num_threads {
        let thread_games = games_per_thread + if i < remainder { 1 } else { 0 };

        let handle = thread::spawn(move || {
            let mut results = Vec::new();

            for _ in 0..thread_games {
                match std::panic::catch_unwind(play_one_game) {
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
    let mut file = if let Ok(f) = std::fs::OpenOptions::new().append(true).open(path) {
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

    let pb = ProgressBar::new(duration.as_secs());
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
        let results = play_parallel_games(BATCH_SIZE, thread_count);

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
