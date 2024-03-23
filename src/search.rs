use crate::board::*;
use crate::eval::*;
use crate::get_bit;
use crate::helper::*;
use crate::movegen::*;
use crate::r#move::*;

use std::cmp;

pub const INFINITY: i32 = 999_999_999;
pub const MAX_PLY: usize = 64;
pub const SEARCH_DEPTH: usize = 6;

pub struct Searcher {
    pub killer_moves: [[Move; MAX_PLY]; 2],
    pub history_scores: [[i32; 64]; 12],
    pub pv_length: [usize; 64],
    pub pv: [[Move; MAX_PLY]; MAX_PLY],
    pub ply: usize,
    pub nodes: usize,
}

impl Searcher {
    fn new() -> Self {
        Searcher {
            killer_moves: [[NULL_MOVE; MAX_PLY]; 2],
            history_scores: [[0; 64]; 12],
            pv_length: [0usize; 64],
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            ply: 0,
            nodes: 0,
        }
    }

    fn negamax(&mut self, position: &mut Board, depth: usize, alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return self.quiescence_search(position, alpha, beta);
        }
        self.pv_length[self.ply] = self.ply + depth;

        if position.fifty_move == 100 {
            //this is here instead of eval to save some nodes in cases
            //where 50 move rule is reached on non-leaf node
            return 0;
        }

        let mut child_nodes = gen_legal(position);
        child_nodes.order_moves(*position, self);

        if child_nodes.moves[0] == NULL_MOVE {
            return is_checkmate(*position);
        }

        let mut alpha = alpha;

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

                return beta;
            }
            if eval > alpha {
                if !child_nodes.moves[i].is_capture() {
                    self.history_scores[child_nodes.moves[i].piece_moved()]
                        [child_nodes.moves[i].square_to()] += depth as i32 * depth as i32;
                    //idea that moves closer to root node are more significant
                }
                self.pv[self.ply][self.ply] = child_nodes.moves[i];
                for i in self.ply + 1..self.pv_length[self.ply + 1] {
                    self.pv[self.ply][i] = self.pv[self.ply + 1][i];
                    //copy from next row in pv table
                }
                alpha = eval;
            }
        }
        alpha
    }

    fn quiescence_search(&mut self, position: &mut Board, alpha: i32, beta: i32) -> i32 {
        let eval = evaluate(position);
        if eval >= beta {
            return beta;
        }

        let delta = 1000; //delta pruning - try to avoid wasting time on hopeless positions
        if eval < alpha - delta {
            return alpha;
        }

        let mut alpha = cmp::max(alpha, eval);

        let mut moves = gen_captures(position);
        moves.order_moves(*position, self);
        if moves.moves[0] == NULL_MOVE {
            self.nodes += 1;
        }
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
        if s.pv[0][s.ply] == *self {
            self.move_order_score = INFINITY; //pv move searched first
        }
        if self.is_capture() {
            let mut victim_type: usize = 7; //initialise as impossible value
            for i in 0..12 {
                if get_bit(self.square_to(), b.bitboards[i]) == 1 {
                    victim_type = i % 6;
                    break;
                }
            }
            if victim_type == 7 {
                self.print_move();
            }
            let attacker_type = self.piece_moved() % 6;
            self.move_order_score = MVV_LVA[victim_type][attacker_type];
        } else if s.killer_moves[0][b.ply] == *self {
            self.move_order_score = 90; //after captures
        } else if s.killer_moves[1][b.ply] == *self {
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

pub fn best_move(position: &mut Board) -> MoveData {
    let mut eval: i32 = 0;
    let mut pv = String::new();
    let mut s = Searcher::new();
    for depth in 1..SEARCH_DEPTH + 1 {
        eval = s.negamax(position, depth, -INFINITY, INFINITY);
    }

    for i in 0..MAX_PLY {
        if s.pv[0][i] == NULL_MOVE {
            break;
        }
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
