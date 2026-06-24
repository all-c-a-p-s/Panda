use crate::board::Board;
use crate::board::r#move::{Move, NULL_MOVE};
use crate::search::thread::{CORRHIST_SIZE, NodeTable, Thread};
use crate::search::{INFINITY, MAX_DEPTH};
use crate::util::Piece;
use crate::util::helper::piece_type;

const HISTORY_MAX: i32 = 16_384;
const CORRELATION_MAX: i32 = 4_096;

pub const OVERALL_HISTORY_MAX: i32 = HISTORY_MAX + CORRELATION_MAX * 2;

const CORRHIST_GRAIN: i32 = 256;
const CORRHIST_SCALE: i32 = 256;
const CORRHIST_MAX: i32 = 256 * 32;

const MATE: i32 = INFINITY - MAX_DEPTH as i32;

impl Thread<'_> {
    pub fn update_pv(&mut self, mv: Move) {
        let next_ply = self.ply + 1;
        self.pv[self.ply][self.ply] = mv;
        for i in next_ply..self.pv_length[next_ply] {
            self.pv[self.ply][i] = self.pv[next_ply][i];
            //copy from next row in pv table
        }
        self.pv_length[self.ply] = self.pv_length[next_ply];
    }

    pub fn get_history(&self, mv: Move, pc: Piece) -> i32 {
        let side = pc.colour();
        let from = mv.square_from();
        let to = mv.square_to();

        self.info.piece_history[pc][to] + self.info.square_history[side][from][to]
    }

    pub fn get_cmh(&self, mv: Move, b: &Board) -> i32 {
        let sq = mv.square_to();
        if self.ply > 0
            && let Some(prev) = self.info.ss[self.ply - 1].square_moved_to
        {
            let side = b.side_to_move;
            self.info.counter_correlation[side][prev][sq]
        } else {
            0
        }
    }

    pub fn get_fmh(&self, mv: Move, b: &Board) -> i32 {
        let sq = mv.square_to();
        if self.ply > 1
            && let Some(prev) = self.info.ss[self.ply - 2].square_moved_to
        {
            let side = b.side_to_move;
            self.info.followup_correlation[side][prev][sq]
        } else {
            0
        }
    }

    pub fn get_conthist(&self, mv: Move, b: &Board) -> i32 {
        self.get_cmh(mv, b) + self.get_fmh(mv, b)
    }

    pub fn get_overall_history(&self, mv: Move, b: &Board, pc: Piece) -> i32 {
        self.get_history(mv, pc) + self.get_conthist(mv, b)
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

        let update = |entry: &mut i32, mv: Move| {
            let sign = if mv == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus / CORRELATION_MAX;
            *entry += delta;
        };

        let Some(prev) = self.info.ss[self.ply - 2].square_moved_to else {
            return;
        };

        let side = b.side_to_move;

        if tactical {
            for &mv in tacticals {
                let sq = mv.square_to();

                let entry = &mut self.info.followup_correlation[side][prev][sq];
                update(entry, mv);
            }
        } else {
            for &mv in tacticals {
                let sq = mv.square_to();

                let entry = &mut self.info.followup_correlation[side][prev][sq];
                update(entry, mv);
            }

            for &mv in quiets {
                let sq = mv.square_to();

                let entry = &mut self.info.followup_correlation[side][prev][sq];
                update(entry, mv);
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

        let update = |entry: &mut i32, mv: Move| {
            let sign = if mv == cutoff_move { 1 } else { -1 };
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

        let side = b.side_to_move;

        if tactical {
            for &mv in tacticals {
                let sq = mv.square_to();

                let entry = &mut self.info.counter_correlation[side][prev][sq];
                update(entry, mv);
            }
        } else {
            for &mv in tacticals {
                let sq = mv.square_to();

                let entry = &mut self.info.counter_correlation[side][prev][sq];
                update(entry, mv);
            }

            for &mv in quiets {
                let sq = mv.square_to();

                let entry = &mut self.info.counter_correlation[side][prev][sq];

                update(entry, mv);
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
        const SHM: i32 = HISTORY_MAX / 2; // max for history entry in a single table

        let bonus = (150 * depth as i32 - 125).clamp(-SHM, SHM);
        //penalise all moves that have been checked and have not caused beta cutoff

        let update = |entry: &mut i32, mv: Move| {
            let sign = if mv == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus / SHM;
            *entry += delta;
        };

        if tactical {
            // penalise all captures that failed to cause cutoff
            for &mv in tacticals {
                if !mv.is_capture(b) {
                    continue;
                }
                let piece = mv.piece_moved(b);
                let to = mv.square_to();
                let captured = piece_type(mv.piece_captured(b));

                let entry = &mut self.info.caphist[piece][to][captured];
                update(entry, mv);
            }
        } else {
            // penalise all moves quiets that failed to cause cutoff
            for &mv in quiets {
                let piece = mv.piece_moved(b);
                let to = mv.square_to();
                let from = mv.square_from();

                let entry = &mut self.info.piece_history[piece][to];
                update(entry, mv);

                let side = b.side_to_move;

                let entry = &mut self.info.square_history[side][from][to];
                update(entry, mv);
            }

            for &mv in tacticals {
                if !mv.is_capture(b) {
                    continue;
                }
                let piece = mv.piece_moved(b);
                let to = mv.square_to();
                let captured = piece_type(mv.piece_captured(b));
                let entry = &mut self.info.caphist[piece][to][captured];
                update(entry, mv);
            }
        }
    }

    pub fn update_corrhist(&mut self, b: &Board, depth: u8, diff: i32) {
        let idx = b.pawn_hash as usize % CORRHIST_SIZE;
        let side = b.side_to_move;

        let entry = &mut self.info.corrhist[side][idx];

        let new_weight = (depth + 1).min(16) as i32;

        let scaled_diff = diff * CORRHIST_GRAIN;

        *entry = (*entry * (CORRHIST_SCALE - new_weight) + scaled_diff * new_weight) / CORRHIST_SCALE;
        *entry = (*entry).clamp(-CORRHIST_MAX, CORRHIST_MAX);
    }

    pub fn eval_with_corrhist(&self, b: &Board, raw_eval: i32) -> i32 {
        let idx = b.pawn_hash as usize % CORRHIST_SIZE;
        let side = b.side_to_move;

        let entry = self.info.corrhist[side][idx];
        (raw_eval + entry / CORRHIST_GRAIN).clamp(-MATE + 1, MATE - 1)
    }

    pub fn age_corrhist(&mut self) {
        self.info.corrhist[0].iter_mut().for_each(|x| *x /= 2);
        self.info.corrhist[1].iter_mut().for_each(|x| *x /= 2);
    }

    pub fn reset_thread(&mut self) {
        self.nodes = 0;
        self.seldepth = 0;
        self.info.nodetable = NodeTable::default();
        self.pv_length = [0; 64];
        self.pv = [[NULL_MOVE; MAX_DEPTH]; MAX_DEPTH];
        self.ply = 0;
        self.moves_fully_searched = 0;

        self.age_corrhist();
    }
}
