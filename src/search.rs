use crate::board::*;
use crate::eval::*;
use crate::get_bishop_attacks;
use crate::get_rook_attacks;
use crate::helper::*;
use crate::magic::*;
use crate::movegen::*;
use crate::r#move::*;
use crate::transposition::*;
use crate::uci::*;
use crate::STARTPOS;

use crate::types::*;

use std::cmp;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const INFINITY: i32 = 1_000_000_000;
pub const MAX_PLY: usize = 64;
pub const MAX_SEARCH_DEPTH: usize = 32;
pub const REDUCTION_LIMIT: usize = 2;

const FULL_DEPTH_MOVES: usize = 1;

const SINGULARITY_DE_MARGIN: i32 = 40;

#[allow(dead_code)]
const NULLMOVE_MAX_DEPTH: usize = 6;
#[allow(dead_code)]
const NULLMOVE_MIN_DEPTH: usize = 3;

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
const SEE_QSEARCH_MARGIN: i32 = 1;

#[allow(unused)]
const LMP_DEPTH: usize = 5;

const DO_SINGULARITY_EXTENSION: bool = false;

const HASH_MOVE_SCORE: i32 = 1_000_000;
const PV_MOVE_SCORE: i32 = 500_000;
const QUEEN_PROMOTION: i32 = 400_000;
const WINNING_CAPTURE: i32 = 300_000;
const FIRST_KILLER_MOVE: i32 = 200_000;
const SECOND_KILLER_MOVE: i32 = 100_000;
const LOSING_CAPTURE: i32 = -100_000;
const UNDER_PROMOTION: i32 = -200_000;

pub const MAX_GAME_PLY: usize = 1024;

const MIN_MOVE_TIME: usize = 1; //make sure move time is never 0
const MOVE_OVERHEAD: usize = 50;

#[allow(unused)]
const TIME_TO_START_SEARCH: usize = 0; //initialise big TT (if not using HashMap)
                                       //leave 100ms total margin

pub static mut REPETITION_TABLE: [u64; MAX_GAME_PLY] = [0u64; MAX_GAME_PLY];

#[derive(Copy, Clone)]
struct SearchStackEntry {
    eval: i32,
}

struct SearchInfo {
    ss: [SearchStackEntry; MAX_SEARCH_DEPTH],
    lmr_table: LMRTable,
    history_table: [[i32; 64]; 12],
    killer_moves: [[Move; MAX_PLY]; 2],
    excluded: Option<Move>,
}

struct LMRTable {
    reduction_table: [[[i32; 32]; 32]; 2],
}

impl Default for SearchStackEntry {
    fn default() -> Self {
        Self { eval: -INFINITY }
    }
}

impl Default for LMRTable {
    fn default() -> Self {
        //formula for reductions from Weiss chess engine
        let mut reduction_table = [[[0; 32]; 32]; 2];
        for depth in 0..32 {
            for played in 0..32 {
                reduction_table[0][depth][played] =
                    (0.33 + f64::ln(depth as f64) * f64::ln(played as f64) / 3.20) as i32;
                //tactical move
                reduction_table[1][depth][played] =
                    (1.64 + f64::ln(depth as f64) * f64::ln(played as f64) / 2.80) as i32;
                //quiet move
            }
        }
        LMRTable { reduction_table }
    }
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            ss: [SearchStackEntry::default(); MAX_SEARCH_DEPTH],
            lmr_table: LMRTable::default(),
            history_table: [[0i32; 64]; 12],
            killer_moves: [[NULL_MOVE; 64]; 2],
            excluded: None,
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
    pub timer: Timer,
    pub moves_fully_searched: usize,
    pub do_pruning: bool,
    info: SearchInfo,
}

pub struct Timer {
    stopped: bool,
    max_nodes: usize,
    end_time: Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            stopped: false,
            max_nodes: 0,
            end_time: Instant::now(),
        }
    }
}

struct NullMoveUndo {
    ep: Option<Square>,
    pinned: BitBoard,
}

fn reduction_ok(tactical: bool, in_check: bool) -> bool {
    !(tactical || in_check)
}

