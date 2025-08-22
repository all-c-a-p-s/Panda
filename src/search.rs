use crate::board::Board;
use crate::eval::evaluate;
use crate::helper::{piece_type, read_param, tuneable_params};
use crate::ordering::SEE_VALUES;
use crate::r#move::{Commit, Move, MoveList, NULL_MOVE};
use crate::thread::{SearchStackEntry, Thread};
use crate::transposition::{EntryFlag, TTEntry, TT};
use crate::types::PieceType;
use crate::uci::print_thinking;
use crate::Colour;

use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

pub const INFINITY: i32 = 1_000_000_000;
pub const MAX_PLY: usize = 64;
pub const MATE: i32 = INFINITY - MAX_PLY as i32;

pub const REDUCTION_LIMIT: u8 = 2;

const FULL_DEPTH_MOVES: u8 = 1;

const CORRHIST_GRAIN: i32 = 256;
const CORRHIST_SCALE: i32 = 256;
const CORRHIST_MAX: i32 = 256 * 32;
pub const CORRHIST_SIZE: usize = 16_384;

const HISTORY_MAX: i32 = 16_384;

// name, type, val, min, max

tuneable_params! {
    SINGULARITY_DE_MARGIN, i32, 25, 10, 150;
    ASPIRATION_WINDOW, i32, 20, 10, 70;
    RAZORING_MARGIN, i32, 273, 100, 500;
    MAX_RAZOR_DEPTH, u8, 2, 1, 12;
    RFP_DEPTH, u8, 6, 1, 12;
    RFP_MARGIN, u8, 27, 20, 200;
    TT_FUTILITY_MARGIN, i32, 150, 40, 400;
    HISTORY_PRUNING_DEPTH, i32, 4, 1, 12;
    SEE_PRUNING_DEPTH, i32, 8, 1, 12;
    SEE_QUIET_MARGIN, i32, 51, 20, 300;
    SEE_NOISY_MARGIN, i32, 37, 20, 250;
    SEE_QSEARCH_MARGIN, i32, 22, 1, 100;
    QSEARCH_FP_MARGIN, i32, 166, 1, 350;
    LMP_DEPTH, u8, 4, 1, 12;
    IIR_DEPTH_MINIMUM, u8, 9, 1, 12;
    HASH_MOVE_SCORE, i32, 1_000_000, 1_000_000, 1_000_000;
    QUEEN_PROMOTION, i32, 750_000, -999_999, 999_999;
    WINNING_CAPTURE, i32, 500_000, -999_999, 999_999;
    FIRST_KILLER_MOVE, i32, 94_419, -999_999, 999_999;
    LOSING_CAPTURE, i32, -300_000, -999_999, 999_999;
    UNDER_PROMOTION, i32, -500_000, -999_999, 999_999;
    COUNTERMOVE_BONUS, i32, 55_151, -999_999, 999999;
    FOLLOWUP_BONUS, i32, 20_000, -999_999, 999_999;
    NMP_FACTOR, i32, 20, 1, 100;
    NMP_BASE, i32, 200, 50, 500;
    HISTORY_NODE_DIVISOR, usize, 1024, 256, 8192;
    HISTORY_MIN_THRESHOLD, i32, 8192, 1024, 32768;
    LMR_TACTICAL_BASE, i32, 33, 0, 500;
    LMR_TACTICAL_DIVISOR, i32, 320, 100, 500;
    LMR_QUIET_BASE, i32, 164, 0, 500;
    LMR_QUIET_DIVISOR, i32, 280, 100, 500;
}

const DO_SINGULARITY_EXTENSION: bool = true;
const DO_SINGULARITY_DE: bool = true;

