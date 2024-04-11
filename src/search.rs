use crate::board::*;
use crate::eval::*;
use crate::get_bishop_attacks;
use crate::get_rook_attacks;
use crate::helper::*;
use crate::movegen::*;
use crate::r#move::*;
use crate::transposition::*;
use crate::STARTPOS;

use std::cmp;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const INFINITY: i32 = 1_000_000_000;
pub const MAX_PLY: usize = 64;
pub const MAX_SEARCH_DEPTH: usize = 32;
pub const REDUCTION_LIMIT: usize = 3;
// can't reduce search to below 3 - 2 = 1 ply
const FULL_DEPTH_MOVES: usize = 4;
const NULLMOVE_MIN_DEPTH: usize = 2;
const ASPIRATION_WINDOW: i32 = 40;

const RAZORING_MARGIN: i32 = 300;
const MAX_RAZOR_DEPTH: usize = 4;

const BETA_PRUNING_DEPTH: usize = 6;
const BETA_PRUNING_MARGIN: usize = 80;

const ALPHA_PRUNING_DEPTH: usize = 4;
const ALPHA_PRUNING_MARGIN: i32 = 2000;

const SEE_PRUNING_DEPTH: i32 = 4;
const SEE_QUIET_MARGIN: i32 = 100;
const SEE_NOISY_MARGIN: i32 = 70;
#[allow(unused)]
const SEE_QSEARCH_MARGIN: i32 = 130;
//TODO: test using static margin or just zero
//in SEE pruning in QSearch

#[allow(unused)]
const LMP_DEPTH: usize = 4;

const HASH_MOVE_SCORE: i32 = 1_000_000;
const PV_MOVE_SCORE: i32 = 500_000;
const QUEEN_PROMOTION: i32 = 400_000;
const WINNING_CAPTURE: i32 = 300_000;
const FIRST_KILLER_MOVE: i32 = 200_000;
const SECOND_KILLER_MOVE: i32 = 100_000;
const LOSING_CAPTURE: i32 = -100_000;
const UNDER_PROMOTION: i32 = -200_000;

pub const MAX_GAME_PLY: usize = 1024;

const TIME_TO_MOVE: usize = 100;
const TIME_TO_START_SEARCH: usize = 0; //initialise big TT (if not using HashMap)
                                       //leave a second total margin

pub static mut REPETITION_TABLE: [u64; MAX_GAME_PLY] = [0u64; MAX_GAME_PLY];
pub static mut START_DEPTH: usize = 0;

struct SearchInfo {
    lmr_table: LMRTable,
    history_table: [[i32; 64]; 12],
    killer_moves: [[Move; MAX_PLY]; 2],
}

struct LMRTable {
    reduction_table: [[[i32; 32]; 32]; 2],
}

impl Default for LMRTable {
    fn default() -> Self {
        //formula for reductions from Wiess chess engine
        let mut reduction_table = [[[0; 32]; 32]; 2];
        let (mut depth, mut played) = (0, 0);
        while depth < 32 {
            while played < 32 {
                reduction_table[0][depth][played] =
                    (0.33 + f64::ln(depth as f64) * f64::ln(played as f64) / 3.20) as i32;
                //tactical move
                reduction_table[1][depth][played] =
                    (1.64 + f64::ln(depth as f64) * f64::ln(played as f64) / 2.80) as i32;
                //quiet move
                played += 1;
            }
            depth += 1;
        }
        LMRTable { reduction_table }
    }
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            lmr_table: LMRTable::default(),
            history_table: [[0i32; 64]; 12],
            killer_moves: [[NULL_MOVE; 64]; 2],
        }
    }
}