//make null move for NMP
//we have to update pinners but not checkers since NMP is never done while in check
fn make_null_move(b: &mut Board) -> NullMoveUndo {
    b.side_to_move = b.side_to_move.opponent();
    b.last_move_null = true;

    let pinned_reset = b.pinned;

    let colour = b.side_to_move;

    //SAFETY: there MUST be a king on the board
    let our_king = unsafe {
        lsfb(
            b.bitboards[match colour {
                Colour::White => Piece::WK,
                Colour::Black => Piece::BK,
            }],
        )
        .unwrap_unchecked()
    };

    let mut their_attackers = if colour == Colour::White {
        b.occupancies[BLACK]
            & ((BISHOP_EDGE_RAYS[our_king] & (b.bitboards[Piece::BB] | b.bitboards[Piece::BQ]))
                | ROOK_EDGE_RAYS[our_king] & (b.bitboards[Piece::BR] | b.bitboards[Piece::BQ]))
    } else {
        b.occupancies[WHITE]
            & ((BISHOP_EDGE_RAYS[our_king] & (b.bitboards[Piece::WB] | b.bitboards[Piece::WQ]))
                | ROOK_EDGE_RAYS[our_king] & (b.bitboards[Piece::WR] | b.bitboards[Piece::WQ]))
    };

    while let Some(sq) = lsfb(their_attackers) {
        let ray_between = RAY_BETWEEN[sq][our_king] & b.occupancies[BOTH];
        if count(ray_between) == 1 {
            b.pinned |= ray_between
        }
        their_attackers = pop_bit(sq, their_attackers);
    }

    if let Some(reset) = b.en_passant {
        b.en_passant = None;
        return NullMoveUndo {
            ep: Some(reset),
            pinned: pinned_reset,
        };
    }
    NullMoveUndo {
        ep: None,
        pinned: pinned_reset,
    }
}

fn undo_null_move(b: &mut Board, undo: &NullMoveUndo) {
    b.side_to_move = match b.side_to_move {
        Colour::White => Colour::Black,
        Colour::Black => Colour::White,
    };
    b.last_move_null = false;
    b.en_passant = undo.ep;
    b.pinned = undo.pinned;
}

fn is_insufficient_material(b: &Board) -> bool {
    if count(
        b.bitboards[Piece::WP]
            | b.bitboards[Piece::WR]
            | b.bitboards[Piece::WQ]
            | b.bitboards[Piece::BP]
            | b.bitboards[Piece::BR]
            | b.bitboards[Piece::BQ],
    ) != 0
    {
        return false;
    }
    if count(b.bitboards[Piece::WB]) >= 2 || count(b.bitboards[Piece::BB]) >= 2 {
        return false;
    }
    count(b.bitboards[Piece::WN]) <= 2 && count(b.bitboards[Piece::BN]) <= 2
    //can technically arise a position where KvKNN is mate so this
    //could cause some bug in theory lol
}

