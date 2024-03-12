use crate::board::*;
use crate::eval::*;
use crate::movegen::*;
use crate::r#move::*;

use std::cmp;

pub const INFINITY: i32 = 999_999_999;

fn negamax(position: Board, depth: usize, alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return quiescence_search(position, alpha, beta);
    }

    if position.fifty_move == 100 {
        //this is here instead of eval to save some nodes in cases
        //where 50 move rule is reached on non-leaf node
        return 0;
    }

    let child_nodes = gen_legal(position);

    if child_nodes.moves[0] == NULL_MOVE {
        return is_checkmate(position);
    }

    let mut alpha = alpha;

    for i in 0..MAX_MOVES {
        if child_nodes.moves[i] == NULL_MOVE {
            break;
        }
        let eval = -negamax(
            make_move(child_nodes.moves[i], position),
            depth - 1,
            -beta,
            -alpha,
        );
        if beta <= eval {
            break;
        }
        alpha = cmp::max(alpha, eval);
    }
    alpha
}

fn quiescence_search(position: Board, alpha: i32, beta: i32) -> i32 {
    let eval = evaluate(&position);
    if eval >= beta {
        return beta;
    }

    let mut alpha = cmp::max(alpha, eval);

    let moves = gen_captures(position);
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        let eval = -quiescence_search(make_move(moves.moves[i], position), -beta, -alpha);
        if eval >= beta {
            return beta;
        }
        alpha = cmp::max(alpha, eval);
    }
    alpha
}

pub fn best_move(position: Board) -> Move {
    let moves = gen_legal(position);
    let mut best_eval = -INFINITY;
    let mut best_move = NULL_MOVE;
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        let eval = -negamax(make_move(moves.moves[i], position), 4, -INFINITY, INFINITY);
        if eval > best_eval {
            best_eval = eval;
            best_move = moves.moves[i];
        }
    }
    best_move
}
