use crate::board::*;
use crate::eval::*;
use crate::helper::*;
use crate::magic::*;
use crate::movegen::*;
use crate::ordering::*;
use crate::r#move::*;
use crate::thread::*;
use crate::transposition::*;
use crate::types::*;
use crate::uci::*;
use crate::zobrist::{BLACK_TO_MOVE, EP_KEYS};

use std::cmp;
#[allow(unused_imports)]
use std::collections::HashMap;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

pub const INFINITY: i32 = 1_000_000_000;
pub const MAX_PLY: usize = 64;
pub const REDUCTION_LIMIT: u8 = 2;

const FULL_DEPTH_MOVES: u8 = 1;

const SINGULARITY_DE_MARGIN: i32 = 40;

#[allow(dead_code)]
const NULLMOVE_MAX_DEPTH: u8 = 6;
#[allow(dead_code)]
const NULLMOVE_MIN_DEPTH: u8 = 3;

const ASPIRATION_WINDOW: i32 = 40;

const RAZORING_MARGIN: i32 = 300;
const MAX_RAZOR_DEPTH: u8 = 4;

const BETA_PRUNING_DEPTH: u8 = 6;
const BETA_PRUNING_MARGIN: u8 = 80;

const ALPHA_PRUNING_DEPTH: u8 = 4;
const ALPHA_PRUNING_MARGIN: i32 = 2000;

const SEE_PRUNING_DEPTH: i32 = 4;
const SEE_QUIET_MARGIN: i32 = 100;
const SEE_NOISY_MARGIN: i32 = 70;
const SEE_QSEARCH_MARGIN: i32 = 1;

#[allow(unused)]
const LMP_DEPTH: u8 = 5;

const IIR_DEPTH_MINIMUM: u8 = 6;
const DO_SINGULARITY_EXTENSION: bool = true;
const DO_SINGULARITY_DE: bool = false;

pub const MAX_GAME_PLY: usize = 1024;

#[allow(unused)]
const TIME_TO_START_SEARCH: usize = 0; //initialise big TT (if not using HashMap)
                                       //leave 100ms total margin

struct NullMoveUndo {
    ep: Option<Square>,
    pinned: BitBoard,
    hash_key: u64,
}

fn reduction_ok(tactical: bool, in_check: bool) -> bool {
    !(tactical || in_check)
}

