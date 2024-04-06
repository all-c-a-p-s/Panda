use crate::board::*;
use crate::eval::*;
use crate::helper::*;
use crate::movegen::*;
use crate::r#move::*;
use crate::transposition::*;

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

const RAZORING_MARGIN: i32 = 300;
const MAX_RAZOR_DEPTH: usize = 4;

const BETA_PRUNING_DEPTH: usize = 6;
const BETA_PRUNING_MARGIN: usize = 80;

const ALPHA_PRUNING_DEPTH: usize = 4;
const ALPHA_PRUNING_MARGIN: i32 = 2000;

const HASH_MOVE_SCORE: i32 = 20_000;
const PV_MOVE_SCORE: i32 = 10_000;
const FIRST_KILLER_MOVE: i32 = 90;
const SECOND_KILLER_MOVE: i32 = 80;

pub const MAX_GAME_PLY: usize = 1024;

const TIME_TO_MOVE: usize = 100;
const TIME_TO_START_SEARCH: usize = 0; //initialise big TT (if not using HashMap)
                                       //leave a second total margin

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

fn reduction_ok(tactical: bool, is_check: bool) -> bool {
    !(tactical || is_check)
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

fn is_insufficient_material(b: &Board) -> bool {
    if count(
        b.bitboards[WP]
            | b.bitboards[WR]
            | b.bitboards[WQ]
            | b.bitboards[BP]
            | b.bitboards[BR]
            | b.bitboards[BQ],
    ) != 0
    {
        return false;
    }
    if count(b.bitboards[WB]) >= 2 || count(b.bitboards[BB]) >= 2 {
        return false;
    }
    count(b.bitboards[WN]) <= 2 && count(b.bitboards[BN]) <= 2
    //can technically arise a position where KvKNN is mate so this
    //could cause some bug in theory lol
}

unsafe fn is_drawn(position: &Board) -> bool {
    if position.fifty_move == 100 {
        return true;
    }
    unsafe {
        for key in REPETITION_TABLE.iter().take(position.ply) {
            if *key == position.hash_key {
                return true;
                //return true on one repetition because otherwise the third
                //repetition will not be reached because the search will stop
                //after a tt hit on the second repetition
            }
        }
    }
    is_insufficient_material(position)
}

impl Searcher {
    pub fn new(end_time: Instant) -> Self {
        Searcher {
            killer_moves: [[NULL_MOVE; MAX_PLY]; 2],
            history_scores: [[0; 64]; 12],
            pv_length: [0usize; 64],
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            tt_white: HashMap::new(), //128 MB
            tt_black: HashMap::new(), //seems to work pretty well (avoids t1 errors)
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
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        /* return a score that can never be >= alpha if the search is cancelled
        other engines return zero here but I don't see how this would work
        in cases where 0 > alpha. Returning value that gets recursively passed
        so that it is equal to -INFINITY for the engine should work imo because
        if this happens on PV move it breaks during the main moves loop below
        and so count of moves searched fully is zero -> discard result */
        if Instant::now() > self.end_time && self.ply != 0 {
            return match self.ply % 2 {
                0 => -INFINITY,
                1 => INFINITY,
                _ => unreachable!(),
            };
        }

        let is_pv = beta - alpha != 1;
        //full window search

        self.pv_length[self.ply] = self.ply;
        let mut hash_flag = EntryFlag::Alpha;

        if self.ply != 0 {
            //check 50 move rule, repetition and insufficient material
            unsafe {
                if is_drawn(position) {
                    return 0;
                }
            }

            //mate distance pruning:
            //check if line is so good/bad that being mated in the current ply
            //or mating in the next ply would not change alpha/beta
            let r_alpha = cmp::max(alpha, -INFINITY + self.ply as i32);
            let r_beta = cmp::min(beta, INFINITY - self.ply as i32 - 1);
            if r_alpha >= r_beta {
                return r_alpha;
            }
        }

        let mut best_move = NULL_MOVE; //used for TT hash -> move ordering
                                       //this is useful in cases where it cannot return the eval of the hash lookup
                                       //due to the bounds, but it can use the best_move field for move ordering
        if self.ply != 0 {
            let hash_lookup = match position.side_to_move {
                //hash lookup
                Colour::White => self.tt_white.lookup(position.hash_key, alpha, beta, depth),
                Colour::Black => self.tt_black.lookup(position.hash_key, alpha, beta, depth),
            };

            if let Some(k) = hash_lookup.eval {
                return k;
            } else if !hash_lookup.best_move.is_null() {
                best_move = hash_lookup.best_move;
            };
        }

        if depth == 0 {
            //qsearch on leaf nodes
            return self.quiescence_search(position, alpha, beta);
        }

        let is_check = match position.side_to_move {
            //used in both search extensions and LMR
            Colour::White => is_attacked(lsfb(position.bitboards[WK]), Colour::Black, position),
            Colour::Black => is_attacked(lsfb(position.bitboards[BK]), Colour::White, position),
        };

        let static_eval = evaluate(position);
        if !is_check && !is_pv {
            //eval is very low so only realistic way to increase it is with captures
            //we only need to qsearch to evaluate the position
            if depth <= MAX_RAZOR_DEPTH && static_eval + RAZORING_MARGIN * (depth as i32) <= alpha {
                let score = self.quiescence_search(position, alpha, beta);
                if score > alpha {
                    return score;
                }
            }

            //Beta Pruning / Reverse Futility Pruning:
            //If eval >= beta + some margin, assume that we can achieve at least beta
            if depth <= BETA_PRUNING_DEPTH
                && static_eval - (BETA_PRUNING_MARGIN * depth) as i32 >= beta
            {
                return static_eval;
            }

            //eval is so bad that even a huge margin fails to raise alpha
            if depth <= ALPHA_PRUNING_DEPTH && static_eval + ALPHA_PRUNING_MARGIN <= alpha {
                return static_eval;
            }
        }

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

        //Internal Iterative Deepening:
        //pv node and no tt hit -> move ordering will be terrible
        //so do a shallower search to rectify move ordering
        //by fixing history tables and pv move
        //according to wiki this should make little difference on average
        //but should make the search more consistent
        if is_pv && depth > 3 && best_move.is_null() {
            self.negamax(position, depth - 2, alpha, beta);
        }

        let mut child_nodes = MoveList::gen_legal(position);
        child_nodes.order_moves(position, self, &best_move);

        if child_nodes.moves[0].is_null() {
            //no legal moves -> mate or stalemate
            return match is_check {
                true => -INFINITY + self.ply as i32,
                false => 0,
            };
        }

        for i in 0..MAX_MOVES {
            if child_nodes.moves[i].is_null() {
                //no moves/no time left
                break;
            }

            let tactical = child_nodes.moves[i].is_tactical(position);
            //must be done before making the move on the board

            unsafe { REPETITION_TABLE[position.ply] = position.hash_key };

            let commit = position.make_move(child_nodes.moves[i]);
            self.ply += 1;

            let eval = match i == 0 {
                true => -self.negamax(position, depth - 1, -beta, -alpha),
                //normal search on pv move (no moves searched yet)
                false => {
                    /* non-pv move -> search with reduced window
                    this assumes that our move ordering is good enough
                    that we will be able to prove that these moves are bad
                    often enough that it outweighs the cost of re-searching
                    then if we are unable to prove so */
                    let mut reduction_eval = match i >= FULL_DEPTH_MOVES
                        && depth >= REDUCTION_LIMIT
                        && reduction_ok(tactical, is_check)
                    {
                        true => -self.negamax(position, depth - 2, -alpha - 1, -alpha),
                        false => alpha + 1, //hack to make sure always > alpha so always searched properly
                    };
                    if reduction_eval > alpha {
                        //failed to prove that move is bad -> re-search with same depth but reduced
                        //window
                        reduction_eval = -self.negamax(position, depth - 1, -alpha - 1, -alpha);
                    }

                    if reduction_eval > alpha && reduction_eval < beta {
                        //move actually inside PV window -> search at full depth
                        reduction_eval = -self.negamax(position, depth - 1, -beta, -alpha);
                    }
                    reduction_eval
                }
            };

            position.undo_move(child_nodes.moves[i], commit);
            self.ply -= 1;

            unsafe { REPETITION_TABLE[position.ply] = 0u64 };

            if Instant::now() > self.end_time && self.ply == 0 {
                break;
                //as above
            }

            if eval >= beta {
                //only write quiet moves into history table because captures
                //will be scored separately
                if !tactical {
                    self.killer_moves[1][self.ply] = self.killer_moves[0][self.ply];
                    self.killer_moves[0][self.ply] = child_nodes.moves[i];
                }

                //write to hash table
                let hash_entry = TTEntry::new(depth, beta, EntryFlag::Beta, best_move);
                match position.side_to_move {
                    Colour::White => self.tt_white.write(position.hash_key, hash_entry),
                    Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
                };

                return beta;
            }
            if eval > alpha {
                if !tactical {
                    self.history_scores[child_nodes.moves[i].piece_moved(position)]
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
                best_move = child_nodes.moves[i];
            }

            if self.ply == 0 {
                self.moves_searched += 1;
                //used to ensure in the iterative deepening search that
                //at least one move has been searched fully
            }
        }

        let hash_entry = TTEntry::new(depth, alpha, hash_flag, best_move);
        match position.side_to_move {
            Colour::White => self.tt_white.write(position.hash_key, hash_entry),
            Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
        };
        alpha
    }

    pub fn quiescence_search(&mut self, position: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        let mut hash_flag = EntryFlag::Alpha;

        let hash_lookup = match position.side_to_move {
            //hash lookup
            Colour::White => self.tt_white.lookup(position.hash_key, alpha, beta, 0),
            Colour::Black => self.tt_black.lookup(position.hash_key, alpha, beta, 0),
            //lookups with depth zero because any TT entry will necessarily
            //have had a quiescence search done so we will always take it
        };

        let mut best_move = NULL_MOVE;
        if let Some(k) = hash_lookup.eval {
            return k;
        } else if !hash_lookup.best_move.is_null() {
            best_move = hash_lookup.best_move;
        };

        let eval = evaluate(position);
        self.nodes += 1;
        //node count = every position that gets evaluated
        if eval >= beta {
            return beta;
        }

        //don't need repetition detection as it's impossible to have repetition with captures
        let delta = 1000; //delta pruning - try to avoid wasting time on hopeless positions
        if eval < alpha - delta {
            return alpha;
        }

        alpha = cmp::max(alpha, eval);

        let mut captures = MoveList::gen_captures(position);

        if captures.moves[0].is_null() {
            //no captures
            return alpha;
        }

        captures.order_moves(position, self, &best_move);
        for i in 0..MAX_MOVES {
            if captures.moves[i].is_null() {
                break;
            }
            let commit = position.make_move(captures.moves[i]);
            self.ply += 1;
            let eval = -self.quiescence_search(position, -beta, -alpha);
            position.undo_move(captures.moves[i], commit);
            self.ply -= 1;
            if eval >= beta {
                //hash write in case of beta cutoff
                let hash_entry = TTEntry::new(0, beta, EntryFlag::Beta, best_move);
                match position.side_to_move {
                    Colour::White => self.tt_white.write(position.hash_key, hash_entry),
                    Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
                };
                return beta;
            }
            if eval > alpha {
                alpha = eval;
                hash_flag = EntryFlag::Exact;
                best_move = captures.moves[i];
            }
            alpha = cmp::max(alpha, eval);
        }

        let hash_entry = TTEntry::new(0, alpha, hash_flag, best_move);
        match position.side_to_move {
            Colour::White => self.tt_white.write(position.hash_key, hash_entry),
            Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
        };
        alpha
    }

    pub fn reset_searcher(&mut self) {
        //try to keep tt, history and killer moves
        self.nodes = 0;
        self.pv_length = [0; 64];
        self.pv = [[NULL_MOVE; MAX_PLY]; MAX_PLY];
        self.ply = 0;
        self.moves_searched = 0;

        //not clearing seems to be worse as even though the first few search depths are instant
        //the next depths don't have advantages of iterative deepening like pv and move ordering
        //heuristics
        self.tt_white = HashMap::new();
        self.tt_black = HashMap::new();

        //test shifting killer moves by 2
        self.killer_moves = [[NULL_MOVE; MAX_PLY]; 2];
        /*
                for i in 2..MAX_PLY {
                    self.killer_moves[0][i - 2] = self.killer_moves[0][i];
                    self.killer_moves[1][i - 2] = self.killer_moves[1][i];
                }

                self.killer_moves[0][MAX_PLY - 2] = NULL_MOVE;
                self.killer_moves[1][MAX_PLY - 2] = NULL_MOVE;
                self.killer_moves[0][MAX_PLY - 1] = NULL_MOVE;
                self.killer_moves[1][MAX_PLY - 1] = NULL_MOVE;
        */
        self.history_scores = [[0; 64]; 12];
    }
}

const MVV_LVA: [[i32; 6]; 6] = [
    //most valuable victim least valuable attacker
    [205, 204, 203, 202, 201, 200], //victim pawn
    [305, 304, 303, 302, 301, 300], //victim knight
    [405, 404, 403, 402, 401, 400], //victim bishop
    [505, 504, 503, 502, 501, 500], //victim rook
    [605, 604, 603, 602, 601, 600], //victim queen
    [0, 0, 0, 0, 0, 0],             //victim king
];

impl Move {
    pub fn score_move(self, b: &Board, s: &Searcher, hash_move: &Move) -> i32 {
        if self.is_null() {
            -INFINITY
            //important for this to come before checking hash move
            //otherwise null move can get given hash move score
        } else if self == *hash_move {
            HASH_MOVE_SCORE
            //before pv move because this has been verified by >= search depth
        } else if self == s.pv[0][s.ply] {
            PV_MOVE_SCORE
        } else if self.is_capture(b) {
            let victim_type: usize = piece_type(b.pieces_array[self.square_to()]);
            let attacker_type = piece_type(self.piece_moved(b));
            if self.is_promotion() {
                return match self.promoted_piece() {
                    //promotions sorted by likelihood to be good
                    QUEEN => 1000,
                    KNIGHT => 900,
                    ROOK => 800,
                    BISHOP => 700,
                    _ => unreachable!(),
                } + MVV_LVA[victim_type][attacker_type];
            }
            MVV_LVA[victim_type][attacker_type]
        } else if self.is_promotion() {
            match self.promoted_piece() {
                //promotions sorted by likelihood to be good
                QUEEN => 1000,
                KNIGHT => 900,
                ROOK => 800,
                BISHOP => 700,
                _ => unreachable!(),
            }
        } else if self.is_en_passant() {
            MVV_LVA[PAWN][PAWN]
        } else if s.killer_moves[0][s.ply] == self {
            FIRST_KILLER_MOVE //after captures
        } else if s.killer_moves[1][s.ply] == self {
            SECOND_KILLER_MOVE
        } else {
            s.history_scores[self.piece_moved(b)][self.square_to()]
        }
    }
}

#[derive(Copy, Clone)]
pub struct MoveOrderEntry<'a> {
    m: &'a Move,
    score: i32,
}

impl MoveList {
    pub fn order_moves(&mut self, board: &Board, s: &Searcher, best_move: &Move) {
        let mut ordered_moves = [MoveOrderEntry {
            m: &NULL_MOVE,
            score: 0,
        }; MAX_MOVES];

        #[allow(clippy::needless_range_loop)]
        for i in 0..self.moves.len() {
            if self.moves[i].is_null() {
                break;
            }
            ordered_moves[i].m = &self.moves[i];
            ordered_moves[i].score = self.moves[i].score_move(board, s, best_move);
        }

        ordered_moves.sort_by(|a, b| b.score.cmp(&a.score));

        let mut final_moves = [NULL_MOVE; MAX_MOVES];

        for i in 0..MAX_MOVES {
            if ordered_moves[i].m.is_null() {
                break;
            }
            final_moves[i] = *ordered_moves[i].m;
        }
        self.moves = final_moves
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
        + TIME_TO_START_SEARCH
}

pub fn best_move(
    position: &mut Board,
    time_left: usize,
    inc: usize,
    moves_to_go: usize,
    s: &mut Searcher,
) -> MoveData {
    //TODO: test using one searcher across whole game (not clearing TT)
    let start = Instant::now();
    let move_duration = Duration::from_millis(
        move_time(time_left, inc, moves_to_go, position.ply)
            .try_into()
            .unwrap(),
    );

    let end_time = start + move_duration;
    //calculate time to cancel search

    s.reset_searcher();
    s.end_time = end_time;

    let mut eval: i32 = 0;
    let mut previous_eval = eval; //used for cases shere search cancelled after searching zero moves fully
    let mut pv = String::new();

    let mut previous_pv = s.pv;
    let mut previous_pv_length = s.pv_length;

    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let mut depth = 1;

    let rt_table_reset = unsafe { REPETITION_TABLE };

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

        if start.elapsed() * 2 > move_duration {
            /*more than half of time used -> no point starting another search as its likely
             * that zero moves will be searched fully.
             * ofc this also catches situations where all of move duration has elapsed */
            break;
        }

        s.moves_searched = 0;
        unsafe { REPETITION_TABLE = rt_table_reset };

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
