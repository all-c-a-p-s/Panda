#![cfg_attr(feature = "datagen", allow(dead_code, unused))]

pub(crate) mod macros;
pub mod ordering;
pub mod tables;
pub mod thread;
pub mod transposition;

pub use ordering::*;
pub use thread::*;
pub use transposition::*;

use arrayvec::ArrayVec;

use crate::board::Board;
use crate::board::r#move::{Commit, Move, MoveList, NULL_MOVE};
use crate::eval::evaluate;
use crate::search::macros::*;
use crate::util::helper::{read_param, tuneable_params};
use crate::util::types::PieceType;
use crate::util::uci::print_thinking;

use core::option::Option::Some;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

pub const INFINITY: i32 = 1_000_000_000;
pub const MAX_DEPTH: usize = 64;
pub const MATE: i32 = INFINITY - MAX_DEPTH as i32;

pub const REDUCTION_LIMIT: u8 = 2;

const FULL_DEPTH_MOVES: u8 = 1;

// name, type, val, min, max

tuneable_params! {
    // params for search conditions
    SINGULARITY_DE_MARGIN, i32, 25, 10, 150;
    ASPIRATION_WINDOW, i32, 17, 10, 70;
    RAZORING_MARGIN, i32, 273, 100, 500;
    MAX_RAZOR_DEPTH, u8, 4, 1, 12;
    RFP_DEPTH, u8, 6, 1, 12;
    RFP_MARGIN, u8, 26, 20, 200;
    TT_FUTILITY_MARGIN, i32, 154, 40, 400;
    HISTORY_PRUNING_DEPTH, i32, 3, 1, 12;
    SEE_PRUNING_DEPTH, i32, 8, 1, 12;
    SEE_QUIET_MARGIN, i32, 47, 20, 300;
    SEE_NOISY_MARGIN, i32, 41, 20, 250;
    SEE_QSEARCH_MARGIN, i32, 25, 1, 100;
    QSEARCH_FP_MARGIN, i32, 180, 1, 350;
    LMP_DEPTH, u8, 4, 1, 12;
    IIR_DEPTH_MINIMUM, u8, 9, 1, 12;

    // move ordering scores
    HASH_MOVE_SCORE, i32, 1_000_000, 1_000_000, 1_000_000;
    QUEEN_PROMOTION, i32, 750_000, -999_999, 999_999;
    WINNING_CAPTURE, i32, 500_000, -999_999, 999_999;
    FIRST_KILLER_MOVE, i32, 100_000, -999_999, 999_999;
    LOSING_CAPTURE, i32, -300_000, -999_999, 999_999;
    UNDER_PROMOTION, i32, -500_000, -999_999, 999_999;

    // factors affecting reductions etc
    NMP_FACTOR, i32, 22, 1, 100;
    NMP_BASE, i32, 200, 50, 500;
    HISTORY_NODE_DIVISOR, usize, 655, 256, 8192;
    HISTORY_MIN_THRESHOLD, i32, 7870, 1024, 32768;
    LMR_TACTICAL_BASE, i32, 38, 0, 500;
    LMR_TACTICAL_DIVISOR, i32, 319, 100, 500;
    LMR_QUIET_BASE, i32, 144, 0, 500;
    LMR_QUIET_DIVISOR, i32, 285, 100, 500;

    // LERP weights
    RFP_BETA_WEIGHT, i32, 22, 0, 1024;
    NMP_BETA_WEIGHT, i32, 327, 0, 1024;
    STAND_PAT_BETA_WEIGHT, i32, 170, 0, 1024;
}

const DO_SINGULARITY_EXTENSION: bool = true;
const DO_SINGULARITY_DE: bool = true;

pub const REPETITION_TABLE_SIZE: usize = 100 + 1;