pub const MAX_GAME_PLY: usize = 1024;

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

    /// The purpose of the `singularity()` function is to prove that a move is better than alternatives by
    /// a significant margin. If this is true, we should extend it since it is more important. This
    /// function determines how much we should extend by.

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
        let threshold = (tt_score - (i32::from(depth) * 2 + 20)).max(-INFINITY);

        self.info.excluded[self.ply] = Some(best_move);

        let excluded_eval = self.negamax(position, depth / 2, threshold - 1, threshold, cutnode);

        self.info.excluded[self.ply] = None;

        if DO_SINGULARITY_DE
            && !pv_node
            && excluded_eval < threshold - read_param!(SINGULARITY_DE_MARGIN)
            && self.double_extensions <= 6
        {
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
    /// - All-node: a node in which all moves were searched and alpha returned (upper bound)
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
        let singular = self.info.excluded[self.ply].is_some();

        self.pv_length[self.ply] = self.ply;

        if depth == 0 {
            return self.qsearch(position, alpha, beta);
        }

        let mut hash_flag = EntryFlag::UpperBound;
        self.nodes += 1;

        let (mut tt_depth, mut tt_bound, mut tt_score, mut tt_hit) =
            (0, EntryFlag::Missing, 0, false);
        let mut best_move = NULL_MOVE;

        let in_check = position.checkers != 0;

        if !root && !singular {
            if position.is_drawn() {
                return 0;
            }

            // Mate Distance Pruning:
            // Check if line is so good/bad that being mated in the current ply
            // or mating in the next ply would not change alpha/beta
            let r_alpha = alpha.max(-INFINITY + self.ply as i32);
            let r_beta = beta.min(INFINITY - self.ply as i32 - 1);
            if r_alpha >= r_beta {
                return r_alpha;
            }

            if let Some(entry) = self.tt.lookup(position.hash_key) {
                best_move = entry.best_move;
                tt_hit = true;
                tt_score = entry.eval;
                tt_depth = entry.depth;
                tt_bound = entry.flag;

                // We accept values from the TT if:
                //      (1) the depth of the entry >= our depth
                // OR   (2) we don't expect much from this node, and the eval is well below beta
                if !singular {
                    if !pv_node
                        && depth <= entry.depth
                        && match entry.flag {
                            EntryFlag::Exact => true,
                            EntryFlag::LowerBound => entry.eval >= beta,
                            EntryFlag::UpperBound => entry.eval <= alpha,
                            EntryFlag::Missing => false,
                        }
                    {
                        return entry.eval;
                    } else if cutnode
                        && entry.eval
                            - read_param!(TT_FUTILITY_MARGIN)
                                * i32::from(depth - entry.depth).max(1)
                            >= beta
                        && entry.flag != EntryFlag::UpperBound
                        && !in_check
                    {
                        return entry.eval;
                    }
                }
            }
        }

        let tt_move = !best_move.is_null();
        let tt_move_capture = if tt_move {
            best_move.is_capture(position)
        } else {
            false
        };

        //reset killers for child nodes
        self.info.killer_moves[self.ply + 1] = None;

        let mut static_eval = evaluate(position);
        if !singular {
            static_eval = self.eval_with_corrhist(position, static_eval);
        }

        if tt_hit
            && !((static_eval > tt_score && tt_bound == EntryFlag::LowerBound)
                || (static_eval < tt_score && tt_bound == EntryFlag::UpperBound))
        {
            static_eval = tt_score;
        }

        if self.ply < MAX_PLY {
            self.info.ss[self.ply] = SearchStackEntry {
                eval: static_eval,
                previous_square: None,
                previous_piece: None,
                made_capture: false,
            };
        }

        // Improving heuristic:
        // useful for considering whether or not to prune in the search:
        // - if improving then we should expect to fail high more
        // - and to fail low less
        // opposite goes for opponent_worsening
        let improving = match self.ply {
            2.. => self.info.ss[self.ply].eval > self.info.ss[self.ply - 2].eval,
            _ => false,
        } && !in_check;

        let opponent_worsening = match self.ply {
            3.. => self.info.ss[self.ply - 1].eval < self.info.ss[self.ply - 3].eval,
            _ => false,
        };

        let opponent_captured = match self.ply {
            1.. => self.info.ss[self.ply - 1].made_capture,
            _ => false,
        };

        // Static pruning: here we attempt to show that the position does not require any further
        // search
        if !in_check && !singular && self.do_pruning && !pv_node {
            // Reverse Futility Pruning:
            // If eval >= beta + some margin, assume that we can achieve at least beta
            if depth <= read_param!(RFP_DEPTH)
                && static_eval
                    - i32::from(read_param!(RFP_MARGIN) * (depth - u8::from(improving)))
                    >= beta
            {
                return static_eval;
            }

            // Razoring:
            // If we're very far behind it's likely that the only way to raise alpha will be with
            // captures, so just run a qsearch
            if depth <= read_param!(MAX_RAZOR_DEPTH)
                && static_eval
                    + read_param!(RAZORING_MARGIN)
                        * i32::from(
                            depth + u8::from(improving)
                                - u8::from(opponent_captured && !opponent_worsening),
                        )
                    <= alpha
            {
                let qeval = self.qsearch(position, alpha, beta);
                if qeval < alpha {
                    return qeval;
                }
            }

            // Null Move Pruning (NMP):
            // If we are still able to reach an eval >= beta if we give our opponent
            // another move, then their previous move was probably bad
            if !position.is_kp_endgame()
                && !position.last_move_null
                && static_eval + read_param!(NMP_FACTOR) * i32::from(depth) - read_param!(NMP_BASE)
                    >= beta
                && !root
            {
                let undo = position.make_null_move();
                self.ply += 1;
                let r = 2
                    + i32::from(depth) / 4
                    + ((static_eval - beta) / 256).min(3)
                    + i32::from(improving)
                    + i32::from(opponent_worsening);
                let reduced_depth = (i32::from(depth) - r).max(1) as u8;
                let null_move_eval =
                    -self.negamax(position, reduced_depth, -beta, -beta + 1, !cutnode);
                //minimal window used because all that matters is whether the search result is better than beta
                position.undo_null_move(&undo);
                self.ply -= 1;
                if null_move_eval >= beta {
                    return beta;
                }
            }
        }

        // Internal Iterative Reduction (IIR):
        // if we don't have a TT hit then move ordering here will be terrible
        // so its better to reduce and set up TT move for next iteration
        if (pv_node || cutnode) && depth >= read_param!(IIR_DEPTH_MINIMUM) && !tt_move {
            depth -= 1;
        }

        let (mut quiets, mut caps) = (vec![], vec![]);

        let mut move_list = MoveList::gen_moves::<false>(position);
        let mut scores = move_list.get_scores(self, position, &best_move);

        let (mut legal, mut considered) = (0, 0);
        let mut skip_quiets = false;
        let mut best_score = -INFINITY;

        while let Some((m, _ms)) = move_list.get_next(&mut scores) {
            if let Some(n) = self.info.excluded[self.ply] {
                if n == m {
                    considered += 1;
                    continue;
                }
            }

            considered += 1;

            let tactical = m.is_tactical(position);
            let quiet = !tactical;
            let not_mated = best_score > -MATE;

            let is_killer = self.info.killer_moves[self.ply] == Some(m);
            let is_check = position.checkers != 0;

            // Early Pruning: try to prune moves before we search them properly
            // by showing that they're not worth investigating
            #[cfg(not(feature = "datagen"))]
            if !root && not_mated {
                if quiet && skip_quiets && !is_killer {
                    continue;
                }

                // Late Move Pruning (LMP):
                // after a certain point start skipping all quiets after the current
                // move. The threshold I'm currently using comes from Weiss
                let lmp_threshold = if improving {
                    2 + depth * depth
                } else {
                    depth * depth / 2
                };
                if depth <= read_param!(LMP_DEPTH) && legal > lmp_threshold && !in_check {
                    skip_quiets = true;
                }

                let r = self.info.lmr_table.reduction_table[usize::from(quiet)]
                    [depth.min(31) as usize][considered.min(31) as usize]
                    + i32::from(!improving);
                let lmr_depth = i32::from(depth) - 1 - r.max(0);

                // SEE Pruning:
                // skip moves that fail SEE by a depth-dependent threshold
                if lmr_depth <= read_param!(SEE_PRUNING_DEPTH) && considered > 1 && !pv_node {
                    let margin = if tactical {
                        read_param!(SEE_NOISY_MARGIN)
                    } else {
                        read_param!(SEE_QUIET_MARGIN)
                    };
                    let threshold = margin * i32::from(depth);
                    if !m.see(position, threshold) {
                        continue;
                    }
                }
            }

            let Ok(commit) = position.try_move(m) else {
                continue;
            };

            let nodes_before = self.nodes;
            legal += 1;
            self.ply += 1;
            // update after pruning above

            // update for countermove heuristic
            self.info.ss[self.ply].previous_piece = Some(position.get_piece_at(m.square_to()));
            self.info.ss[self.ply].previous_square = Some(m.square_to());

            if m.is_capture(position) {
                self.info.ss[self.ply].made_capture = true;
            }

            // A singular move is a move which seems to be forced or at least much stronger than
            // others. We should therefore extend to investigate it further.
            let maybe_singular = DO_SINGULARITY_EXTENSION
                && !root
                && depth >= 8
                && !singular
                && m == best_move
                && tt_depth >= depth - 3
                && tt_bound != EntryFlag::UpperBound;

            let extension = if maybe_singular {
                self.singularity(
                    position, best_move, &commit, tt_score, depth, pv_node, alpha, beta, cutnode,
                )
            } else {
                Some(i32::from(in_check && !root))
            };

            if extension.is_none() {
                // MultiCut case from singularity() function
                return tt_score - (i32::from(depth) * 2);
            } else if maybe_singular {
                position.play_unchecked(best_move);
                self.ply += 1;
                // we unmade the move while calling the singularity() function
                if extension == Some(2) {
                    self.double_extensions += 1;
                }
            }

            let new_depth =
                i32::clamp(i32::from(depth) - 1 + extension.unwrap(), 0, MAX_PLY as i32) as u8;

            let eval = if legal == 1 {
                // note that this is one because the variable is updated above
                -self.negamax(position, new_depth, -beta, -alpha, false)
                // normal search on pv move (no moves searched yet)
            } else {
                // non-pv move -> search with reduced window
                // this assumes that our move ordering is good enough
                // that we will be able to prove that these moves are bad
                // often enough that it outweighs the cost of re-searching
                // then if we are unable to prove so

                let mut reduction_eval = if legal
                    > (FULL_DEPTH_MOVES
                        + u8::from(pv_node)
                        + u8::from(!tt_move)
                        + u8::from(root)
                        + u8::from(tactical))
                    && depth >= REDUCTION_LIMIT
                    && not_mated
                {
                    let mut r = 1;
                    if quiet {
                        r = self.info.lmr_table.reduction_table[usize::from(quiet)]
                            [depth.min(31) as usize][legal.min(31) as usize];

                        r -= i32::from(pv_node);
                        r += i32::from(tt_move_capture);
                        r += i32::from(!improving);

                        r -= i32::from(is_check);

                        r -= self.info.history_table[m.piece_moved(position)][m.square_to()] / 8192;
                    }

                    let mut reduced_depth = (i32::from(new_depth) - r).max(1) as u8;
                    reduced_depth = reduced_depth.clamp(1, new_depth);
                    // avoid dropping into qsearch or extending

                    -self.negamax(position, reduced_depth, -alpha - 1, -alpha, true)
                } else {
                    alpha + 1
                };
                if reduction_eval > alpha {
                    // failed to prove that move is bad -> re-search with same depth but reduced
                    // window
                    reduction_eval =
                        -self.negamax(position, new_depth, -alpha - 1, -alpha, !cutnode);
                }

                if reduction_eval > alpha && reduction_eval < beta {
                    // move actually inside PV window -> search at full depth
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
                self.info.nodetable.add(m, self.nodes - nodes_before);
                self.moves_fully_searched += 1;
                // used to ensure in the iterative deepening search that
                // at least one move has been searched fully
            }

            if quiet {
                quiets.push(m);
            } else {
                caps.push(m);
            }

            best_score = best_score.max(eval);

            if eval > alpha {
                alpha = eval;
                self.update_pv(m);
                hash_flag = EntryFlag::Exact;
                best_move = m;
            }

            if eval >= beta {
                self.update_search_tables(position, &quiets, &caps, m, tactical, depth);
                hash_flag = EntryFlag::LowerBound;
                break;
            }
        }

        if legal == 0 {
            return (-INFINITY + self.ply as i32) * i32::from(in_check);
        }

        if !self.is_stopped() && !singular {
            let hash_entry =
                TTEntry::new(depth, best_score, hash_flag, best_move, position.hash_key);

            self.tt.write(position.hash_key, hash_entry);

            if !(in_check
                || best_move.is_capture(position)
                || (hash_flag == EntryFlag::LowerBound && best_score <= static_eval)
                || (hash_flag == EntryFlag::UpperBound && best_score >= static_eval))
            {
                self.update_corrhist(position, depth, best_score - static_eval);
            }
        }

        best_score
    }

    /// Quiescence Search:
    /// Search all noisy moves, or all moves if in check.
    /// This is done to prevent the horizon effect.
    pub fn qsearch(&mut self, position: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;

        if position.is_drawn() {
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
                return entry.eval;
            }
        }

        let mut eval = evaluate(position);
        eval = self.eval_with_corrhist(position, eval);

        let in_check = position.checkers != 0;

        if eval >= beta {
            return beta;
        }
        let mut best_score = eval;

        alpha = alpha.max(best_score);

        let mut captures = if in_check {
            MoveList::gen_moves::<false>(position)
        } else {
            MoveList::gen_moves::<true>(position)
        };

        let mut scores = captures.get_scores(self, position, &best_move);

        while let Some((m, _ms)) = captures.get_next(&mut scores) {
            if m.is_capture(position) {
                // if not capture then must be a check evasion
                let best_case = eval + SEE_VALUES[piece_type(position.get_piece_at(m.square_to()))];
                let worst_case = best_case - SEE_VALUES[piece_type(m.piece_moved(position))];

                //first check if we beat beta even in the worst case
                if worst_case > beta {
                    return beta;
                }
            }

            //next check if we fail SEE by threshold
            if !m.see(position, read_param!(SEE_QSEARCH_MARGIN)) {
                continue;
            }

            //if we're far behind, only consider moves which win significant material
            if best_score + read_param!(QSEARCH_FP_MARGIN) <= alpha
                && !m.see(
                    position,
                    SEE_VALUES[PieceType::Knight] - SEE_VALUES[PieceType::Bishop] - 1,
                )
            {
                continue;
            }

            let Ok(commit) = position.try_move(m) else {
                continue;
            };

            self.ply += 1;

            let eval = -self.qsearch(position, -beta, -alpha);
            position.undo_move(m, &commit);
            self.ply -= 1;

            if self.is_stopped() {
                return 0;
            }

            best_score = best_score.max(eval);
            if eval > alpha {
                alpha = eval;
                hash_flag = EntryFlag::Exact;
                best_move = m;
            }
            if alpha >= beta {
                break;
            }
        }

        if !self.is_stopped() {
            let hash_entry = TTEntry::new(0, best_score, hash_flag, best_move, position.hash_key);
            self.tt.write(position.hash_key, hash_entry);
        }
        best_score
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
        quiets: &Vec<Move>,
        caps: &Vec<Move>,
        cutoff_move: Move,
        tactical: bool,
        depth: u8,
    ) {
        self.update_history(b, quiets, caps, cutoff_move, tactical, depth);
        if !tactical {
            self.update_killer_moves(cutoff_move);
            self.update_counter_moves(cutoff_move);
            self.update_followup(cutoff_move);
        }
    }

    pub fn update_killer_moves(&mut self, cutoff_move: Move) {
        self.info.killer_moves[self.ply] = Some(cutoff_move);
    }

    pub fn update_counter_moves(&mut self, cutoff_move: Move) {
        self.info.ss[self.ply].previous_piece.inspect(|x| {
            self.info.ss[self.ply].previous_square.inspect(|y| {
                self.info.counter_moves[*x][*y] = cutoff_move;
            });
        });
    }

    pub fn update_followup(&mut self, cutoff_move: Move) {
        if self.ply == 0 {
            return;
        }
        self.info.ss[self.ply - 1].previous_piece.inspect(|x| {
            self.info.ss[self.ply - 1].previous_square.inspect(|y| {
                self.info.followup_moves[*x][*y] = cutoff_move;
            });
        });
    }

    pub fn update_corrhist(&mut self, b: &Board, depth: u8, diff: i32) {
        let idx = (b.pawn_hash() % 16384) as usize;
        let side = usize::from(b.side_to_move == Colour::White);

        let entry = &mut self.info.corrhist[side][idx];

        let new_weight = i32::from((depth + 1).min(16));
        let scaled_diff = diff + CORRHIST_GRAIN;

        *entry =
            (*entry * (CORRHIST_SCALE - new_weight) + scaled_diff * new_weight) / CORRHIST_SCALE;
        *entry = (*entry).clamp(-CORRHIST_MAX, CORRHIST_MAX);
    }

    #[must_use] pub fn eval_with_corrhist(&self, b: &Board, raw_eval: i32) -> i32 {
        let idx = (b.pawn_hash() % 16384) as usize;
        let side = usize::from(b.side_to_move == Colour::White);

        let entry = self.info.corrhist[side][idx];
        (raw_eval + entry / CORRHIST_GRAIN).clamp(-MATE + 1, MATE - 1)
    }

    pub fn age_corrhist(&mut self) {
        self.info
            .corrhist
            .iter_mut()
            .for_each(|side| side.iter_mut().for_each(|k| *k /= 2));
    }

    pub fn update_history(
        &mut self,
        b: &Board,
        quiets: &Vec<Move>,
        caps: &Vec<Move>,
        cutoff_move: Move,
        tactical: bool,
        depth: u8,
    ) {
        let bonus = (300 * i32::from(depth) - 250).clamp(-HISTORY_MAX, HISTORY_MAX);
        //penalise all moves that have been checked and have not caused beta cutoff

        let update = |entry: &mut i32, m: Move| {
            let sign = if m == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus / HISTORY_MAX;
            *entry += delta;
        };

        if tactical {
            // penalise all captures that failed to cause cutoff
            for &m in caps {
                let piece = m.piece_moved(b);
                let target = m.square_to();
                let captured = piece_type(m.piece_captured(b));

                let entry = &mut self.info.caphist_table[piece][target][captured];
                update(entry, m);
            }
        } else {
            // penalise all moves quiets that failed to cause cutoff
            for &m in quiets {
                let piece = m.piece_moved(b);
                let target = m.square_to();

                let entry = &mut self.info.history_table[piece][target];
                update(entry, m);
            }

            for &m in caps {
                let piece = m.piece_moved(b);
                let target = m.square_to();
                let captured = piece_type(m.piece_captured(b));
                let entry = &mut self.info.caphist_table[piece][target][captured];
                update(entry, m);
            }
        }
    }

    pub fn reset_thread(&mut self) {
        self.nodes = 0;
        self.pv_length = [0; 64];
        self.pv = [[NULL_MOVE; MAX_PLY]; MAX_PLY];
        self.ply = 0;
        self.moves_fully_searched = 0;

        //reset tables
        self.info.killer_moves = [None; MAX_PLY];
        self.info.history_table = [[0; 64]; 12];
        self.info.caphist_table = [[[0; 5]; 64]; 12];
        self.age_corrhist();
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
}

