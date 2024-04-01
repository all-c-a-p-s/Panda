use crate::board::*;
use crate::eval::*;
use crate::get_bit;
use crate::helper::*;
use crate::movegen::*;
use crate::r#move::*;
use crate::zobrist::*;

use std::cmp;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const INFINITY: i32 = 1_000_000_000;
pub const MAX_PLY: usize = 64;
pub const MAX_SEARCH_DEPTH: usize = 32;
pub const REDUCTION_LIMIT: usize = 3;
// can't reduce search to below 3 - 2 = 1 ply
const FULL_DEPTH_MOVES: usize = 4;
const NULLMOVE_R: usize = 2;
const ASPIRATION_WINDOW: i32 = 50;

pub const MAX_GAME_PLY: usize = 1024;

const TIME_TO_MOVE: usize = 1000;
//leave a second to cancel search and output best move

pub static mut REPETITION_TABLE: [u64; MAX_GAME_PLY] = [0u64; MAX_GAME_PLY];

pub struct Searcher {
    pub killer_moves: [[Move; MAX_PLY]; 2],
    pub history_scores: [[i32; 64]; 12],
    pub pv_length: [usize; 64],
    pub pv: [[Move; MAX_PLY]; MAX_PLY],
    pub tt_white: HashMap<u64, TTEntry>,
    pub tt_black: HashMap<u64, TTEntry>,
    pub ply: usize,
    pub nodes: usize,
    pub end_time: Instant,
    pub moves_searched: usize,
}

fn reduction_ok(m: Move, is_check: bool) -> bool {
    if m.is_capture() || m.promoted_piece() != 15 {
        return false;
    }
    !is_check //false if board is check
}

fn make_null_move(b: &mut Board) -> usize {
    //returns en passant reset
    b.side_to_move = match b.side_to_move {
        Colour::White => Colour::Black,
        Colour::Black => Colour::White,
    };
    b.ply += 1;
    b.last_move_null = true;
    if b.en_passant != NO_SQUARE {
        let reset = b.en_passant;
        b.en_passant = NO_SQUARE;
        return reset;
    }
    NO_SQUARE
}

fn undo_null_move(b: &mut Board, ep_reset: usize) {
    b.side_to_move = match b.side_to_move {
        Colour::White => Colour::Black,
        Colour::Black => Colour::White,
    };
    b.ply -= 1;
    b.last_move_null = false;
    b.en_passant = ep_reset;
}

impl Searcher {
    pub fn new(end_time: Instant) -> Self {
        Searcher {
            killer_moves: [[NULL_MOVE; MAX_PLY]; 2],
            history_scores: [[0; 64]; 12],
            pv_length: [0usize; 64],
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            tt_white: HashMap::new(),
            tt_black: HashMap::new(),
            ply: 0,
            nodes: 0,
            end_time,
            moves_searched: 0,
        }
    }