fn lerp(u: i32, v: i32, w1: i32) -> i32 {
    ((u as i64 * w1 as i64 + v as i64 * (1024 - w1) as i64) / 1024) as i32
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

    /// Here we try to prove that a move is better than alternatives by a significant margin.
    /// If this is true, we should extend it since it is more important. This function determines
    /// how much we should extend by.
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
        position.undo_move(best_move, commit, Some(&mut self.info.stck));
        self.ply -= 1;
        // undo move already made on board
        let threshold = (tt_score - (depth as i32 * 2 + 20)).max(-INFINITY);

        self.info.excluded[self.ply] = Some(best_move);

        let excluded_eval = self.negamax(position, depth / 2, threshold - 1, threshold, cutnode);

        self.info.excluded[self.ply] = None;

        if singularity_de!(self, pv_node, excluded_eval, threshold) {
            Some(2)
        } else if excluded_eval < threshold {
            Some(1)
        } else if threshold >= beta {
            // MultiCut: more than one move will be able to beat beta
            // here we return None to indicate that the search should terminate
            // and return beta
            None
        } else if tt_score >= beta {
            Some(-1)
        } else {
            Some(0)
        }
    }

    /// Negamax:
    /// Alpha-Beta search with various enhancements.
    /// Node Types:
    /// - PV-node: a node in which the value returned is between alpha and beta (exact)
    /// - Cutnode: a node in which a beta cutoff occurred, value returned >= beta (lower bound)
    /// - All-node: a node in which all moves were searched and value returned <= alpha (upper bound)
    ///   If we can predict the type of a node, we can make better decisions about pruning.
    pub fn negamax(&mut self, position: &mut Board, mut depth: u8, mut alpha: i32, beta: i32, cutnode: bool) -> i32 {
        if self.should_exit() {
            return 0;
        }

        self.seldepth = self.seldepth.max(self.ply as u8);
        self.nodes += 1;

        if self.ply == MAX_DEPTH - 1 {
            return evaluate(position, &self.info.stck.current());
        }

        let pv_node = beta - alpha != 1;
        let root = self.ply == 0;
        let singular = self.info.excluded[self.ply].is_some();

        self.pv_length[self.ply] = self.ply;

        if depth == 0 {
            return self.qsearch(position, alpha, beta);
        }

        let mut hash_flag = EntryFlag::UpperBound;

        let (mut tt_depth, mut tt_bound, mut tt_score, mut tt_hit) = (0, EntryFlag::Missing, 0, false);
        let mut best_move = NULL_MOVE;

        let in_check = position.checkers != 0;

        if !root && !singular {
            if position.is_drawn() {
                return if self.ply.is_multiple_of(2) { 1 } else { -1 };
            }

            // Mate Distance Pruning:
            // Check if line is so good/bad that being mated in the current ply
            // or mating in the next ply would not change alpha/beta
            let r_alpha = alpha.max(-INFINITY + self.ply as i32);
            let r_beta = beta.min(INFINITY - self.ply as i32 - 1);
            if r_alpha >= r_beta {
                return r_alpha;
            }
        }

        if let Some(entry) = self.tt.lookup(position.hash_key) {
            best_move = entry.best_move;
            tt_hit = true;
            tt_score = entry.eval;
            tt_depth = entry.depth;
            tt_bound = entry.flag;

            // We accept values from the TT if:
            //      (1) the depth of the entry >= our depth, with the correct bound
            // OR   (2) we are in an expected cutnode, and the eval is well above beta
            if tt_cutoff!(singular, root, pv_node, depth, entry, beta, alpha, cutnode, in_check) {
                return entry.eval;
            }
        }

        let tt_move_exists = !best_move.is_null();
        let tt_move_capture = if tt_move_exists { best_move.is_capture(position) } else { false };

        // reset killers for child nodes
        self.info.killer_moves[self.ply + 1] = None;

        let mut static_eval = evaluate(position, &self.info.stck.current());
        if !singular {
            let corrected = self.eval_with_corrhist(position, static_eval);
            static_eval = corrected;
        }

        let mut tt_correction = 0;
        if should_correct_with_tt!(tt_hit, static_eval, tt_score, tt_bound) {
            tt_correction = (tt_score - static_eval).abs();
            static_eval = tt_score;
        }

        if self.ply < MAX_DEPTH {
            self.info.ss[self.ply] =
                SearchStackEntry { eval: static_eval, square_moved_to: None, piece_moved: None, made_capture: false };
        }

        // Improving heuristic:
        // useful for considering whether or not to prune in the search:
        // - if improving then we should expect to fail high more
        // - and to fail low less
        // same goes for opponent_worsening
        let improving = match self.ply {
            2.. => self.info.ss[self.ply].eval > self.info.ss[self.ply - 2].eval,
            _ => false,
        } && !in_check;

        let opponent_worsening = match self.ply {
            3.. => self.info.ss[self.ply - 1].eval < self.info.ss[self.ply - 3].eval,
            _ => false,
        };

        let opponent_captured = self.ply > 0 && self.info.ss[self.ply - 1].made_capture;

        // Static pruning: here we attempt to show that the position does not require any further
        // search
        if can_static_prune!(self, in_check, singular, pv_node) {
            // Reverse Futility Pruning:
            // If eval >= beta + some margin, assume that we can achieve at least beta
            if can_rfp!(depth, static_eval, improving, beta) {
                return lerp(beta, static_eval, read_param!(RFP_BETA_WEIGHT));
            }

            // Razoring:
            // If our opponent just captured and the static eval is far below alpha, it's likely
            // that only captures can raise alpha. Hence, we just run a qsearch.
            if can_razor!(depth, static_eval, improving, opponent_captured, opponent_worsening, alpha) {
                let qeval = self.qsearch(position, alpha, beta);
                if qeval < alpha {
                    return qeval;
                }
            }

            // Null Move Pruning (NMP):
            // If we are still able to reach an eval >= beta if we give our opponent
            // another move, then their previous move was probably bad
            if can_nmp!(position, static_eval, depth, beta, root) {
                let undo = position.make_null_move();
                self.ply += 1;
                let r = 2
                    + depth as i32 / 4
                    + ((static_eval - beta) / 256).min(3)
                    + improving as i32
                    + opponent_worsening as i32;
                let reduced_depth = (depth as i32 - r).max(1) as u8;
                let null_move_eval = -self.negamax(position, reduced_depth, -beta, -beta + 1, !cutnode);
                // null window used because all that matters is whether the search result is better than beta
                position.undo_null_move(&undo);
                self.ply -= 1;
                if null_move_eval >= beta {
                    return lerp(beta, null_move_eval, read_param!(NMP_BETA_WEIGHT));
                }
            }
        }

        // Internal Iterative Reduction (IIR):
        // if we don't have a TT hit then move ordering here will be terrible
        // so its better to reduce and set up TT move for next iteration
        if do_iir!(pv_node, cutnode, depth, tt_move_exists) {
            depth -= 1;
        }

        let (mut quiets, mut caps) = (ArrayVec::<Move, 64>::new(), ArrayVec::<Move, 64>::new());

        let mut movelist = MoveList::empty();
        let mut movepicker = MovePicker::new();

        let (mut good_caps, mut bad_caps) = (MoveList::empty(), MoveList::empty());

        let (mut played, mut considered) = (0, 0);
        let mut best_score = -INFINITY;

        let (mut done_killer, mut done_counter) = (false, false);

        let counter = if self.ply > 0
            && let Some(pc) = self.info.ss[self.ply - 1].piece_moved
            && let Some(sq) = self.info.ss[self.ply - 1].square_moved_to
        {
            self.info.counter_moves[pc][sq]
        } else {
            None
        };

        while let Some(m) = movepicker.get_next(
            best_move,
            self.info.killer_moves[self.ply],
            counter,
            position,
            &mut movelist,
            &mut good_caps,
            &mut bad_caps,
            self,
            depth,
            pv_node,
            cutnode,
        ) {
            if m == best_move && considered > 0 {
                // hash move being generated in a later stage, but we've considered it already
                continue;
            }

            considered += 1;

            if let Some(n) = self.info.excluded[self.ply]
                && n == m
            {
                continue;
            }

            let tactical = m.is_tactical(position);
            let quiet = !tactical;
            let not_mated = best_score > -MATE;

            let is_killer = self.info.killer_moves[self.ply] == Some(m);
            let is_counter = Some(m) == counter;

            if is_killer && done_killer || is_counter && done_counter {
                // killer/counter generated in later stage by movepicker
                continue;
            }

            done_killer |= is_killer;
            done_counter |= is_counter;

            if !position.is_legal(m) {
                continue;
            }

            let piece_moved = m.piece_moved(position);

            // Early Pruning: try to prune moves before we search them properly
            // by showing that they're not worth investigating
            #[cfg(not(feature = "datagen"))]
            if !root && not_mated {
                // Late Move Pruning (LMP):
                // after a certain point start skipping all quiets after the current
                // move. The threshold I'm currently using comes from Weiss
                let d_sq = depth.min(15) * depth.min(15);
                // avoid overflow...
                let lmp_threshold = if improving { 2 + d_sq } else { d_sq / 2 };
                if do_lmp!(depth, played, lmp_threshold, in_check) {
                    movepicker.skip_quiets(&movelist);
                }

                let r = self.info.lmr_table.reduction_table[quiet as usize][depth.min(31) as usize]
                    [considered.min(31) as usize]
                    + !improving as i32;
                let lmr_depth = depth as i32 - 1 - r.max(0);

                // SEE Pruning:
                // skip moves that fail SEE by a depth-dependent threshold
                if do_see_pruning!(lmr_depth, considered, pv_node) {
                    let margin = if tactical { read_param!(SEE_NOISY_MARGIN) } else { read_param!(SEE_QUIET_MARGIN) };
                    let threshold = margin * depth as i32;
                    if !m.see(position, threshold) {
                        continue;
                    }
                }
            }

            // checked to be legal above
            let commit = position.play_unchecked(m, Some(&mut self.info.stck));

            if self.ply < MAX_DEPTH {
                self.info.ss[self.ply].square_moved_to = Some(m.square_to());
                self.info.ss[self.ply].piece_moved = Some(piece_moved);

                if tactical {
                    self.info.ss[self.ply].made_capture = true;
                }
            }

            let nodes_before = self.nodes;

            played += 1;
            self.ply += 1;
            // update after pruning above

            // A singular move is a move which seems to be forced or at least much stronger than
            // others. We should therefore extend to investigate it further.
            let maybe_singular = maybe_singular!(root, depth, singular, m, best_move, tt_depth, tt_bound);

            let ext = if maybe_singular {
                self.singularity(position, best_move, &commit, tt_score, depth, pv_node, alpha, beta, cutnode)
            } else {
                Some((in_check && !root) as i32)
            };

            let Some(extension) = ext else {
                // MultiCut case from singularity function
                return tt_score - (depth as i32 * 2);
            };

            if maybe_singular {
                position.play_unchecked(best_move, Some(&mut self.info.stck));
                self.ply += 1;
                // we unmade the move while calling the singularity function
            }

            if extension == 2 {
                self.double_extensions += 1;
            }

            let new_depth = (depth as i32 - 1 + extension).clamp(0, MAX_DEPTH as i32) as u8;

            let eval = if played == 1 {
                // Internal Aspiration Window:
                // Assume the value of our lower-depth search has some merit, so we may be able to search on
                // a tighter window around this value.
                if do_iaw!(pv_node, tt_hit, tt_bound, root, singular, tt_score, alpha, beta) {
                    let depth_diff = (depth as i32 - tt_depth as i32).abs().max(1);
                    let mut delta = (tt_correction / 2).clamp(10, 25) * depth_diff;

                    let mut fails = 0;

                    let mut w_alpha = (tt_score - delta).max(alpha);
                    let mut w_beta = (tt_score + delta).min(beta);
                    loop {
                        if (w_alpha == alpha && w_beta == beta) || w_beta - w_alpha == 1 {
                            break -self.negamax(position, new_depth, -w_beta, -w_alpha, false);
                        }

                        let w_eval = -self.negamax(position, new_depth, -w_beta, -w_alpha, false);

                        if w_eval > w_alpha && w_eval < w_beta {
                            break w_eval;
                        }

                        fails += 1;
                        delta *= 2;

                        // if we fail outside the window then we get a good bound for the min/max
                        // score we can achieve

                        if w_eval <= w_alpha {
                            if w_eval <= alpha {
                                break w_eval;
                            }
                            w_beta = (w_alpha.max(alpha) + w_beta) / 2;
                            w_alpha = (w_alpha - delta).max(alpha);
                        } else {
                            if w_eval >= beta {
                                break w_eval;
                            }
                            w_beta = (w_beta + delta).min(beta);
                        }

                        if fails >= 2 {
                            break -self.negamax(position, new_depth, -beta, -alpha, false);
                        }
                    }
                } else {
                    -self.negamax(position, new_depth, -beta, -alpha, false)
                }
            } else {
                // Principle Variation Search (PVS):
                // Assume that our move ordering is good enough that
                // we will be able to prove relatively inexpensively that late
                // moves aren't worth investigating.

                let mut r_eval = -INFINITY;
                let do_full_depth_zw =
                    if should_reduce!(played, pv_node, tt_move_exists, root, tactical, depth, not_mated) {
                        let mut r = 1;
                        // fixed reduction of 1 for captures seems to work well
                        if quiet {
                            r = self.info.lmr_table.reduction_table[quiet as usize][depth.min(31) as usize]
                                [played.min(31) as usize];

                            // reduce more when we have reason to expect little from this move
                            r += tt_move_capture as i32;
                            r += !improving as i32;

                            // reduce less when this move is important/promising
                            r -= pv_node as i32;
                            r -= in_check as i32;
                            r -= (is_killer || is_counter) as i32;

                            // either increase or decrease reduction depending on history score
                            r -= self.info.history_table[piece_moved][m.square_to()] / 8192;
                        }

                        let reduced_depth = (new_depth as i32 - r).clamp(1, new_depth as i32) as u8;
                        // avoid dropping into qsearch or extending

                        r_eval = -self.negamax(position, reduced_depth, -alpha - 1, -alpha, true);
                        r_eval > alpha && reduced_depth < new_depth
                    } else {
                        true
                    };

                if do_full_depth_zw {
                    // failed to prove that move is bad -> re-search with same depth but still zw
                    r_eval = -self.negamax(position, new_depth, -alpha - 1, -alpha, !cutnode);
                }

                if pv_node && r_eval > alpha {
                    // move actually inside PV window -> search at full depth
                    r_eval = -self.negamax(position, new_depth, -beta, -alpha, false);
                }
                r_eval
            };

            position.undo_move(m, &commit, Some(&mut self.info.stck));
            self.ply -= 1;

            if extension == 2 {
                self.double_extensions -= 1;
            }

            if self.is_stopped() {
                return 0;
            }

            if root {
                self.info.nodetable.add(m, self.nodes - nodes_before);
                self.moves_fully_searched += 1;
            }

            if quiet && !quiets.is_full() {
                quiets.push(m);
            } else if !caps.is_full() {
                caps.push(m);
            }

            best_score = best_score.max(eval);

            if eval > alpha {
                alpha = eval;
                self.update_pv(m);
                hash_flag = EntryFlag::Exact;
                best_move = m;

                if eval >= beta {
                    self.update_search_tables(position, &quiets, &caps, m, tactical, depth);
                    hash_flag = EntryFlag::LowerBound;
                    break;
                }
            }
        }

        if played == 0 {
            return (-INFINITY + self.ply as i32) * in_check as i32;
        }

        if !self.is_stopped() && !singular {
            let hash_entry = TTEntry::new(depth, best_score, hash_flag, best_move, position.hash_key);

            self.tt.write(position.hash_key, hash_entry);

            if corrhist_update_allowed!(in_check, best_move, position, hash_flag, best_score, static_eval) {
                self.update_corrhist(position, depth, best_score - static_eval);
            }
        }

        best_score
    }

    /// Quiescence Search:
    /// Search all noisy moves, or find an evasion if in check.
    /// This is done to prevent the horizon effect.
    pub fn qsearch(&mut self, position: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        self.seldepth = self.seldepth.max(self.ply as u8);

        if position.is_drawn() {
            return if self.ply.is_multiple_of(2) { 1 } else { -1 };
        }

        if self.should_exit() {
            return 0;
        }

        if self.ply == MAX_DEPTH - 1 {
            return evaluate(position, &self.info.stck.current());
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
                return entry.eval;
            }
        }

        let in_check = position.checkers != 0;

        let mut static_eval = evaluate(position, &self.info.stck.current());
        static_eval = self.eval_with_corrhist(position, static_eval);

        let mut best_score = if in_check { -INFINITY + 1 } else { static_eval };

        if best_score >= beta {
            return lerp(beta, best_score, read_param!(STAND_PAT_BETA_WEIGHT));
        }

        alpha = alpha.max(best_score);

        let mut movelist = MoveList::empty();
        let (mut good_caps, mut bad_caps) = (MoveList::empty(), MoveList::empty());

        let mut movepicker = if in_check { MovePicker::new() } else { MovePicker::for_qsearch() };

        let q_hash = if in_check || best_move.is_tactical(position) { best_move } else { NULL_MOVE };

        let mut could_be_mated = in_check;

        while let Some(m) = movepicker.get_next(
            q_hash,
            if in_check && could_be_mated { self.info.killer_moves[self.ply] } else { None },
            None,
            position,
            &mut movelist,
            &mut good_caps,
            &mut bad_caps,
            self,
            0,
            false,
            false,
        ) {
            if !position.is_legal(m) {
                continue;
            }

            if could_be_mated {
                best_score = best_score.max(static_eval);
                if best_score >= beta {
                    return lerp(beta, best_score, read_param!(STAND_PAT_BETA_WEIGHT));
                }
                movepicker.skip_quiets(&movelist);
                could_be_mated = false;
            }

            let check_futility = best_score + read_param!(QSEARCH_FP_MARGIN) <= alpha;

            //if we're far behind, only consider moves which win significant material
            if check_futility {
                if !m.see(position, SEE_VALUES[PieceType::Knight] - SEE_VALUES[PieceType::Bishop] - 1) {
                    continue;
                }
            } else if !m.see(position, read_param!(SEE_QSEARCH_MARGIN)) {
                // alternatively just skip any move which fails SEE by this margin
                // note anything that passes the futility check will pass this so there's no need
                // to do SEE check twice on such moves
                continue;
            }

            //checked to be legal above
            let commit = position.play_unchecked(m, Some(&mut self.info.stck));

            self.ply += 1;

            let eval = -self.qsearch(position, -beta, -alpha);
            position.undo_move(m, &commit, Some(&mut self.info.stck));
            self.ply -= 1;

            if self.is_stopped() {
                return 0;
            }

            best_score = best_score.max(eval);
            if eval > alpha {
                alpha = eval;
                hash_flag = EntryFlag::Exact;
                best_move = m;

                if eval >= beta {
                    hash_flag = EntryFlag::LowerBound;
                    break;
                }
            }
        }

        if !self.is_stopped() {
            let hash_entry = TTEntry::new(0, best_score, hash_flag, best_move, position.hash_key);
            self.tt.write(position.hash_key, hash_entry);
        }
        best_score
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
    pv: [[Move; MAX_DEPTH]; MAX_DEPTH],
    pv_length: [usize; MAX_DEPTH],

    delta: i32,
    alpha: i32,
    beta: i32,
    depth: u8,

    show_thinking: bool,
    start_time: Instant,
}