impl IterDeepData {
    fn new<const SHOW_THINKING: bool>(start_time: Instant) -> Self {
        Self {
            eval: 0,
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            pv_length: [0; MAX_PLY],
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
        let eval = s.negamax(position, id.depth.max(1), id.alpha, id.beta, false);

        if s.is_stopped() || Instant::now() >= s.timer.end_time {
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
    s: &mut Thread,
) -> MoveData {
    let start = Instant::now();

    s.reset_thread();
    s.timer.end_time = start + Duration::from_millis(hard_limit as u64);

    let mut id = IterDeepData::new::<SHOW_THINKING>(start);

    while (id.depth as usize) < MAX_PLY {
        let eval = aspiration_window(position, s, &mut id);

        if s.is_stopped() {
            break;
        }

        id.eval = eval;
        id.depth += 1;

        let fraction = s.info.nodetable.get(id.pv[0][0]) as f64 / s.nodes as f64;
        let multiplier = 2.2 * (1.3 * fraction).cos(); //guessed with some desmos eyeballing

        let soft_end =
            id.start_time + Duration::from_millis((soft_limit as f64 * multiplier) as u64);
        let end = s.timer.end_time.min(soft_end);

        if Instant::now() > end {
            //not the same as above break statement because eval was updated
            //which won't affect choice of move but will affect data we report
            s.stop.store(true, Relaxed);
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