    pub fn negamax(
        &mut self,
        position: &mut Board,
        mut depth: usize,
        alpha: i32,
        beta: i32,
    ) -> i32 {
        if Instant::now() > self.end_time {
            return match self.ply % 2 {
                0 => -INFINITY,
                1 => INFINITY,
                _ => panic!("maths is broken lol"),
            };
        }

        self.pv_length[self.ply] = self.ply;
        let mut hash_flag = EntryFlag::Alpha;

        if depth == 0 {
            return self.quiescence_search(position, alpha, beta);
        }

        if position.fifty_move == 100 {
            //this is here instead of eval to save some nodes in cases
            //where 50 move rule is reached on non-leaf node
            //in theory if the fiftieth move is mate this doesn't work
            return 0;
        }

        let hash_key = hash(position);
        let mut repetition_count = 0;
        unsafe {
            for key in REPETITION_TABLE.iter().take(position.ply) {
                if *key == hash_key {
                    repetition_count += 1;
                    //return 0 even for single repetition
                }
            }
        }

        if repetition_count > 1 {
            return 0;
        }

        let mut alpha = alpha;

        match position.side_to_move {
            //hash lookup
            Colour::White => {
                if let Some(entry) = self.tt_white.get(&hash_key) {
                    if entry.depth >= depth {
                        match entry.flag {
                            EntryFlag::Beta => {
                                //lower bound hash entry
                                if entry.eval >= beta {
                                    return beta;
                                }
                            }
                            EntryFlag::Alpha => {
                                //upper bound entry
                                if entry.eval <= alpha {
                                    return alpha;
                                }
                            }
                            EntryFlag::Exact => return entry.eval,
                            //pv entry
                        }
                    }
                }
            }
            Colour::Black => {
                if let Some(entry) = self.tt_black.get(&hash_key) {
                    if entry.depth >= depth {
                        match entry.flag {
                            EntryFlag::Beta => {
                                if entry.eval >= beta {
                                    return beta;
                                }
                            }
                            EntryFlag::Alpha => {
                                if entry.eval <= alpha {
                                    return alpha;
                                }
                            }
                            EntryFlag::Exact => return entry.eval,
                        }
                    }
                };
            }
        }

        let is_check = match position.side_to_move {
            //used in both search extensions and LMR
            Colour::White => is_attacked(lsfb(position.bitboards[WK]), Colour::Black, position),
            Colour::Black => is_attacked(lsfb(position.bitboards[BK]), Colour::White, position),
        };

        if is_check && self.ply != 0 {
            depth += 1;
            //if this occurs at ply zero then next search iteration will basically be skipped
            //because of hash lookup
        }

        if !position.is_kp_endgame()
            && !position.last_move_null
            && depth > NULLMOVE_R
            && !is_check
            && self.ply > 0
        {
            //ok to null-move prune
            let ep_reset = make_null_move(position);
            self.ply += 1;
            //idea that if opponent cannot improve their position with 2 moves in a row
            //the first of these moves must be bad
            let null_move_eval = -self.negamax(position, depth - NULLMOVE_R, -beta, -beta + 1);
            //minimal window used because all that matters is whether the search result is better than beta
            undo_null_move(position, ep_reset);
            self.ply -= 1;
            if null_move_eval >= beta {
                return beta;
            }
        }

        let mut child_nodes = gen_legal(position);
        child_nodes.order_moves(*position, self);

        if child_nodes.moves[0] == NULL_MOVE {
            //no legal moves
            return match is_check {
                true => -INFINITY + self.ply as i32,
                false => 0,
            };
        }

        for i in 0..MAX_MOVES {
            if child_nodes.moves[i] == NULL_MOVE {
                break;
            }
            let commit = position.make_move(child_nodes.moves[i]);
            self.ply += 1;

            unsafe { REPETITION_TABLE[position.ply] = hash(position) };

            let eval = match i == 0 {
                true => -self.negamax(position, depth - 1, -beta, -alpha),
                //normal search on pv move
                false => {
                    let mut reduction_eval = match i >= FULL_DEPTH_MOVES
                        && depth >= REDUCTION_LIMIT
                        && reduction_ok(child_nodes.moves[i], is_check)
                    {
                        true => -self.negamax(position, depth - 2, -alpha - 1, -alpha),
                        false => alpha + 1, //hack to make sure always > alpha so always searched properly
                    };
                    if reduction_eval > alpha {
                        reduction_eval = -self.negamax(position, depth - 1, -alpha - 1, -alpha);
                    }

                    if reduction_eval > alpha && reduction_eval < beta {
                        reduction_eval = -self.negamax(position, depth - 1, -beta, -alpha);
                    }
                    reduction_eval
                }
            };

            position.undo_move(child_nodes.moves[i], commit);
            self.ply -= 1;

            unsafe { REPETITION_TABLE[position.ply] = 0u64 };

            if eval >= beta {
                if !child_nodes.moves[i].is_capture() {
                    self.killer_moves[1][self.ply] = self.killer_moves[0][self.ply];
                    self.killer_moves[0][self.ply] = child_nodes.moves[i];
                }

                let hash_entry = TTEntry {
                    flag: EntryFlag::Beta,
                    depth,
                    eval: beta,
                };
                match position.side_to_move {
                    Colour::White => self.tt_white.insert(hash(position), hash_entry),
                    Colour::Black => self.tt_black.insert(hash(position), hash_entry),
                };

                return beta;
            }
            if eval > alpha {
                if !child_nodes.moves[i].is_capture() {
                    self.history_scores[child_nodes.moves[i].piece_moved()]
                        [child_nodes.moves[i].square_to()] += depth as i32 * depth as i32;
                    //idea that moves closer to root node are more significant
                }
                let next_ply = self.ply + 1;
                self.pv[self.ply][self.ply] = child_nodes.moves[i];
                for j in next_ply..self.pv_length[next_ply] {
                    self.pv[self.ply][j] = self.pv[next_ply][j];
                    //copy from next row in pv table
                }
                self.pv_length[self.ply] = self.pv_length[next_ply];
                alpha = eval;
                hash_flag = EntryFlag::Exact;
            }

            if Instant::now() > self.end_time && self.ply == 0 {
                //check if time is up every time at root node
                //this way pv shouldn't get messed up, although time usage might be imperfect
                break;
            } else if self.ply == 0 {
                self.moves_searched += 1;
            }
        }
        let hash_entry = TTEntry {
            flag: hash_flag,
            depth,
            eval: alpha,
        };
        match position.side_to_move {
            Colour::White => self.tt_white.insert(hash_key, hash_entry),
            Colour::Black => self.tt_black.insert(hash_key, hash_entry),
        };
        alpha
    }