//make null move for NMP
//we have to update pinners but not checkers since NMP is never done while in check
fn make_null_move(b: &mut Board) -> NullMoveUndo {
    let hash_reset = b.hash_key;
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

    b.hash_key ^= BLACK_TO_MOVE;

    if let Some(reset) = b.en_passant {
        b.hash_key ^= EP_KEYS[reset];
        b.en_passant = None;
        return NullMoveUndo {
            ep: Some(reset),
            pinned: pinned_reset,
            hash_key: hash_reset,
        };
    }

    NullMoveUndo {
        ep: None,
        pinned: pinned_reset,
        hash_key: hash_reset,
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
    b.hash_key = undo.hash_key;
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
    if count(b.bitboards[Piece::WB]) >= 2
        || count(b.bitboards[Piece::BB]) >= 2
        || count(b.bitboards[Piece::WB]) >= 1 && count(b.bitboards[Piece::WN]) >= 1
        || count(b.bitboards[Piece::BB]) >= 1 && count(b.bitboards[Piece::BN]) >= 1
    {
        return false;
    }
    count(b.bitboards[Piece::WN]) <= 2 && count(b.bitboards[Piece::BN]) <= 2
    //can technically arise a position where KvKNN is mate so this
    //could cause some bug in theory lol
}

fn is_drawn(position: &Board) -> bool {
    if position.fifty_move == 100 {
        return true;
    }

    for key in position.history.iter().take(position.ply - 1) {
        //take ply - 1 because the start position (with 0 ply) is included
        if *key == position.hash_key {
            return true;
            //return true on one repetition because otherwise the third
            //repetition will not be reached because the search will stop
            //after a tt hit on the second repetition
        }
    }

    is_insufficient_material(position)
}

impl Thread<'_> {
    fn should_check_exit(&self) -> bool {
        const CHECK_INTERVAL: usize = 4095;
        self.nodes & CHECK_INTERVAL == 0
    }

    fn should_exit(&mut self) -> bool {
        if self.stop.load(Relaxed) {
            return true;
        } else if self.should_check_exit() {
            let done = Instant::now() > self.timer.end_time || self.nodes >= self.timer.max_nodes;
            self.stop.store(done, Relaxed);
            return done;
        }
        false
    }

    fn is_stopped(&self) -> bool {
        self.stop.load(Relaxed)
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
        depth: u8,
        pv_node: bool,
        _alpha: i32,
        beta: i32,
        cutnode: bool,
    ) -> Option<i32> {
        position.undo_move(best_move, commit);
        self.ply -= 1;
        //undo move already made on board
        let threshold = std::cmp::max(tt_score - (depth as i32 * 2 + 20), -INFINITY);

        self.info.excluded[self.ply] = Some(best_move);

        let excluded_eval = self.negamax(position, depth / 2, threshold - 1, threshold, cutnode);

        self.info.excluded[self.ply] = None;

        if DO_SINGULARITY_DE && !pv_node && excluded_eval < threshold - SINGULARITY_DE_MARGIN {
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
        mut depth: u8,
        mut alpha: i32,
        beta: i32,
        cutnode: bool,
    ) -> i32 {
        if self.should_exit() {
            return 0;
        } else if self.ply == MAX_PLY - 1 {
            return evaluate(position);
        }
        let pv_node = beta - alpha != 1;
        let root = self.ply == 0;
        //full window search

        self.pv_length[self.ply] = self.ply;

        if depth == 0 {
            //qsearch on leaf nodes
            return self.qsearch(position, alpha, beta);
        }

        let mut hash_flag = EntryFlag::UpperBound;
        self.nodes += 1;

        //NOTE: tt_score is only used in singular search, in which case we know that there is
        //definitely a hash result, so this value of 0 is never actually read
        let (mut tt_depth, mut tt_bound, mut tt_score) = (0, EntryFlag::Missing, 0);
        let mut best_move = NULL_MOVE; //used for TT hash -> move ordering
                                       //this is useful in cases where it cannot return the eval of the hash lookup
                                       //due to the bounds, but it can use the best_move field for move ordering

        //don't probe TT in singular search
        if !root && self.info.excluded[self.ply].is_none() {
            //check 50 move rule, repetition and insufficient material
            if is_drawn(position) {
                return 0;
            }

            // mate distance pruning:
            // check if line is so good/bad that being mated in the current ply
            // or mating in the next ply would not change alpha/beta
            let r_alpha = cmp::max(alpha, -INFINITY + self.ply as i32);
            let r_beta = cmp::min(beta, INFINITY - self.ply as i32 - 1);
            if r_alpha >= r_beta {
                return r_alpha;
            }

            if let Some(entry) = self.tt.lookup(position.hash_key) {
                best_move = entry.best_move;
                tt_score = entry.eval;
                tt_depth = entry.depth;
                tt_bound = entry.flag;

                if !pv_node
                    && depth <= entry.depth
                    && match entry.flag {
                        EntryFlag::Exact => true,
                        EntryFlag::LowerBound => entry.eval >= beta,
                        EntryFlag::UpperBound => entry.eval <= alpha,
                        EntryFlag::Missing => false,
                    }
                {
                    return entry.eval.clamp(alpha, beta);
                }
            }
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
        if !in_check && self.info.excluded[self.ply].is_none() && self.do_pruning {
            let static_eval = evaluate(position);
            if self.ply < MAX_PLY {
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
                        - (BETA_PRUNING_MARGIN * cmp::max(depth - improving as u8, 0)) as i32
                        >= beta
                {
                    return static_eval;
                }

                // Alpha Pruning:
                // eval is so bad that even a huge margin fails to raise alpha
                if depth <= ALPHA_PRUNING_DEPTH && static_eval + ALPHA_PRUNING_MARGIN <= alpha {
                    return static_eval;
                }

                // Razoring:
                // eval is very low so only realistic way to increase it is with captures
                // we only need to qsearch to evaluate the position
                if depth <= MAX_RAZOR_DEPTH
                    && static_eval + RAZORING_MARGIN * (depth as i32) <= alpha
                {
                    let score = self.qsearch(position, alpha, beta);
                    if score > alpha {
                        return score;
                    }
                }

                // Null move pruning:
                // If we are still able to reach an eval >= beta if we give our opponent
                // another move, then their previous move was probably bad
                if !position.is_kp_endgame()
                    && !position.last_move_null
                    && static_eval >= beta + 200 - 20 * (depth as i32)
                    && !root
                {
                    let undo = make_null_move(position);
                    self.ply += 1;
                    let r = 2 + depth as i32 / 4 + cmp::min((static_eval - beta) / 256, 3);
                    let reduced_depth = cmp::max(depth as i32 - r, 1) as u8;
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

        // IIR: if we don't have a TT hit then move ordering here will be terrible
        // so its better to reduce and set up TT move for next iteration
        if (pv_node || cutnode) && depth >= IIR_DEPTH_MINIMUM && !tt_move {
            depth -= 1;
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

        #[allow(unused)]
        for (i, &m) in move_list.moves.iter().enumerate() {
            if m.is_null() {
                //no pseudolegal moves left in move list
                break;
            } else if let Some(n) = self.info.excluded[self.ply] {
                if n == m {
                    moves_seen += 1;
                    continue;
                }
            }

            //from what I can see strong engines update this before checking whether or not the
            //move is legal
            moves_seen += 1;

            let tactical = m.is_tactical(position);
            let quiet = !tactical;
            let not_mated = alpha > -INFINITY + MAX_PLY as i32;
            //must be done before making the move on the board

            let is_killer = m == self.info.killer_moves[0][self.ply]
                || m == self.info.killer_moves[1][self.ply];

            //Early Pruning: try to prune moves before we search them properly
            //by showing that they're not worth investigating
            if !root && not_mated {
                if quiet && skip_quiets && !is_killer {
                    continue;
                }
                let r: i32 = self.info.lmr_table.reduction_table[quiet as usize]
                    [cmp::min(depth, 31) as usize][cmp::min(moves_seen, 31) as usize]
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
                    if !m.see(position, threshold) {
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
                && self.info.excluded[self.ply].is_none()
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

            let new_depth =
                i32::clamp(depth as i32 - 1 + extension.unwrap(), 0, MAX_PLY as i32) as u8;

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
                    [cmp::min(depth, 31) as usize][cmp::min(moves_played, 31) as usize];

                let mut reduction_eval = if moves_played
                    > (FULL_DEPTH_MOVES + pv_node as u8 + !tt_move as u8 + root as u8)
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

                    let mut reduced_depth = cmp::max(new_depth as i32 - r, 1) as u8;
                    reduced_depth = reduced_depth.clamp(1, new_depth);
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

            if self.is_stopped() {
                return 0;
            }

            if root {
                self.moves_fully_searched += 1;
                //used to ensure in the iterative deepening search that
                //at least one move has been searched fully
            }

            if eval > alpha {
                alpha = eval;
                self.update_pv(m);
                hash_flag = EntryFlag::Exact;
                best_move = m;
            }

            //search failed high
            if eval >= beta {
                //only write quiet moves into history table because captures
                //will be scored separately
                self.update_search_tables(position, &move_list, m, tactical, depth, moves_played);
                hash_flag = EntryFlag::LowerBound;
                break;
            }
        }

        if moves_played == 0 {
            //no legal moves -> mate or stalemate
            return match in_check {
                true => -INFINITY + self.ply as i32,
                false => 0,
            };
        }

        if !self.is_stopped() {
            let hash_entry = TTEntry::new(depth, alpha, hash_flag, best_move, position.hash_key);

            self.tt.write(position.hash_key, hash_entry);
        }

        alpha
    }

    pub fn qsearch(&mut self, position: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;

        if is_drawn(position) {
            return 0;
        }

        if self.should_exit() {
            return 0;
        } else if self.ply == MAX_PLY - 1 {
            return evaluate(position);
        }

        let mut hash_flag = EntryFlag::UpperBound;

        let mut best_move = NULL_MOVE;

        if let Some(entry) = self.tt.lookup(position.hash_key) {
            best_move = entry.best_move;
            if match entry.flag {
                EntryFlag::Exact => true,
                EntryFlag::LowerBound => entry.eval >= beta,
                EntryFlag::UpperBound => entry.eval <= alpha,
                EntryFlag::Missing => false,
            } {
                return entry.eval.clamp(alpha, beta);
            }
        }

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

        for c in captures.moves {
            if c.is_null() {
                //no more pseudo-legal moves
                break;
            }

            let worst_case = SEE_VALUES[piece_type(position.get_piece_at(c.square_to()))]
                - SEE_VALUES[piece_type(c.piece_moved(position))];

            if eval + worst_case > beta {
                //prune in the case that our move > beta even if we lose the piece
                //that we just moved
                return beta;
            }

            if !c.see(position, SEE_QSEARCH_MARGIN) {
                //prune moves that fail see by threshold
                continue;
            }

            if eval + 200 <= alpha
                && !c.see(
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

            let eval = -self.qsearch(position, -beta, -alpha);
            position.undo_move(c, &commit);
            self.ply -= 1;

            if self.is_stopped() {
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
        if !self.is_stopped() {
            let hash_entry = TTEntry::new(0, alpha, hash_flag, best_move, position.hash_key);

            self.tt.write(position.hash_key, hash_entry);
        }
        alpha
    }

    pub fn update_pv(&mut self, m: Move) {
        let next_ply = self.ply + 1;
        self.pv[self.ply][self.ply] = m;
        for i in next_ply..self.pv_length[next_ply] {
            self.pv[self.ply][i] = self.pv[next_ply][i];
            //copy from next row in pv table
        }
        self.pv_length[self.ply] = self.pv_length[next_ply];
    }

    pub fn update_search_tables(
        &mut self,
        b: &Board,
        moves: &MoveList,
        cutoff_move: Move,
        tactical: bool,
        depth: u8,
        moves_played: u8,
    ) {
        if !tactical {
            self.update_killer_moves(cutoff_move, tactical);
            self.update_history_table(b, moves, cutoff_move, depth, moves_played as usize)
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
        depth: u8,
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

    pub fn reset_thread(&mut self) {
        //try to keep tt, history and killer moves
        self.nodes = 0;
        self.pv_length = [0; 64];
        self.pv = [[NULL_MOVE; MAX_PLY]; MAX_PLY];
        self.ply = 0;
        self.moves_fully_searched = 0;

        //reset move ordering heuristics
        self.info.killer_moves = [[NULL_MOVE; MAX_PLY]; 2];
        self.info.history_table = [[0; 64]; 12];
    }
}

pub struct MoveData {
    pub m: Move,
    pub nodes: usize,
    pub eval: i32,
    pub pv: String,
}

struct IterDeepData {
    eval: i32,
    pv: [[Move; MAX_PLY]; MAX_PLY],
    pv_length: [usize; MAX_PLY],

    delta: i32,
    alpha: i32,
    beta: i32,
    depth: u8,

    show_thinking: bool,
    start_time: Instant,
    soft_limit: Instant,
}

impl IterDeepData {
    fn new(start_time: Instant, show_thinking: bool, soft_limit: Instant) -> Self {
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
            soft_limit,
        }
    }
}

fn aspiration_window(position: &mut Board, s: &mut Thread, id: &mut IterDeepData) -> i32 {
    loop {
        let eval = s.negamax(
            position,
            std::cmp::max(id.depth, 1),
            id.alpha,
            id.beta,
            false,
        );

        if s.is_stopped() {
            if s.moves_fully_searched > 0 {
                id.pv = s.pv;
                id.pv_length = s.pv_length;
            }
            return 0;
            //this return value will not actually be used
        }

        s.moves_fully_searched = 0;

        if eval > id.alpha && eval < id.beta {
            //within window -> just update pv and set up for next iteration
            //note atm this must be a strict inequality since search is failing hard

            id.pv = s.pv;
            id.pv_length = s.pv_length;

            id.delta = ASPIRATION_WINDOW;

            id.alpha = eval - id.delta;
            id.beta = eval + id.delta;

            if id.show_thinking {
                print_thinking(id.depth, eval, s, id.start_time);
            }

            return eval;
        }

        if eval <= id.alpha {
            //failed low -> widen window down, do not update pv
            id.alpha = std::cmp::max(id.alpha - id.delta, -INFINITY);
            id.beta = (id.alpha + id.beta) / 2;
            id.delta += id.delta / 2;
        } else if eval >= id.beta {
            //failed high -> widen window up, also update pv
            id.beta = std::cmp::min(id.beta + id.delta, INFINITY);
            id.delta += id.delta / 2;

            id.pv = s.pv;
            id.pv_length = s.pv_length;
        }
    }
}

pub fn iterative_deepening(
    position: &mut Board,
    soft_limit: usize,
    hard_limit: usize,
    s: &mut Thread,
    show_thinking: bool,
) -> MoveData {
    let start = Instant::now();

    //TODO: no aspiration window for first few depths

    s.reset_thread();
    s.timer.end_time = start + Duration::from_millis(hard_limit as u64);

    let soft_limit = Instant::now() + Duration::from_millis(soft_limit as u64);
    let mut id = IterDeepData::new(start, show_thinking, soft_limit);

    while (id.depth as usize) < MAX_PLY {
        let eval = aspiration_window(position, s, &mut id);

        if s.is_stopped() {
            break;
        }

        id.eval = eval;
        id.depth += 1;

        if Instant::now() > id.soft_limit {
            //not the same as above break statement because eval was updated
            //which won't affect choice of move but will affect data we report
            break;
        }
    }

    let pv = id.pv[0]
        .iter()
        .take(id.pv_length[0])
        .fold(String::new(), |acc, m| acc + (m.uci() + " ").as_str());

    MoveData {
        m: id.pv[0][0],
        nodes: s.nodes,
        eval: id.eval,
        pv,
    }
}