unsafe fn is_drawn(position: &Board) -> bool {
    if position.fifty_move == 100 {
        return true;
    }
    #[allow(static_mut_refs)]
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
    pub fn new(end_time: Instant, max_nodes: usize) -> Self {
        let timer = Timer {
            stopped: false,
            end_time,
            max_nodes,
        };

        Searcher {
            pv_length: [0usize; 64],
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            tt_white: HashMap::new(), //128 MB
            tt_black: HashMap::new(), //seems to work pretty well (avoids t1 errors)
            ply: 0,
            nodes: 0,
            timer,
            moves_fully_searched: 0,
            do_pruning: true,
            info: SearchInfo::default(),
        }
    }

    fn should_check_exit(&self) -> bool {
        const CHECK_INTERVAL: usize = 4095;
        self.nodes & CHECK_INTERVAL == 0
    }

    fn should_exit(&mut self) -> bool {
        if self.timer.stopped {
            return true;
        } else if self.should_check_exit() {
            self.timer.stopped =
                Instant::now() > self.timer.end_time || self.nodes >= self.timer.max_nodes;
            return self.timer.stopped;
        }
        false
    }

    //The purpose of the singularity() function is to prove that a move is better than alternatives by
    //a significant margin. If this is true, we should extend it since it is more important. This
    //function determines how much we should extend by.

    #[allow(clippy::too_many_arguments)]
    fn singularity(
        &mut self,
        position: &mut Board,
        best_move: Move,
        commit: &Commit,
        tt_score: i32,
        depth: usize,
        pv_node: bool,
        _alpha: i32,
        beta: i32,
        cutnode: bool,
    ) -> Option<i32> {
        position.undo_move(best_move, commit);
        self.ply -= 1;
        //undo move already made on board
        let threshold = std::cmp::max(tt_score - (depth as i32 * 2 + 20), -INFINITY);

        self.info.excluded = Some(best_move);

        let excluded_eval = self.negamax(position, depth / 2, threshold, threshold - 1, cutnode);

        self.info.excluded = None;

        if !pv_node && excluded_eval < threshold - SINGULARITY_DE_MARGIN {
            Some(2)
        } else if excluded_eval < threshold {
            Some(1)
        } else if threshold >= beta {
            //MultiCut: more than one move will be able to beat beta
            //here we return None to indicate that the search should terminate
            //and return beta
            None
        } else if tt_score >= beta {
            Some(-1)
        } else {
            Some(0)
        }
    }

    pub fn negamax(
        &mut self,
        position: &mut Board,
        depth: usize,
        mut alpha: i32,
        beta: i32,
        cutnode: bool,
    ) -> i32 {
        if self.should_exit() {
            return 0;
        }
        let pv_node = beta - alpha != 1;
        let root = self.ply == 0;
        //full window search

        self.pv_length[self.ply] = self.ply;

        if depth <= 0 {
            //qsearch on leaf nodes
            return self.quiescence_search(position, alpha, beta);
        }

        let mut hash_flag = EntryFlag::UpperBound;
        self.nodes += 1;

        //NOTE: tt_score is only used in singular search, in which case we know that there is
        //definitely a hash result, so this value of 0 is never actually read
        let (mut tt_depth, mut tt_bound, mut tt_score) = (0, EntryFlag::Missing, 0);
        let mut best_move = NULL_MOVE; //used for TT hash -> move ordering
                                       //this is useful in cases where it cannot return the eval of the hash lookup
                                       //due to the bounds, but it can use the best_move field for move ordering

        //don't probe TB in singular search
        if !root && self.info.excluded.is_none() {
            //check 50 move rule, repetition and insufficient material
            unsafe {
                if is_drawn(position) {
                    return 0;
                }
            }

            // mate distance pruning:
            // check if line is so good/bad that being mated in the current ply
            // or mating in the next ply would not change alpha/beta
            let r_alpha = cmp::max(alpha, -INFINITY + self.ply as i32);
            let r_beta = cmp::min(beta, INFINITY - self.ply as i32 - 1);
            if r_alpha >= r_beta {
                return r_alpha;
            }

            let hash_lookup = match position.side_to_move {
                //hash lookup
                Colour::White => {
                    self.tt_white
                        .lookup(position.hash_key, alpha, beta, depth, &mut tt_score)
                }
                Colour::Black => {
                    self.tt_black
                        .lookup(position.hash_key, alpha, beta, depth, &mut tt_score)
                }
            };

            if let Some(k) = hash_lookup.eval {
                return k;
            } else if !hash_lookup.best_move.is_null() {
                best_move = hash_lookup.best_move;
                tt_depth = hash_lookup.depth;
                tt_bound = hash_lookup.flag;
            };
        }

        let tt_move = !best_move.is_null();
        let tt_move_capture = if best_move.is_null() {
            false
        } else {
            best_move.is_capture(position)
        };

        //reset killers for child nodes
        self.info.killer_moves[0][self.ply + 1] = NULL_MOVE;
        self.info.killer_moves[1][self.ply + 1] = NULL_MOVE;

        let in_check = position.checkers != 0;
        let mut improving = false;

        //avoid static pruning when in check or in singular search
        if !in_check && self.info.excluded.is_none() && self.do_pruning {
            let static_eval = evaluate(position);
            if self.ply < MAX_SEARCH_DEPTH {
                self.info.ss[self.ply] = SearchStackEntry { eval: static_eval };
            }

            //measuring whether the search is improving (better static eval than 2 tempi ago)
            //is useful in adjusting how we should prune/reduce. if the search is improving,
            //we should be more cautious reducing. if not, we can reduce more
            improving = match self.ply {
                2..=31 => self.info.ss[self.ply].eval > self.info.ss[self.ply - 2].eval,
                _ => false,
            };

            //Static pruning: here we attempt to show that the position does not require any further
            //search
            if !pv_node {
                //Beta Pruning / Reverse Futility Pruning:
                //If eval >= beta + some margin, assume that we can achieve at least beta
                if depth <= BETA_PRUNING_DEPTH
                    && static_eval
                        - (BETA_PRUNING_MARGIN * cmp::max(depth - improving as usize, 0)) as i32
                        >= beta
                {
                    return static_eval;
                }

                //eval is so bad that even a huge margin fails to raise alpha
                if depth <= ALPHA_PRUNING_DEPTH && static_eval + ALPHA_PRUNING_MARGIN <= alpha {
                    return static_eval;
                }

                //eval is very low so only realistic way to increase it is with captures
                //we only need to qsearch to evaluate the position
                if depth <= MAX_RAZOR_DEPTH
                    && static_eval + RAZORING_MARGIN * (depth as i32) <= alpha
                {
                    let score = self.quiescence_search(position, alpha, beta);
                    if score > alpha {
                        return score;
                    }
                }

                // Null move pruning: if we cannot improve our position with 2 moves in a row,
                // then the first of these moves is probably bad (exception is zugzwang)
                // the third condition is a technique I found in various strong engines
                // (SF, Obsidian etc.)
                if !position.is_kp_endgame()
                    && !position.last_move_null
                    && static_eval >= beta + 200 - 20 * (depth as i32)
                    && !root
                {
                    let undo = make_null_move(position);
                    self.ply += 1;
                    let r = 2 + depth as i32 / 4 + cmp::min((static_eval - beta) / 256, 3);
                    let reduced_depth = cmp::max(depth as i32 - r, 1) as usize;
                    let null_move_eval =
                        -self.negamax(position, reduced_depth, -beta, -beta + 1, !cutnode);
                    //minimal window used because all that matters is whether the search result is better than beta
                    undo_null_move(position, &undo);
                    self.ply -= 1;
                    if null_move_eval >= beta {
                        return beta;
                    }
                }
            }
        }

        // Generate pseudo-legal moves here because this is faster in cases where
        // the search is pruned early, and so we don't actually have to check whether later
        // pseudo-legal moves are legal.
        let mut move_list = MoveList::gen_moves::<false>(position);
        move_list.order_moves(position, self, &best_move);

        let (mut moves_played, mut moves_seen) = (0, 0);
        //the former of these is for legal moves we actually search
        //the latter for pseudo-legal moves we consider
        let mut skip_quiets = false;

        for m in move_list.moves {
            if m.is_null() {
                //no pseudolegal moves left in move list
                break;
            } else if let Some(n) = self.info.excluded {
                if n == m {
                    continue;
                }
            }

            //from what I can see strong engines update this before checking whether or not the
            //move is legal
            moves_seen += 1;

            let tactical = m.is_tactical(position);
            let quiet = !tactical;
            let not_mated = alpha > -INFINITY + MAX_SEARCH_DEPTH as i32;
            //must be done before making the move on the board

            let is_killer = m == self.info.killer_moves[0][self.ply]
                || m == self.info.killer_moves[1][self.ply];

            //Early Pruning: try to prune moves before we search them properly
            //by showing that they're not worth investigating
            if !root && not_mated && self.do_pruning {
                if quiet && skip_quiets && !is_killer {
                    continue;
                }
                let r: i32 = self.info.lmr_table.reduction_table[quiet as usize]
                    [cmp::min(depth, 31)][cmp::min(moves_seen, 31)]
                    + (!improving) as i32;
                let lmr_depth = std::cmp::max(depth as i32 - 1 - r, 0);

                //SEE Pruning: if a move fails SEE by a depth-dependent threshold,
                //prune it
                if lmr_depth <= SEE_PRUNING_DEPTH && moves_seen > 1 && !pv_node {
                    let margin = if tactical {
                        SEE_NOISY_MARGIN
                    } else {
                        SEE_QUIET_MARGIN
                    };
                    let threshold = margin * depth as i32;
                    //prune if move fails to beat SEE threshold
                    if !m.static_exchange_evaluation(position, threshold) {
                        continue;
                    }
                }

                //Late Move Pruning: after a certain point start skipping all quiets after the current
                //move. The threshold I'm currently using comes from Weiss
                let lmp_threshold = match improving {
                    true => depth * depth + 2,
                    false => depth * depth / 2,
                };
                if depth <= LMP_DEPTH && moves_seen > lmp_threshold && !in_check {
                    skip_quiets = true;
                }
            }

            let Ok(commit) = position.try_move(m) else {
                continue;
            };

            moves_played += 1;
            self.ply += 1;
            //update after pruning above

            //A singular move is a move which seems to be forced or at least much stronger than
            //others. We should therefore extend to investigate it further.

            // after I implemented it I realised singularity extension currently loses elo for
            // Panda, but I didn't want to throw away the code
            let maybe_singular = DO_SINGULARITY_EXTENSION
                && !root
                && depth >= 8
                && self.info.excluded.is_none()
                && m == best_move
                && tt_depth >= depth - 3
                && tt_bound != EntryFlag::UpperBound;

            let extension = if maybe_singular {
                self.singularity(
                    position, best_move, &commit, tt_score, depth, pv_node, alpha, beta, cutnode,
                )
            } else {
                Some((in_check && !root) as i32)
            };

            if extension.is_none() {
                //MultiCut case from singularity() function
                return tt_score - (depth as i32 * 2);
            } else if maybe_singular {
                position.play_unchecked(best_move);
                self.ply += 1;
                //we unmade the move while calling the singularity() function
            }

            let new_depth = i32::clamp(
                depth as i32 + extension.unwrap() - 1,
                0,
                MAX_SEARCH_DEPTH as i32,
            ) as usize;

            let eval = if moves_played == 1 {
                //note that this is one because the variable is updated above
                -self.negamax(position, new_depth, -beta, -alpha, false)
                //normal search on pv move (no moves searched yet)
            } else {
                // non-pv move -> search with reduced window
                // this assumes that our move ordering is good enough
                // that we will be able to prove that these moves are bad
                // often enough that it outweighs the cost of re-searching
                // then if we are unable to prove so

                let mut r: i32 = self.info.lmr_table.reduction_table[quiet as usize]
                    [cmp::min(depth, 31)][cmp::min(moves_played, 31)];

                let mut reduction_eval = if moves_played
                    > (FULL_DEPTH_MOVES + pv_node as usize + !tt_move as usize + root as usize)
                    && depth >= REDUCTION_LIMIT
                    && reduction_ok(tactical, in_check)
                {
                    //decrease reduction for pv-nodes
                    r -= pv_node as i32;
                    //increase reduction for quiet moves where tt move is noisy
                    r += tt_move_capture as i32;
                    //reduce more in nodes where search isn't improving
                    r += !improving as i32;
                    //reduce more in cutnodes
                    r += cutnode as i32;

                    let mut reduced_depth = cmp::max(new_depth as i32 - r, 1) as usize;
                    reduced_depth = usize::clamp(reduced_depth, 1, new_depth);
                    //avoid dropping into qsearch or extending

                    -self.negamax(position, reduced_depth, -alpha - 1, -alpha, true)
                } else {
                    alpha + 1
                };
                if reduction_eval > alpha {
                    //failed to prove that move is bad -> re-search with same depth but reduced
                    //window
                    reduction_eval =
                        -self.negamax(position, new_depth, -alpha - 1, -alpha, !cutnode);
                }

                if reduction_eval > alpha && reduction_eval < beta {
                    //move actually inside PV window -> search at full depth
                    reduction_eval = -self.negamax(position, new_depth, -beta, -alpha, false);
                }
                reduction_eval
            };

            position.undo_move(m, &commit);
            self.ply -= 1;

            if self.timer.stopped {
                return 0;
            }

            if self.ply == 0 {
                self.moves_fully_searched += 1;
                //used to ensure in the iterative deepening search that
                //at least one move has been searched fully
            }

            if eval > alpha {
                alpha = eval;

                let next_ply = self.ply + 1;
                self.pv[self.ply][self.ply] = m;
                for j in next_ply..self.pv_length[next_ply] {
                    self.pv[self.ply][j] = self.pv[next_ply][j];
                    //copy from next row in pv table
                }
                self.pv_length[self.ply] = self.pv_length[next_ply];

                //search failed high
                if alpha >= beta {
                    //only write quiet moves into history table because captures
                    //will be scored separately
                    self.update_search_tables(
                        position,
                        &move_list,
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
            }
        }

        if moves_played == 0 {
            //no legal moves -> mate or stalemate
            return match in_check {
                true => -INFINITY + self.ply as i32,
                false => 0,
            };
        }

        if !self.timer.stopped {
            let hash_entry = TTEntry::new(depth, alpha, hash_flag, best_move);
            match position.side_to_move {
                Colour::White => self.tt_white.write(position.hash_key, hash_entry),
                Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
            };
        }
        alpha
    }

    pub fn quiescence_search(&mut self, position: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;

        unsafe {
            if is_drawn(position) {
                return 0;
            }
        }

        if self.should_exit() {
            return 0;
        }

        let mut hash_flag = EntryFlag::UpperBound;

        let mut _tt_score = 0;

        let hash_lookup = match position.side_to_move {
            //hash lookup
            Colour::White => {
                self.tt_white
                    .lookup(position.hash_key, alpha, beta, 0, &mut _tt_score)
            }
            Colour::Black => {
                self.tt_black
                    .lookup(position.hash_key, alpha, beta, 0, &mut _tt_score)
            } //lookups with depth zero because any TT entry will necessarily
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

        let in_check = position.checkers != 0;

        let mut captures = if in_check {
            //try generating all moves in the case that we're in check because it's unsound to rely
            //on static eval + if could be mate
            MoveList::gen_moves::<false>(position)
        } else {
            MoveList::gen_moves::<true>(position)
        };

        captures.order_moves(position, self, &best_move);

        //TODO: try skipping quiets after we've proved we're not mated and we've seen a few

        for c in captures.moves {
            if c.is_null() {
                //no more pseudo-legal moves
                break;
            }

            let worst_case = SEE_VALUES
                [piece_type(unsafe { position.pieces_array[c.square_to()].unwrap_unchecked() })]
                - SEE_VALUES[piece_type(c.piece_moved(position))];

            if eval + worst_case > beta {
                //prune in the case that our move > beta even if we lose the piece
                //that we just moved
                return beta;
            }

            if !c.static_exchange_evaluation(position, SEE_QSEARCH_MARGIN) {
                //prune moves that fail see by threshold
                continue;
            }

            if eval + 200 <= alpha
                && !c.static_exchange_evaluation(
                    position,
                    SEE_VALUES[PieceType::Knight] - SEE_VALUES[PieceType::Bishop] - 1,
                )
            {
                continue;
            }

            let Ok(commit) = position.try_move(c) else {
                continue;
            };

            self.ply += 1;

            let eval = -self.quiescence_search(position, -beta, -alpha);
            position.undo_move(c, &commit);
            self.ply -= 1;

            if self.timer.stopped {
                return 0;
            }

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
        if !self.timer.stopped {
            let hash_entry = TTEntry::new(0, alpha, hash_flag, best_move);
            match position.side_to_move {
                Colour::White => self.tt_white.write(position.hash_key, hash_entry),
                Colour::Black => self.tt_black.write(position.hash_key, hash_entry),
            };
        }
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

        self.timer.stopped = false;

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
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res1 = m.static_exchange_evaluation(&position1, 0);
    assert!(res1, "first see test position failed");

    let position2 = Board::from("8/2b4k/8/4p3/8/5N2/K7/8 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res2 = m.static_exchange_evaluation(&position2, 0);
    assert!(!res2, "second see test position failed");

    let position3 = Board::from("8/2b4k/8/4p3/8/5N2/K7/4R3 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res3 = m.static_exchange_evaluation(&position3, 0);
    assert!(res3, "third see test position failed");

    let position4 = Board::from("4q3/2b4k/8/4p3/8/5N2/K7/4R3 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res4 = m.static_exchange_evaluation(&position4, 0);
    assert!(!res4, "fourth see test position failed");

    let position5 = Board::from("4q3/2b4k/8/4p3/8/5N2/K7/Q3R3 w - - 0 1");
    let m = encode_move(Square::F3, Square::E5, None, NO_FLAG);
    let res5 = m.static_exchange_evaluation(&position5, 0);
    assert!(res5, "fifth see test position failed");

    //test start position with no captures
    let position6 = Board::from(STARTPOS);
    let m = encode_move(Square::E2, Square::E4, None, NO_FLAG);
    let res6 = m.static_exchange_evaluation(&position6, 0);
    assert!(res6, "sixth see test position failed");

    let position7 = Board::from("4k3/8/2n2b2/8/3P4/2P5/8/3K4 b - - 0 1");
    let m = encode_move(Square::C6, Square::D4, None, NO_FLAG);
    let res7 = m.static_exchange_evaluation(&position7, 0);
    assert!(!res7, "seventh see test position failed");

    //test sliding attack updates
    let position8 = Board::from("3q3k/3r4/3r4/3p4/8/3R4/3R4/3Q3K w - - 0 1");
    let m = encode_move(Square::D3, Square::D5, None, NO_FLAG);
    let res8 = m.static_exchange_evaluation(&position8, 0);
    assert!(!res8, "eighth see test position failed");

    let position9 = Board::from("7k/8/3r4/3p4/4P3/5B2/8/7K w - - 0 1");
    let m = encode_move(Square::E4, Square::D5, None, NO_FLAG);
    let res9 = m.static_exchange_evaluation(&position9, 0);
    assert!(res9, "ninth see test position failed");

    println!("see test passed");
}

//same as MG evaluation weights (haven't updated these in a while)
const SEE_VALUES: [i32; 6] = [85, 306, 322, 490, 925, INFINITY];

impl Move {
    pub fn static_exchange_evaluation(self, b: &Board, threshold: i32) -> bool {
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
                Colour::White => Piece::WQ,
                Colour::Black => Piece::BQ,
            },
            false => self.piece_moved(b),
        };

        let mut balance = match b.pieces_array[sq_to] {
            None => 0,
            Some(k) => SEE_VALUES[piece_type(k)],
        } + threshold;

        if self.is_promotion() {
            balance += SEE_VALUES[PieceType::Queen] - SEE_VALUES[PieceType::Pawn];
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

        let bishop_attackers = b.bitboards[Piece::WB]
            | b.bitboards[Piece::BB]
            | b.bitboards[Piece::WQ]
            | b.bitboards[Piece::BQ];
        let rook_attackers = b.bitboards[Piece::WR]
            | b.bitboards[Piece::BR]
            | b.bitboards[Piece::WQ]
            | b.bitboards[Piece::BQ];

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

            let pieces = match colour {
                Colour::White => WHITE_PIECES,
                Colour::Black => BLACK_PIECES,
            };

            for piece in pieces {
                if side_attackers & b.bitboards[piece] > 0 {
                    next_victim = piece;
                    break;
                }
            }

            //SAFETY: if this was zero we would have broken above
            occupancies ^= set_bit(
                unsafe { lsfb(side_attackers & b.bitboards[next_victim]).unwrap_unchecked() },
                0,
            );

            if piece_type(next_victim) == PieceType::Pawn
                || piece_type(next_victim) == PieceType::Bishop
                || piece_type(next_victim) == PieceType::Queen
            {
                //only diagonal moves can reveal new diagonal attackers
                attackers |= get_bishop_attacks(sq_to as usize, occupancies) & bishop_attackers;
            }

            if piece_type(next_victim) == PieceType::Rook
                || piece_type(next_victim) == PieceType::Queen
            {
                //same for rook attacks
                attackers |= get_rook_attacks(sq_to as usize, occupancies) & rook_attackers;
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
                if next_victim == Piece::WK
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
            let victim_type =
                piece_type(unsafe { b.pieces_array[self.square_to()].unwrap_unchecked() });
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
                PieceType::Queen => QUEEN_PROMOTION,
                PieceType::Knight => UNDER_PROMOTION,
                PieceType::Rook => UNDER_PROMOTION,
                PieceType::Bishop => UNDER_PROMOTION,
                _ => unreachable!(),
            }
        } else if self.is_en_passant() {
            MVV_LVA[PieceType::Pawn][PieceType::Pawn]
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

        for (i, m) in self.moves.iter().enumerate() {
            if m.is_null() {
                break;
            }
            ordered_moves[i].m = m;
            ordered_moves[i].score = m.score_move(board, s, best_move);
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

pub fn move_time(time: usize, increment: usize, moves_to_go: usize, _ply: usize) -> usize {
    let time_until_flag = time - MOVE_OVERHEAD;
    let default_moves_to_go: usize = 40;

    let m = if moves_to_go == 0 {
        default_moves_to_go
    } else {
        usize::clamp(moves_to_go, 2, default_moves_to_go)
    };

    //note time - increment must be +ve since we got increment last turn
    let ideal_time = time_until_flag / (m / 2) + increment;
    let t = cmp::min(ideal_time, time_until_flag);

    cmp::max(t, MIN_MOVE_TIME)
}

impl Move {
    pub fn uci(self) -> String {
        let mut res = String::new();
        res += coordinate(self.square_from()).as_str();
        res += coordinate(self.square_to()).as_str();

        if self.is_promotion() {
            res += match self.promoted_piece() {
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Rook => "r",
                PieceType::Queen => "q",
                _ => unreachable!(),
            }
        }
        res
    }
}

struct IterDeepData {
    eval: i32,
    pv: [[Move; MAX_PLY]; MAX_PLY],
    pv_length: [usize; MAX_PLY],

    delta: i32,
    alpha: i32,
    beta: i32,
    depth: usize,

    show_thinking: bool,
    start_time: Instant,
}

impl IterDeepData {
    fn new(start_time: Instant, show_thinking: bool) -> Self {
        Self {
            eval: 0,
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            pv_length: [0; MAX_PLY],
            delta: ASPIRATION_WINDOW,
            alpha: -INFINITY,
            beta: INFINITY,
            depth: 1,
            show_thinking,
            start_time,
        }
    }
}

fn aspiration_window(position: &mut Board, s: &mut Searcher, id: &mut IterDeepData) -> i32 {
    loop {
        let eval = s.negamax(
            position,
            std::cmp::max(id.depth, 1),
            id.alpha,
            id.beta,
            false,
        );

        if s.timer.stopped {
            if s.moves_fully_searched > 0 {
                id.pv = s.pv;
                id.pv_length = s.pv_length;
            }
            return 0;
            //this return value will not actually be used
        }

        id.pv = s.pv;
        id.pv_length = s.pv_length;
        id.eval = eval;

        s.moves_fully_searched = 0;

        if eval <= id.alpha {
            //fail low -> widen window down, do not update pv
            id.alpha = std::cmp::max(id.alpha - id.delta, -INFINITY);
            id.beta = (id.alpha + id.beta) / 2;
            id.delta += id.delta / 2;
        } else if eval >= id.beta {
            //fail high -> widen window up
            id.beta = std::cmp::min(id.beta + id.delta, INFINITY);
            id.delta += id.delta / 2;
        } else {
            //within window -> just update pv and set up for next iteration

            id.delta = ASPIRATION_WINDOW;

            id.alpha = eval - id.delta;
            id.beta = eval + id.delta;

            if id.show_thinking {
                print_thinking(id.depth, eval, &s, id.start_time);
            }

            return eval;
        }
    }
}

pub fn best_move(
    position: &mut Board,
    time_left: usize,
    inc: usize,
    moves_to_go: usize,
    movetime: usize,
    s: &mut Searcher,
    show_thinking: bool,
) -> MoveData {
    let start = Instant::now();
    let move_duration = match movetime {
        0 => Duration::from_millis(
            move_time(time_left, inc, moves_to_go, position.ply)
                .try_into()
                .unwrap(),
        ),
        k => {
            if k < MOVE_OVERHEAD {
                Duration::from_millis(k as u64)
            } else {
                let t = cmp::max(MIN_MOVE_TIME, k - MOVE_OVERHEAD);
                Duration::from_millis(t as u64)
            }
        }
    };

    let end_time = start + move_duration;
    //calculate time to cancel search

    s.reset_searcher();
    s.timer.end_time = end_time;

    let mut id = IterDeepData::new(start, show_thinking);
    let mut pv = String::new();

    while id.depth < MAX_SEARCH_DEPTH {
        let eval = aspiration_window(position, s, &mut id);

        if s.timer.stopped {
            break;
        }

        id.eval = eval;
        id.depth += 1;
    }

    for m in id.pv[0].iter().take(id.pv_length[0]) {
        pv += m.uci().as_str();
        pv += " ";
    }

    MoveData {
        m: id.pv[0][0],
        nodes: s.nodes,
        eval: id.eval,
        pv,
    }
}