    pub fn quiescence_search(&mut self, position: &mut Board, alpha: i32, beta: i32) -> i32 {
        let eval = evaluate(position);
        self.nodes += 1;
        if eval >= beta {
            return beta;
        }

        //don't need repetition detection as it's impossible to have repetition with captures

        let delta = 1000; //delta pruning - try to avoid wasting time on hopeless positions
        if eval < alpha - delta {
            return alpha;
        }

        let mut alpha = cmp::max(alpha, eval);

        let mut moves = gen_captures(position);

        if moves.moves[0] == NULL_MOVE {
            return alpha;
        }

        moves.order_moves(*position, self);
        for i in 0..MAX_MOVES {
            if moves.moves[i] == NULL_MOVE {
                break;
            }
            let commit = position.make_move(moves.moves[i]);
            self.ply += 1;
            let eval = -self.quiescence_search(position, -beta, -alpha);
            position.undo_move(moves.moves[i], commit);
            self.ply -= 1;
            if eval >= beta {
                return beta;
            }
            alpha = cmp::max(alpha, eval);
        }
        alpha
    }
}

const MVV_LVA: [[i32; 6]; 6] = [
    //most valuable victim least valuable attacker
    [601, 501, 401, 301, 201, 101], //victim pawn
    [602, 502, 402, 302, 202, 102], //victim knight
    [603, 503, 403, 303, 203, 103], //victim bishop
    [604, 504, 404, 304, 204, 104], //victim rook
    [605, 505, 405, 305, 205, 105], //victim queen
    [0, 0, 0, 0, 0, 0],             //victim king
];

impl Move {
    pub fn score_move(&mut self, b: Board, s: &Searcher) {
        if (*self) == NULL_MOVE {
            self.move_order_score = -INFINITY;
            return;
        }

        let pv_move = s.pv[0][s.ply];
        if self.square_from() == pv_move.square_from() && self.square_to() == pv_move.square_to() {
            //there has to be a cleaner way to do this but dereferencing pointer doesn't work
            self.move_order_score = INFINITY; //pv move searched first
            return;
        }
        if self.is_capture() {
            let mut victim_type: usize = 7; //initialise as impossible value
            for i in 0..12 {
                if get_bit(self.square_to(), b.bitboards[i]) == 1 {
                    victim_type = i % 6;
                    break;
                }
            }
            let attacker_type = self.piece_moved() % 6;
            self.move_order_score = MVV_LVA[victim_type][attacker_type];
        } else if self.is_en_passant() {
            self.move_order_score = MVV_LVA[0][0];
        } else if s.killer_moves[0][s.ply] == *self {
            self.move_order_score = 90; //after captures
        } else if s.killer_moves[1][s.ply] == *self {
            self.move_order_score = 80;
        } else {
            self.move_order_score = s.history_scores[self.piece_moved()][self.square_to()];
        }
    }
}

