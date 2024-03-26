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

pub const INFINITY: i32 = 999_999_999;
pub const MAX_PLY: usize = 64;
pub const MAX_SEARCH_DEPTH: usize = 64;

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
}

impl Searcher {
    fn new(end_time: Instant) -> Self {
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
        }
    }

    fn negamax(&mut self, position: &mut Board, depth: usize, alpha: i32, beta: i32) -> i32 {
        self.pv_length[self.ply] = self.ply;
        let mut hash_flag = EntryFlag::Alpha;

        if depth == 0 {
            return self.quiescence_search(position, alpha, beta);
        }

        if position.fifty_move == 100 {
            //this is here instead of eval to save some nodes in cases
            //where 50 move rule is reached on non-leaf node
            return 0;
        }

        let mut alpha = alpha;

        match position.side_to_move {
            Colour::White => {
                if let Some(entry) = self.tt_white.get(&hash(position)) {
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
                }
            }
            Colour::Black => {
                if let Some(entry) = self.tt_black.get(&hash(position)) {
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

        let mut child_nodes = gen_legal(position);
        child_nodes.order_moves(*position, self);

        if child_nodes.moves[0] == NULL_MOVE {
            return is_checkmate(*position);
        }

        for i in 0..MAX_MOVES {
            if child_nodes.moves[i] == NULL_MOVE {
                break;
            }
            let commit = position.make_move(child_nodes.moves[i]);
            self.ply += 1;
            let eval = -self.negamax(position, depth - 1, -beta, -alpha);
            position.undo_move(child_nodes.moves[i], commit);
            self.ply -= 1;

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
            }
        }
        let hash_entry = TTEntry {
            flag: hash_flag,
            depth,
            eval: alpha,
        };
        match position.side_to_move {
            Colour::White => self.tt_white.insert(hash(position), hash_entry),
            Colour::Black => self.tt_black.insert(hash(position), hash_entry),
        };
        alpha
    }

    fn quiescence_search(&mut self, position: &mut Board, alpha: i32, beta: i32) -> i32 {
        let eval = evaluate(position);
        if eval >= beta {
            self.nodes += 1;
            return beta;
        }

        let delta = 1000; //delta pruning - try to avoid wasting time on hopeless positions
        if eval < alpha - delta {
            self.nodes += 1;
            return alpha;
        }

        let mut alpha = cmp::max(alpha, eval);

        let mut moves = gen_captures(position);

        if moves.moves[0] == NULL_MOVE {
            self.nodes += 1; //quiet node
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

pub fn move_time(time: usize, increment: usize, moves_to_go: usize, ply: usize) -> usize {
    let time_left = time + 20 * increment;
    let n_moves = cmp::min(moves_to_go, 10);
    let factor = 2.0 - n_moves as f32 / 10.0;
    let target: f32 = match ply {
        0..=10 => time_left as f32 / 50.0,
        11..=16 => time_left as f32 / 30.0,
        _ => time_left as f32 / 20.0,
    };
    (factor * target) as usize
}

pub fn best_move(position: &mut Board, time_left: usize, inc: usize, moves_to_go: usize) -> MoveData {
    let start = Instant::now();
    let move_duration = Duration::from_millis(move_time(time_left, inc, moves_to_go, position.ply).try_into().unwrap());
    let end_time = start + move_duration;
    let mut eval: i32 = 0;
    let mut pv = String::new();
    let mut s = Searcher::new(end_time);
    for depth in 1..MAX_SEARCH_DEPTH {
        eval = s.negamax(position, depth, -INFINITY, INFINITY);
        println!("info depth {} score cp {} nodes {} pv {}", depth, eval, s.nodes, {
            let mut pv = String::new();
            for i in 0..s.pv_length[0] {
                pv += coordinate(s.pv[0][i].square_from()).as_str();
                pv += coordinate(s.pv[0][i].square_to()).as_str();
                pv += " ";
            }
            pv
            }            
        );
        if start.elapsed() > move_duration || start.elapsed() * 2 > move_duration {
            //more than half of time used -> no point starting another search as it won't be completed
            break;
        }
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
