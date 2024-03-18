use crate::board::*;
use crate::eval::*;
use crate::get_bit;
use crate::movegen::*;
use crate::r#move::*;

use std::cmp;

pub const INFINITY: i32 = 999_999_999;

pub static mut NODES: usize = 0;

fn negamax(position: &mut Board, depth: usize, alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return quiescence_search(position, alpha, beta);
    }

    if position.fifty_move == 100 {
        //this is here instead of eval to save some nodes in cases
        //where 50 move rule is reached on non-leaf node
        return 0;
    }

    let mut child_nodes = gen_legal(position);
    child_nodes.order_moves(*position);

    if child_nodes.moves[0] == NULL_MOVE {
        return is_checkmate(*position);
    }

    let mut alpha = alpha;

    for i in 0..MAX_MOVES {
        if child_nodes.moves[i] == NULL_MOVE {
            break;
        }
        let commit = position.make_move(child_nodes.moves[i]);
        let eval = -negamax(position, depth - 1, -beta, -alpha);
        position.undo_move(child_nodes.moves[i], commit);
        if eval >= beta {
            return beta;
        }
        alpha = cmp::max(alpha, eval);
    }
    alpha
}

fn quiescence_search(position: &mut Board, alpha: i32, beta: i32) -> i32 {
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
    moves.order_moves(*position);
    if moves.moves[0] == NULL_MOVE {
        unsafe { NODES += 1 };
    }
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        let commit = position.make_move(moves.moves[i]);
        let eval = -quiescence_search(position, -beta, -alpha);
        position.undo_move(moves.moves[i], commit);
        if eval >= beta {
            return beta;
        }
        alpha = cmp::max(alpha, eval);
    }
    alpha
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
    pub fn score_move(&mut self, b: Board) -> i32 {
        if (*self) == NULL_MOVE {
            return -INFINITY;
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
        }
        0
    }
}

impl MoveList {
    pub fn order_moves(&mut self, board: Board) {
        for i in 0..MAX_MOVES {
            if self.moves[i] == NULL_MOVE {
                break;
            }
            self.moves[i].score_move(board);
        }
        self.moves
            .sort_by(|a, b| b.move_order_score.cmp(&a.move_order_score));
    }
}

pub fn best_move(position: &mut Board) -> Move {
    let moves = gen_legal(position);
    let mut best_eval = -INFINITY;
    let mut best_move = NULL_MOVE;
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        let commit = position.make_move(moves.moves[i]);
        let eval = -negamax(position, 4, -INFINITY, INFINITY);
        position.undo_move(moves.moves[i], commit);
        if eval > best_eval {
            best_eval = eval;
            best_move = moves.moves[i];
        }
    }
    println!("eval: {}", best_eval);
    best_move
}