impl MoveList {
    pub fn order_moves(&mut self, board: Board, s: &Searcher) {
        for i in 0..MAX_MOVES {
            if self.moves[i] == NULL_MOVE {
                break;
            }
            self.moves[i].score_move(board, s);
        }
        self.moves
            .sort_by(|a, b| b.move_order_score.cmp(&a.move_order_score));
    }
}

pub struct MoveData {
    pub m: Move,
    pub nodes: usize,
    pub eval: i32,
    pub pv: String,
}

#[allow(unused_variables)]
pub fn move_time(time: usize, increment: usize, moves_to_go: usize, ply: usize) -> usize {
    (match ply {
        0..=10 => (time - TIME_TO_MOVE) as f32 / 40.0,
        11..=16 => (time - TIME_TO_MOVE) as f32 / (moves_to_go as f32),
        _ => (time - TIME_TO_MOVE) as f32 / cmp::max((moves_to_go as f32 / 2.0) as usize, 1) as f32,
    }) as usize
}

pub fn best_move(
    position: &mut Board,
    time_left: usize,
    inc: usize,
    moves_to_go: usize,
) -> MoveData {
    let start = Instant::now();
    let move_duration = Duration::from_millis(
        move_time(time_left, inc, moves_to_go, position.ply)
            .try_into()
            .unwrap(),
    );
    let end_time = start + move_duration;
    //calculate time to cancel search

    let mut eval: i32 = 0;
    let mut previous_eval = eval; //used for cases shere search cancelled after searching zero moves fully
    let mut pv = String::new();
    let mut s = Searcher::new(end_time);
    let mut previous_pv = s.pv;
    let mut previous_pv_length = s.pv_length;

    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let mut depth = 1;

    while depth < MAX_SEARCH_DEPTH {
        eval = s.negamax(position, depth, alpha, beta);
        if s.moves_searched == 0 {
            //search cancelled before even pv was searched
            eval = previous_eval;
            s.pv = previous_pv;
            s.pv_length = previous_pv_length;
        } else {
            // >= 1 move searched ok
            previous_eval = eval;
            previous_pv = s.pv;
            previous_pv_length = s.pv_length;
        }

        println!(
            "info depth {} score cp {} nodes {} pv {}time {} nps {}",
            depth,
            eval,
            s.nodes,
            {
                let mut pv = String::new();
                for i in 0..s.pv_length[0] {
                    pv += coordinate(s.pv[0][i].square_from()).as_str();
                    pv += coordinate(s.pv[0][i].square_to()).as_str();
                    pv += " ";
                }
                pv
            },
            start.elapsed().as_millis(),
            {
                let micros = start.elapsed().as_micros() as f64;
                if micros == 0.0 {
                    0
                } else {
                    ((s.nodes as f64 / micros) * 1_000_000.0) as u64
                }
            }
        );
        if start.elapsed() > move_duration || start.elapsed() * 2 > move_duration {
            //more than half of time used -> no point starting another search as it won't be completed
            break;
        }

        s.moves_searched = 0;

        if eval <= alpha || eval >= beta {
            //fell outside window -> re-search with same depth
            alpha = -INFINITY;
            beta = INFINITY;
            continue; //continue without incrementing depth
        }

        //set up search for next iteration
        alpha = eval - ASPIRATION_WINDOW;
        beta = eval + ASPIRATION_WINDOW;
        depth += 1;
    }

    for i in 0..s.pv_length[0] {
        pv += coordinate(s.pv[0][i].square_from()).as_str();
        pv += coordinate(s.pv[0][i].square_to()).as_str();
        pv += " ";
    }

    MoveData {
        m: s.pv[0][0],
        nodes: s.nodes,
        eval,
        pv,
    }
}
