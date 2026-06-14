use crate::Colour;
use crate::board::Board;
use crate::util::helper::piece_type;
use crate::board::r#move::{Move, NULL_MOVE};
use crate::search::{INFINITY, MAX_DEPTH};
use crate::search::thread::{CORRHIST_SIZE, NodeTable, Thread};

const HISTORY_MAX: i32 = 16_384;
const CORRELATION_MAX: i32 = 4_096;

const CORRHIST_GRAIN: i32 = 256;
const CORRHIST_SCALE: i32 = 256;
const CORRHIST_MAX: i32 = 256 * 32;

const MATE: i32 = INFINITY - MAX_DEPTH as i32;

impl Thread<'_> {
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
        quiets: &[Move],
        tacticals: &[Move],
        cutoff_move: Move,
        tactical: bool,
        depth: u8,
    ) {
        self.update_history(b, quiets, tacticals, cutoff_move, tactical, depth);
        self.update_counter_correlation(cutoff_move, depth, tactical, tacticals, quiets, b);
        self.update_followup_correlation(cutoff_move, depth, tactical, tacticals, quiets, b);
        if !tactical {
            self.update_killer_moves(cutoff_move);
        }
    }

    pub fn update_killer_moves(&mut self, cutoff_move: Move) {
        self.info.killer_moves[self.ply] = Some(cutoff_move);
    }

    pub fn update_followup_correlation(
        &mut self,
        cutoff_move: Move,
        depth: u8,
        tactical: bool,
        tacticals: &[Move],
        quiets: &[Move],
        b: &Board,
    ) {
        if self.ply <= 1 {
            return;
        }
        let bonus = (30 * depth as i32 - 20).clamp(-CORRELATION_MAX, CORRELATION_MAX);

        let update = |entry: &mut i32, m: Move| {
            let sign = if m == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus / CORRELATION_MAX;
            *entry += delta;
        };

        let Some(prev) = self.info.ss[self.ply - 2].square_moved_to else {
            return;
        };

        let side = (b.side_to_move == Colour::White) as usize;

        if tactical {
            for &m in tacticals {
                let sq = m.square_to();

                let entry = &mut self.info.followup_correlation[side][prev][sq];
                update(entry, m);
            }
        } else {
            for &m in tacticals {
                let sq = m.square_to();

                let entry = &mut self.info.followup_correlation[side][prev][sq];
                update(entry, m);
            }

            for &m in quiets {
                let sq = m.square_to();

                let entry = &mut self.info.followup_correlation[side][prev][sq];
                update(entry, m);
            }
        }
    }

    pub fn update_counter_correlation(
        &mut self,
        cutoff_move: Move,
        depth: u8,
        tactical: bool,
        tacticals: &[Move],
        quiets: &[Move],
        b: &Board,
    ) {
        if self.ply == 0 {
            return;
        }
        let bonus = (100 * depth as i32 - 50).clamp(-CORRELATION_MAX, CORRELATION_MAX);

        let update = |entry: &mut i32, m: Move| {
            let sign = if m == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus / CORRELATION_MAX;
            *entry += delta;
        };

        let Some(prev) = self.info.ss[self.ply - 1].square_moved_to else {
            return;
        };

        if !tactical {
            let Some(pc) = self.info.ss[self.ply - 1].piece_moved else {
                return;
            };

            self.info.counter_moves[pc][prev] = Some(cutoff_move);
        }

        let side = (b.side_to_move == Colour::White) as usize;

        if tactical {
            for &m in tacticals {
                let sq = m.square_to();

                let entry = &mut self.info.counter_correlation[side][prev][sq];
                update(entry, m);
            }
        } else {
            for &m in tacticals {
                let sq = m.square_to();

                let entry = &mut self.info.counter_correlation[side][prev][sq];
                update(entry, m);
            }

            for &m in quiets {
                let sq = m.square_to();

                let entry = &mut self.info.counter_correlation[side][prev][sq];

                update(entry, m);
            }
        }
    }

    pub fn update_history(
        &mut self,
        b: &Board,
        quiets: &[Move],
        tacticals: &[Move],
        cutoff_move: Move,
        tactical: bool,
        depth: u8,
    ) {
        let bonus = (300 * depth as i32 - 250).clamp(-HISTORY_MAX, HISTORY_MAX);
        //penalise all moves that have been checked and have not caused beta cutoff

        let update = |entry: &mut i32, m: Move| {
            let sign = if m == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus / HISTORY_MAX;
            *entry += delta;
        };

        if tactical {
            // penalise all captures that failed to cause cutoff
            for &m in tacticals {
                if !m.is_capture(b) {
                    continue;
                }
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

            for &m in tacticals {
                if !m.is_capture(b) {
                    continue;
                }
                let piece = m.piece_moved(b);
                let target = m.square_to();
                let captured = piece_type(m.piece_captured(b));
                let entry = &mut self.info.caphist_table[piece][target][captured];
                update(entry, m);
            }
        }
    }

    pub fn update_corrhist(&mut self, b: &Board, depth: u8, diff: i32) {
        let idx = b.pawn_hash as usize % CORRHIST_SIZE;
        let side = b.side_to_move as usize;

        let entry = &mut self.info.corrhist[side][idx];

        let new_weight = (depth + 1).min(16) as i32;

        let scaled_diff = diff * CORRHIST_GRAIN;

        *entry = (*entry * (CORRHIST_SCALE - new_weight) + scaled_diff * new_weight) / CORRHIST_SCALE;
        *entry = (*entry).clamp(-CORRHIST_MAX, CORRHIST_MAX);
    }

    pub fn eval_with_corrhist(&self, b: &Board, raw_eval: i32) -> i32 {
        let idx = b.pawn_hash as usize % CORRHIST_SIZE;
        let side = b.side_to_move as usize;

        let entry = self.info.corrhist[side][idx];
        (raw_eval + entry / CORRHIST_GRAIN).clamp(-MATE + 1, MATE - 1)
    }

    pub fn age_corrhist(&mut self) {
        self.info.corrhist[0].iter_mut().for_each(|x| *x /= 2);
        self.info.corrhist[1].iter_mut().for_each(|x| *x /= 2);
    }

    pub fn reset_thread(&mut self) {
        self.nodes = 0;
        self.info.nodetable = NodeTable::default();
        self.pv_length = [0; 64];
        self.pv = [[NULL_MOVE; MAX_DEPTH]; MAX_DEPTH];
        self.ply = 0;
        self.moves_fully_searched = 0;

        self.age_corrhist();
    }
}