impl IterDeepData {
    fn new<const SHOW_THINKING: bool>(start_time: Instant) -> Self {
        Self {
            eval: 0,
            pv: [[NULL_MOVE; MAX_DEPTH]; MAX_DEPTH],
            pv_length: [0; MAX_DEPTH],
            delta: read_param!(ASPIRATION_WINDOW),
            alpha: -INFINITY,
            beta: INFINITY,
            depth: 1,
            show_thinking: SHOW_THINKING,
            start_time,
        }
    }
}

fn aspiration_window(position: &mut Board, s: &mut Thread, id: &mut IterDeepData) -> i32 {
    loop {
        // Most engines don't use aspiration windows for the first few depths since the search
        // won't be very accurate. However, since Panda preserves the width of the window from the
        // previous depth, it seems that doing aspiration windows on early depths is effective for
        // setting up the window for the future. As of Panda 1.1, this approach does gain elo.

        #[cfg(feature = "datagen")]
        {
            (id.alpha, id.beta) = (-INFINITY, INFINITY);
        }

        let eval = s.negamax(position, id.depth.max(1), id.alpha, id.beta, false);

        if s.is_stopped() || Instant::now() >= s.timer.end_time {
            if s.moves_fully_searched > 0 {
                id.pv = s.pv;
                id.pv_length = s.pv_length;
            }

            return eval;
            //this return value will not be used outside of datagen mode,
            //in which case it comes from a full window search
        }

        s.moves_fully_searched = 0;

        if eval > id.alpha && eval < id.beta {
            //within window -> just update pv and set up for next iteration

            id.pv = s.pv;
            id.pv_length = s.pv_length;

            id.delta = read_param!(ASPIRATION_WINDOW);

            id.alpha = eval - id.delta;
            id.beta = eval + id.delta;

            if id.show_thinking {
                print_thinking(id.depth, eval, s, id.start_time);
            }

            return eval;
        }

        if eval <= id.alpha {
            //failed low -> widen window down, do not update pv
            id.alpha = (id.alpha - id.delta).max(-INFINITY);
            id.beta = (id.alpha + id.beta) / 2;
            id.delta += id.delta / 2;
        } else if eval >= id.beta {
            //failed high -> widen window up, also update pv
            id.beta = (id.beta + id.delta).min(INFINITY);
            id.delta += id.delta / 2;

            id.pv = s.pv;
            id.pv_length = s.pv_length;
        }
    }
}