pub struct Searcher {
    pub pv_length: [usize; 64],
    pub pv: [[Move; MAX_PLY]; MAX_PLY],
    pub tt_white: HashMap<u64, TTEntry>,
    pub tt_black: HashMap<u64, TTEntry>,
    pub ply: usize,
    pub nodes: usize,
    pub end_time: Instant,
    pub moves_fully_searched: usize,
    info: SearchInfo,
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
        for key in REPETITION_TABLE.iter().take(position.ply - 1) {
            //take ply - 1 because the start position (with 0 ply) is included
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
            pv_length: [0usize; 64],
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            tt_white: HashMap::new(), //128 MB
            tt_black: HashMap::new(), //seems to work pretty well (avoids t1 errors)
            ply: 0,
            nodes: 0,
            end_time,
            moves_fully_searched: 0,
            info: SearchInfo::default(),
        }
    }

    pub fn negamax(
        &mut self,
        position: &mut Board,
        mut depth: usize,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        /*
         return a score that can never be >= alpha if the search is cancelled
         other engines return zero here but I don't see how this would work
         in cases where 0 > alpha. Returning value that gets recursively passed
         so that it is equal to -INFINITY for the engine should work imo because
         if this happens on PV move it breaks during the main moves loop below
         and so count of moves searched fully is zero -> discard result
        */
        if Instant::now() > self.end_time && self.ply != 0 {
            return match self.ply % 2 {
                0 => -INFINITY,
                1 => INFINITY,
                _ => unreachable!(),
            };
        }
        let pv_node = beta - alpha != 1;
        let root = self.ply == 0;
        //full window search

        self.pv_length[self.ply] = self.ply;

        if depth == 0 {
            //qsearch on leaf nodes
            return self.quiescence_search(position, alpha, beta);
        }

        let mut hash_flag = EntryFlag::UpperBound;
        self.nodes += 1;

        let mut best_move = NULL_MOVE; //used for TT hash -> move ordering
                                       //this is useful in cases where it cannot return the eval of the hash lookup
                                       //due to the bounds, but it can use the best_move field for move ordering

        if !root {
            //check 50 move rule, repetition and insufficient material
            unsafe {
                if is_drawn(position) {
                    return 0;
                }
            }

            /*
             mate distance pruning:
             check if line is so good/bad that being mated in the current ply
             or mating in the next ply would not change alpha/beta
            */
            let r_alpha = cmp::max(alpha, -INFINITY + self.ply as i32);
            let r_beta = cmp::min(beta, INFINITY - self.ply as i32 - 1);
            if r_alpha >= r_beta {
                return r_alpha;
            }

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

        let tt_move_capture = if best_move.is_null() {
            false
        } else {
            best_move.is_capture(position)
        };

        let is_check = match position.side_to_move {
            //used in both search extensions and LMR
            Colour::White => is_attacked(lsfb(position.bitboards[WK]), Colour::Black, position),
            Colour::Black => is_attacked(lsfb(position.bitboards[BK]), Colour::White, position),
        };

        let static_eval = evaluate(position);

        //reset killers for child nodes
        self.info.killer_moves[0][self.ply + 1] = NULL_MOVE;
        self.info.killer_moves[1][self.ply + 1] = NULL_MOVE;

        if !is_check && !pv_node {
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

            //eval is very low so only realistic way to increase it is with captures
            //we only need to qsearch to evaluate the position
            if depth <= MAX_RAZOR_DEPTH && static_eval + RAZORING_MARGIN * (depth as i32) <= alpha {
                let score = self.quiescence_search(position, alpha, beta);
                if score > alpha {
                    return score;
                }
            }
        }

        if !position.is_kp_endgame()
            && !position.last_move_null
            && static_eval >= beta
            && depth >= NULLMOVE_MIN_DEPTH
            && !is_check
            && !root
        {
            //ok to null-move prune
            let ep_reset = make_null_move(position);
            self.ply += 1;
            //idea that if opponent cannot improve their position with 2 moves in a row
            //the first of these moves must be bad
            let r = 2 + depth as i32 / 4 + cmp::min((static_eval - beta) / 256, 3);
            let reduced_depth = cmp::max(depth as i32 - r, 1) as usize;
            let null_move_eval = -self.negamax(position, reduced_depth, -beta, -beta + 1);
            //minimal window used because all that matters is whether the search result is better than beta
            undo_null_move(position, ep_reset);
            self.ply -= 1;
            if null_move_eval >= beta {
                return beta;
            }
        }

        /*
         Internal Iterative Deepening:
         pv node and no tt hit -> move ordering will be terrible
         so do a shallower search to rectify move ordering
         by fixing history tables and pv move
         according to wiki this should make little difference on average
         but should make the search more consistent
        */
        if pv_node && depth > 3 && best_move.is_null() {
            self.negamax(position, depth - 2, alpha, beta);
        }

        if is_check && !root {
            depth += 1;
            //if this occurs at ply zero then next search iteration will basically be skipped
            //because of hash lookup
        }

        /*
         Generate pseudo-legal moves here because this should be faster in cases where
         the search is pruned early, and so we don't actually have to check whether later
         pseudo-legal moves are legal. The downside of this is that these can theoretically
         interfere with move ordering, but my testing seems to show that this ultimately results
         in a net performace gain due to higher NPS.
        */
        let mut child_nodes = MoveList::gen_moves(position);
        child_nodes.order_moves(position, self, &best_move);

        let mut moves_played = 0;
        let mut skip_quiets = false;

        for m in child_nodes.moves {
            if m.is_null() {
                //no pseudolegal moves left in move list
                break;
            }

            let tactical = m.is_tactical(position);
            let quiet = !tactical;
            let not_mated = alpha > -INFINITY + MAX_SEARCH_DEPTH as i32;
            //must be done before making the move on the board

            let is_killer = m == self.info.killer_moves[0][self.ply]
                || m == self.info.killer_moves[1][self.ply];

            if !root && not_mated {
                //skip quiet moves that we do not expect much from
                if quiet && skip_quiets && !is_killer {
                    //this is kinda messy but you have to know whether the move was legal
                    //to update the moves_played counter
                    let (commit, ok) = position.try_move(m);
                    position.undo_move(m, commit);
                    if ok {
                        moves_played += 1;
                    }
                    continue;
                }

                let r: i32 = self.info.lmr_table.reduction_table[quiet as usize]
                    [cmp::min(depth, 31)][cmp::min(moves_played, 31)];
                let lmr_depth = depth as i32 - 1 - r;

                /*
                 SEE Pruning: if the opponent move fails to beat a depth dependent
                 SEE threshold, skip it
                */
                if lmr_depth <= SEE_PRUNING_DEPTH && moves_played > 1 && !pv_node {
                    let margin = if tactical {
                        SEE_NOISY_MARGIN
                    } else {
                        SEE_QUIET_MARGIN
                    };
                    let threshold = margin * depth as i32;
                    //prune if move fails to beat SEE threshold
                    if !m.static_exchange_evaluation(position, threshold) {
                        //as above
                        let (commit, ok) = position.try_move(m);
                        position.undo_move(m, commit);
                        if ok {
                            moves_played += 1;
                        }
                        continue;
                    }
                }

                //this can definitely be done way better
                //by measuring whether search is improving
                if depth <= LMP_DEPTH && moves_played > depth * depth + 2 && !is_check {
                    skip_quiets = true;
                }
            }

            let (commit, ok) = position.try_move(m);
            //test if move is legal and make it at the same time
            //this obv faster that making the move to check if it is legal
            //then unmaking it and making it again for the search

            if !ok {
                position.undo_move(m, commit);
                continue;
            }

            moves_played += 1;

            self.ply += 1;
            //update after pruning above

            let eval = match moves_played == 1 {
                //note that this is one because the variable is updates above
                true => -self.negamax(position, depth - 1, -beta, -alpha),
                //normal search on pv move (no moves searched yet)
                false => {
                    /*
                     non-pv move -> search with reduced window
                     this assumes that our move ordering is good enough
                     that we will be able to prove that these moves are bad
                     often enough that it outweighs the cost of re-searching
                     then if we are unable to prove so
                    */
                    let mut reduction_eval = match moves_played > FULL_DEPTH_MOVES
                        && depth >= REDUCTION_LIMIT
                        && not_mated
                        && reduction_ok(tactical, is_check)
                    {
                        true => {
                            let mut r: i32 = self.info.lmr_table.reduction_table[quiet as usize]
                                [cmp::min(depth, 31)][cmp::min(moves_played, 31)];

                            //increase reduction for non-pv nodes
                            r += !pv_node as i32;
                            //increase reduction for quiet moves where tt move is noisy
                            r += tt_move_capture as i32;

                            let mut reduced_depth = cmp::max(depth as i32 - r - 1, 1) as usize;
                            reduced_depth = usize::clamp(reduced_depth, 1, depth);
                            //avoid dropping into qsearch or extending

                            -self.negamax(position, reduced_depth, -alpha - 1, -alpha)
                        }
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

            position.undo_move(m, commit);
            self.ply -= 1;

            if Instant::now() > self.end_time && self.ply == 0 {
                break;
                //as above
            }

            unsafe {
                //the second condition is to be sure that this results from a full search
                //and not a search initiated by IID
                if self.ply == 0 && depth == START_DEPTH {
                    self.moves_fully_searched += 1;
                    //used to ensure in the iterative deepening search that
                    //at least one move has been searched fully
                }
            }

            if eval > alpha {
                alpha = eval;

                //search failed high
                if alpha >= beta {
                    //only write quiet moves into history table because captures
                    //will be scored separately
                    self.update_search_tables(
                        position,
                        &child_nodes,
                        m,
                        tactical,
                        depth,
                        moves_played,
                    );
                    hash_flag = EntryFlag::LowerBound;
                    break;
                }

                hash_flag = EntryFlag::Exact;
                best_move = m;
                //NOTE: it is important that this is not above the beta cutoff
                //becuse not all moves have necessarily been searched to cause
                //the cutoff (one refutation)

                let next_ply = self.ply + 1;
                self.pv[self.ply][self.ply] = m;
                for j in next_ply..self.pv_length[next_ply] {
                    self.pv[self.ply][j] = self.pv[next_ply][j];
                    //copy from next row in pv table
                }
                self.pv_length[self.ply] = self.pv_length[next_ply];
            }
        }

        if moves_played == 0 {
            //no legal moves -> mate or stalemate
            return match is_check {
                true => -INFINITY + self.ply as i32,
                false => 0,
            };
        }

        let hash_entry = TTEntry::new(depth, alpha, hash_flag, best_move);
        match position.side_to_move {
            Colour::White => self.tt_white.write(position.hash_key, hash_entry),
            Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
        };
        alpha
    }

    pub fn quiescence_search(&mut self, position: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;

        unsafe {
            if is_drawn(position) {
                return 0;
            }
        }

        let mut hash_flag = EntryFlag::UpperBound;

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

        captures.order_moves(position, self, &best_move);
        for c in captures.moves {
            if c.is_null() {
                //no more pseudo-legal moves
                break;
            }

            let worst_case = SEE_VALUES[piece_type(position.pieces_array[c.square_to()])]
                - SEE_VALUES[piece_type(c.piece_moved(position))];

            if eval + worst_case > beta {
                //prune in the case that our move > beta even if we lose the piece
                //that we just moved
                return beta;
            }

            if !c.static_exchange_evaluation(position, 0) {
                //prune moves that fail see by threshold
                continue;
            }

            //prune neutral captures in bad positions (up to NxB)
            if eval + 200 <= alpha
                && !c.static_exchange_evaluation(
                    position,
                    SEE_VALUES[KNIGHT] - SEE_VALUES[BISHOP - 1],
                )
            {
                continue;
            }

            let (commit, ok) = position.try_move(c);

            if !ok {
                position.undo_move(c, commit);
                continue;
            }

            self.ply += 1;

            let eval = -self.quiescence_search(position, -beta, -alpha);
            position.undo_move(c, commit);
            self.ply -= 1;
            if eval > alpha {
                alpha = eval;
                hash_flag = EntryFlag::Exact;
                best_move = c;
            }
            if alpha >= beta {
                break;
            }
        }

        //write eval to hash table
        let hash_entry = TTEntry::new(0, alpha, hash_flag, best_move);
        match position.side_to_move {
            Colour::White => self.tt_white.write(position.hash_key, hash_entry),
            Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
        };
        alpha
    }

    pub fn update_search_tables(
        &mut self,
        b: &Board,
        moves: &MoveList,
        cutoff_move: Move,
        tactical: bool,
        depth: usize,
        moves_played: usize,
    ) {
        if !tactical {
            self.update_killer_moves(cutoff_move, tactical);
            self.update_history_table(b, moves, cutoff_move, depth, moves_played)
        }
    }

    pub fn update_killer_moves(&mut self, cutoff_move: Move, tactical: bool) {
        if !tactical && self.info.killer_moves[0][self.ply] != cutoff_move {
            //avoid saving the same killer move twice
            self.info.killer_moves[1][self.ply] = self.info.killer_moves[0][self.ply];
            self.info.killer_moves[0][self.ply] = cutoff_move;
        }
    }

    pub fn update_history_table(
        &mut self,
        b: &Board,
        moves: &MoveList,
        cutoff_move: Move,
        depth: usize,
        moves_played: usize,
    ) {
        //penalise all moves that have been checked and have not caused beta cutoff
        for i in 0..moves_played {
            if moves.moves[i].is_null() {
                break;
            }
            let piece = moves.moves[i].piece_moved(b);
            let target = moves.moves[i].square_to();
            if moves.moves[i] == cutoff_move {
                self.info.history_table[piece][target] += (depth * depth) as i32;
            } else {
                self.info.history_table[piece][target] -= (depth * depth) as i32;
            }
        }
    }

    pub fn reset_searcher(&mut self) {
        //try to keep tt, history and killer moves
        self.nodes = 0;
        self.pv_length = [0; 64];
        self.pv = [[NULL_MOVE; MAX_PLY]; MAX_PLY];
        self.ply = 0;
        self.moves_fully_searched = 0;

        //not clearing seems to be worse as even though the first few search depths are instant
        //the next depths don't have advantages of iterative deepening like pv and move ordering
        //heuristics
        self.tt_white = HashMap::new();
        self.tt_black = HashMap::new();

        //reset move ordering heuristics
        self.info.killer_moves = [[NULL_MOVE; MAX_PLY]; 2];
        self.info.history_table = [[0; 64]; 12];
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

pub fn see_test() {
    let position1 = Board::from("8/7k/8/4p3/8/5N2/K7/8 w - - 0 1");
    let m = encode_move(F3, E5, NO_PIECE, NO_FLAG);
    let res1 = m.static_exchange_evaluation(&position1, 0);
    assert!(res1, "first see test position failed");

    let position2 = Board::from("8/2b4k/8/4p3/8/5N2/K7/8 w - - 0 1");
    let m = encode_move(F3, E5, NO_PIECE, NO_FLAG);
    let res2 = m.static_exchange_evaluation(&position2, 0);
    assert!(!res2, "second see test position failed");

    let position3 = Board::from("8/2b4k/8/4p3/8/5N2/K7/4R3 w - - 0 1");
    let m = encode_move(F3, E5, NO_PIECE, NO_FLAG);
    let res3 = m.static_exchange_evaluation(&position3, 0);
    assert!(res3, "third see test position failed");

    let position4 = Board::from("4q3/2b4k/8/4p3/8/5N2/K7/4R3 w - - 0 1");
    let m = encode_move(F3, E5, NO_PIECE, NO_FLAG);
    let res4 = m.static_exchange_evaluation(&position4, 0);
    assert!(!res4, "fourth see test position failed");

    let position5 = Board::from("4q3/2b4k/8/4p3/8/5N2/K7/Q3R3 w - - 0 1");
    let m = encode_move(F3, E5, NO_PIECE, NO_FLAG);
    let res5 = m.static_exchange_evaluation(&position5, 0);
    assert!(res5, "fifth see test position failed");

    //test start position with no captures
    let position6 = Board::from(STARTPOS);
    let m = encode_move(E2, E4, NO_PIECE, NO_FLAG);
    let res6 = m.static_exchange_evaluation(&position6, 0);
    assert!(res6, "sixth see test position failed");

    let position7 = Board::from("4k3/8/2n2b2/8/3P4/2P5/8/3K4 b - - 0 1");
    let m = encode_move(C6, D4, NO_PIECE, NO_FLAG);
    let res7 = m.static_exchange_evaluation(&position7, 0);
    assert!(!res7, "seventh see test position failed");

    //test sliding attack updates
    let position8 = Board::from("3q3k/3r4/3r4/3p4/8/3R4/3R4/3Q3K w - - 0 1");
    let m = encode_move(D3, D5, NO_PIECE, NO_FLAG);
    let res8 = m.static_exchange_evaluation(&position8, 0);
    assert!(!res8, "eighth see test position failed");

    let position9 = Board::from("7k/8/3r4/3p4/4P3/5B2/8/7K w - - 0 1");
    let m = encode_move(E4, D5, NO_PIECE, NO_FLAG);
    let res9 = m.static_exchange_evaluation(&position9, 0);
    assert!(res9, "ninth see test position failed");

    println!("see test passed");
}

//same as MG evaluation weights
const SEE_VALUES: [i32; 6] = [85, 306, 322, 490, 925, INFINITY];

impl Move {
    fn static_exchange_evaluation(self, b: &Board, threshold: i32) -> bool {
        /*
         Iterative approach to SEE inspired by engine Ethereal. This is much faster
         than the recursive implementation I tried to make becuase most of the attack
         bitboards won't change during the SEE search so it's faster to keep them and
         only update slider attack bitboards when it's possible that they changed.
         This also avoids using make_move() and undo_move().
        */
        let sq_from = self.square_from();
        let sq_to = self.square_to();

        let mut next_victim = match self.is_promotion() {
            true => match b.side_to_move {
                //only consider queen promotions
                Colour::White => WQ,
                Colour::Black => BQ,
            },
            false => self.piece_moved(b),
        };

        let mut balance = match b.pieces_array[sq_to] {
            NO_PIECE => 0,
            k => SEE_VALUES[piece_type(k)],
        } + threshold;

        if self.is_promotion() {
            balance += SEE_VALUES[QUEEN] - SEE_VALUES[PAWN];
        }

        if balance < 0 {
            //bad even in best case
            return false;
        }

        balance -= SEE_VALUES[piece_type(next_victim)];

        if balance >= 0 {
            //good even in worst case
            return true;
        }

        let bishop_attackers =
            b.bitboards[WB] | b.bitboards[BB] | b.bitboards[WQ] | b.bitboards[BQ];
        let rook_attackers = b.bitboards[WR] | b.bitboards[BR] | b.bitboards[WQ] | b.bitboards[BQ];

        let mut occupancies = b.occupancies[BOTH] ^ (set_bit(sq_from, 0) | set_bit(sq_to, 0));

        let mut attackers = get_attackers(sq_to, Colour::White, b, occupancies)
            | get_attackers(sq_to, Colour::Black, b, occupancies);

        let mut colour = match b.side_to_move {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        };

        loop {
            let side_attackers = attackers
                & b.occupancies[match colour {
                    Colour::White => WHITE,
                    Colour::Black => BLACK,
                }];
            //doesn't matter that actual board struct isn't getting updated because attackers
            //that get traded off will get popped from the attackers bitboard

            if side_attackers == 0 {
                break;
            }

            let (min, max) = match colour {
                Colour::White => (WP, BP),
                Colour::Black => (BP, 12),
            };

            for piece in min..max {
                if side_attackers & b.bitboards[piece] > 0 {
                    next_victim = piece;
                    break;
                }
            }

            occupancies ^= set_bit(lsfb(side_attackers & b.bitboards[next_victim]), 0);

            if piece_type(next_victim) == PAWN
                || piece_type(next_victim) == BISHOP
                || piece_type(next_victim) == QUEEN
            {
                //only diagonal moves can reveal new diagonal attackers
                attackers |= get_bishop_attacks(sq_to, occupancies) & bishop_attackers;
            }

            if piece_type(next_victim) == ROOK || piece_type(next_victim) == QUEEN {
                //same for rook attacks
                attackers |= get_rook_attacks(sq_to, occupancies) & rook_attackers;
            }

            attackers &= occupancies;
            colour = match colour {
                Colour::White => Colour::Black,
                Colour::Black => Colour::White,
            };

            balance = -balance - 1 - SEE_VALUES[piece_type(next_victim)];

            if balance >= 0 {
                //if last move was king move and opponent still has attackers, the move
                //must have been illegal
                if next_victim == KING
                    && (attackers
                        & b.occupancies[match colour {
                            Colour::White => WHITE,
                            Colour::Black => BLACK,
                        }])
                        > 0
                {
                    colour = match colour {
                        Colour::White => Colour::Black,
                        Colour::Black => Colour::White,
                    };
                }
                break;
            }
        }

        //side to move after the loop loses
        b.side_to_move != colour
    }

    pub fn score_move(self, b: &mut Board, s: &Searcher, hash_move: &Move) -> i32 {
        /*
          MOVE ORDER:
        - TT Move
        - PV Move
        - Queen Promotion
        - Winning Capture + E.P.
        - Killers
        - History
        - Losing Capture
        - Underpromotion
         */

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
            let winning_capture = self.static_exchange_evaluation(b, 0);
            match winning_capture {
                true => WINNING_CAPTURE + MVV_LVA[victim_type][attacker_type],
                false => LOSING_CAPTURE + MVV_LVA[victim_type][attacker_type],
            }
        } else if self.is_promotion() {
            //maybe this should fo before checking if capture
            //because of promotions that are also captures
            match self.promoted_piece() {
                //promotions sorted by likelihood to be good
                QUEEN => QUEEN_PROMOTION,
                KNIGHT => UNDER_PROMOTION,
                ROOK => UNDER_PROMOTION,
                BISHOP => UNDER_PROMOTION,
                _ => unreachable!(),
            }
        } else if self.is_en_passant() {
            MVV_LVA[PAWN][PAWN]
        } else if s.info.killer_moves[0][s.ply] == self {
            FIRST_KILLER_MOVE //after captures
        } else if s.info.killer_moves[1][s.ply] == self {
            SECOND_KILLER_MOVE
        } else {
            s.info.history_table[self.piece_moved(b)][self.square_to()]
        }
    }
}

#[derive(Copy, Clone)]
pub struct MoveOrderEntry<'a> {
    m: &'a Move,
    score: i32,
}

impl MoveList {
    pub fn order_moves(&mut self, board: &mut Board, s: &Searcher, best_move: &Move) {
        let mut ordered_moves = [MoveOrderEntry {
            m: &NULL_MOVE,
            score: -INFINITY,
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
    let time_until_flag = time - TIME_TO_MOVE;

    let ideal_time = match moves_to_go {
        0 => {
            (match ply {
                0..=10 => time_until_flag / 45,
                11..=16 => time_until_flag / 25,
                _ => time_until_flag / 20,
            } + increment
                + TIME_TO_START_SEARCH)
        }
        _ => {
            (match ply {
                0..=10 => (time_until_flag) as f32 / 40.0,
                11..=16 => (time_until_flag) as f32 / (moves_to_go as f32),
                _ => {
                    (time_until_flag) as f32
                        / cmp::max((moves_to_go as f32 / 2.0) as usize, 1) as f32
                }
            }) as usize
                + TIME_TO_START_SEARCH
                + increment
        }
    };

    //prevent that high increment > time left breaks this
    cmp::min(time_until_flag, ideal_time)
}

impl Move {
    pub fn uci(self) -> String {
        let mut res = String::new();
        res += coordinate(self.square_from()).as_str();
        res += coordinate(self.square_to()).as_str();

        if self.is_promotion() {
            res += match self.promoted_piece() {
                KNIGHT => "n",
                BISHOP => "b",
                ROOK => "r",
                QUEEN => "q",
                _ => unreachable!(),
            }
        }
        res
    }
}

pub fn best_move(
    position: &mut Board,
    time_left: usize,
    inc: usize,
    moves_to_go: usize,
    movetime: usize,
    s: &mut Searcher,
) -> MoveData {
    let start = Instant::now();
    let move_duration = match movetime {
        0 => Duration::from_millis(
            move_time(time_left, inc, moves_to_go, position.ply)
                .try_into()
                .unwrap(),
        ),
        k => Duration::from_millis(k as u64),
    };

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
        unsafe { START_DEPTH = depth };
        eval = s.negamax(position, depth, alpha, beta);
        if s.moves_fully_searched == 0 {
            //search cancelled before even pv was searched
            eval = previous_eval;
            s.pv = previous_pv;
            s.pv_length = previous_pv_length;
        } else {
            // >= 1 move searched ok
            //this can sometimes cause it to report eval of -INFINITY if it falls
            //out of aspiration window and then searches no moves fully
            if eval != -INFINITY {
                //note that -INFINITY can only happen in the case of aspiration window bug mentioned
                //above, as mates will be -INFINITY + some ply that is at least one
                previous_eval = eval;
            }
            previous_pv = s.pv;
            previous_pv_length = s.pv_length;
        }

        println!(
            "info depth {} score cp {} nodes {} pv{} time {} nps {}",
            depth,
            eval,
            s.nodes,
            {
                let mut pv = String::new();
                for i in 0..s.pv_length[0] {
                    pv += " ";
                    pv += s.pv[0][i].uci().as_str();
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
            /*
             more than half of time used -> no point starting another search as its likely
             that zero moves will be searched fully.
             ofc this also catches situations where all of move duration has elapsed
            */
            break;
        }

        s.moves_fully_searched = 0;
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