pub fn iterative_deepening<const SHOW_THINKING: bool>(
    position: &mut Board,
    soft_limit: usize,
    hard_limit: usize,
    max_depth: u8,
    s: &mut Thread,
) -> MoveData {
    let start = Instant::now();

    s.reset_thread();
    s.timer.end_time = start + Duration::from_millis(hard_limit as u64);

    let mut id = IterDeepData::new::<SHOW_THINKING>(start);

    while id.depth <= (MAX_DEPTH as u8).min(max_depth) {
        let eval = aspiration_window(position, s, &mut id);

        if s.is_stopped() {
            break;
        }

        id.eval = eval;
        id.depth += 1;

        let fraction = s.info.nodetable.get(id.pv[0][0]) as f64 / s.nodes as f64;

        let multiplier = 2.2 * (1.3 * fraction).cos(); //guessed with some desmos eyeballing

        let soft_end = id.start_time + Duration::from_millis((soft_limit as f64 * multiplier) as u64);
        let mut end = s.timer.end_time;
        if soft_limit < hard_limit {
            end = end.min(soft_end);
        }

        if Instant::now() > end {
            //not the same as above break statement because eval was updated
            //which won't affect choice of move but will affect data we report
            s.stop.store(true, Relaxed);
            break;
        }
    }

    let pv = id.pv[0].iter().take(id.pv_length[0]).fold(String::new(), |acc, m| acc + (m.uci() + " ").as_str());

    MoveData { m: id.pv[0][0], nodes: s.nodes, eval: id.eval, pv }
}
