use crate::board::Board;
use crate::board::r#move::{Move, NULL_MOVE};
use crate::read_param;
use crate::search::thread::{CORRHIST_SIZE, NodeTable, Thread};
use crate::search::{INFINITY, MAX_DEPTH, params};
use crate::util::Piece;
use crate::util::helper::piece_type;

const HISTORY_MAX: i32 = 16_384;
const CONTHIST_MAX: i32 = 4_096;

/// This is not the actual maximum value attainable by a history score, but in practice extremely
/// high history scores won't be reached, so setting this to the maximum attainable history score
/// will dilute results significantly.
pub const EFFECTIVE_HISTORY_MAX: i32 = HISTORY_MAX + CONTHIST_MAX * 2;

const CORRHIST_GRAIN: i32 = 256;
const CORRHIST_MAX: i32 = 256 * 128;

const MATE: i32 = INFINITY - MAX_DEPTH as i32;

impl Thread<'_> {
    const NUM_CONTHISTS: usize = 2;
    pub fn update_pv(&mut self, mv: Move) {
        let next_ply = self.ply + 1;
        self.pv[self.ply][self.ply] = mv;
        for i in next_ply..self.pv_length[next_ply] {
            self.pv[self.ply][i] = self.pv[next_ply][i];
            //copy from next row in pv table
        }
        self.pv_length[self.ply] = self.pv_length[next_ply];
    }

    pub fn get_overall_history(&self, mv: Move, b: &Board, pc: Piece) -> i32 {
        self.get_history(mv, pc) + self.get_conthist(mv, b) + self.get_correlation_history(mv, b, pc)
    }

    pub fn update_search_tables(
        &mut self,
        b: &Board,
        quiets: &[Move],
        tacticals: &[Move],
        cutoff_move: Move,
        tactical: bool,
        can_be_killer: bool,
        depth: u8,
    ) {
        self.update_history(b, quiets, tacticals, cutoff_move, tactical, depth);
        self.update_conthist(cutoff_move, depth, tactical, tacticals, quiets, b);
        self.update_correlation_history(b, quiets, cutoff_move, tactical, depth);
        if can_be_killer {
            self.update_killer_moves(cutoff_move);
        }
    }

    pub fn update_killer_moves(&mut self, cutoff_move: Move) {
        self.info.killer_moves[self.ply] = Some(cutoff_move);
    }

    pub fn get_history(&self, mv: Move, pc: Piece) -> i32 {
        let side = pc.colour();
        let from = mv.square_from();
        let to = mv.square_to();

        let pc_sq = read_param!(HISTORY_PC_SQ_WEIGHT);
        let sq_sq = read_param!(HISTORY_SQ_SQ_WEIGHT);

        let s = pc_sq + sq_sq;

        (pc_sq * self.info.piece_history[pc][to] + sq_sq * self.info.square_history[side][from][to]) / s
    }

    pub fn get_conthist(&self, mv: Move, b: &Board) -> i32 {
        let tables = [&self.info.conthist_1ply, &self.info.conthist_2ply];
        let back = [1, 2];

        let sq = mv.square_to();
        let pc = mv.piece_moved(b);

        let mut r = 0;

        for i in 0..Self::NUM_CONTHISTS {
            if self.ply < back[i] {
                break;
            }

            let Some(old_sq) = self.info.ss[self.ply - back[i]].square_moved_to else {
                break;
            };

            let old_pc = self.info.ss[self.ply - back[i]].piece_moved.unwrap();

            r += tables[i][old_pc][old_sq][pc][sq];
        }

        r
    }

    pub fn get_correlation_history(&self, mv: Move, b: &Board, pc: Piece) -> i32 {
        let pawn_idx = b.pawn_hash as usize & (CORRHIST_SIZE - 1);
        let knb_idx = b.knb_hash as usize & (CORRHIST_SIZE - 1);
        let krq_idx = b.krq_hash as usize & (CORRHIST_SIZE - 1);
        let sq = mv.square_to();

        self.info.pawn_correlation[pawn_idx][pc][sq]
            + self.info.knb_correlation[knb_idx][pc][sq]
            + self.info.krq_correlation[krq_idx][pc][sq]
    }

    pub fn update_conthist(
        &mut self,
        cutoff_move: Move,
        depth: u8,
        tactical: bool,
        tacticals: &[Move],
        quiets: &[Move],
        b: &Board,
    ) {
        let tables = [&mut self.info.conthist_1ply, &mut self.info.conthist_2ply];
        let back = [1, 2];
        let mult = [100, 50];

        let update = |entry: &mut i32, mv: Move, bonus: i32| {
            let sign = if mv == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus.abs() / CONTHIST_MAX;
            *entry += delta;
        };

        for i in 0..Self::NUM_CONTHISTS {
            if self.ply < back[i] {
                break;
            }

            let bonus = (mult[i] * depth as i32 - mult[i] / 2).clamp(-CONTHIST_MAX, CONTHIST_MAX);

            let Some(old_sq) = self.info.ss[self.ply - back[i]].square_moved_to else {
                break;
            };
            let old_pc = self.info.ss[self.ply - back[i]].piece_moved.unwrap();

            if back[i] == 1 && !tactical {
                self.info.counter_moves[old_pc][old_sq] = Some(cutoff_move);
            }

            for &mv in tacticals {
                let sq = mv.square_to();
                let pc = mv.piece_moved(b);

                let entry = &mut tables[i][old_pc][old_sq][pc][sq];
                update(entry, mv, bonus);
            }

            if !tactical {
                for &mv in quiets {
                    let sq = mv.square_to();
                    let pc = mv.piece_moved(b);

                    let entry = &mut tables[i][old_pc][old_sq][pc][sq];
                    update(entry, mv, bonus);
                }
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
        let bonus = (150 * depth as i32 - 125).clamp(-HISTORY_MAX, HISTORY_MAX);
        //penalise all moves that have been checked and have not caused beta cutoff

        let update = |entry: &mut i32, mv: Move| {
            let sign = if mv == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus.abs() / HISTORY_MAX;
            *entry += delta;
        };

        // penalise all captures that failed to cause cutoff
        for &mv in tacticals {
            let piece = mv.piece_moved(b);
            let to = mv.square_to();
            let captured = piece_type(mv.piece_captured(b));

            let entry = &mut self.info.caphist[piece][to][captured];
            update(entry, mv);
        }

        if !tactical {
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
        }
    }

    pub fn update_correlation_history(
        &mut self,
        b: &Board,
        quiets: &[Move],
        cutoff_move: Move,
        tactical: bool,
        depth: u8,
    ) {
        let pawn_idx = b.pawn_hash as usize & (CORRHIST_SIZE - 1);
        let knb_idx = b.knb_hash as usize & (CORRHIST_SIZE - 1);
        let krq_idx = b.krq_hash as usize & (CORRHIST_SIZE - 1);

        let bonus = (150 * depth as i32 - 125).clamp(-HISTORY_MAX, HISTORY_MAX);
        //penalise all moves that have been checked and have not caused beta cutoff

        let update = |entry: &mut i32, mv: Move| {
            let sign = if mv == cutoff_move { 1 } else { -1 };
            let delta = (sign * bonus) - *entry * bonus.abs() / HISTORY_MAX;
            *entry += delta;
        };

        if !tactical {
            // penalise all moves quiets that failed to cause cutoff
            for &mv in quiets {
                let piece = mv.piece_moved(b);
                let to = mv.square_to();

                for entry in [
                    &mut self.info.pawn_correlation[pawn_idx][piece][to],
                    &mut self.info.knb_correlation[knb_idx][piece][to],
                    &mut self.info.krq_correlation[krq_idx][piece][to],
                ] {
                    update(entry, mv);
                }
            }
        }
    }

    pub fn update_corrhist(&mut self, b: &Board, depth: u8, diff: i32) {
        let pawn_idx = b.pawn_hash as usize & (CORRHIST_SIZE - 1);
        let knb_idx = b.knb_hash as usize & (CORRHIST_SIZE - 1);
        let krq_idx = b.krq_hash as usize & (CORRHIST_SIZE - 1);
        let side = b.side_to_move;

        let entries = [
            &mut self.info.pawn_corrhist[side][pawn_idx],
            &mut self.info.knb_corrhist[side][knb_idx],
            &mut self.info.krq_corrhist[side][krq_idx],
        ];

        let scaled_diff = diff * CORRHIST_GRAIN;
        let depth = depth as i32;

        let bonus = (scaled_diff * depth / 8).clamp(-CORRHIST_MAX / 4, CORRHIST_MAX / 4);

        for entry in entries {
            *entry += bonus - *entry * bonus.abs() / CORRHIST_MAX;
            *entry = (*entry).clamp(-CORRHIST_MAX, CORRHIST_MAX);
        }
    }

    pub fn eval_with_corrhist(&self, b: &Board, raw_eval: i32) -> i32 {
        let pawn_idx = b.pawn_hash as usize & (CORRHIST_SIZE - 1);
        let knb_idx = b.knb_hash as usize & (CORRHIST_SIZE - 1);
        let krq_idx = b.krq_hash as usize & (CORRHIST_SIZE - 1);
        let side = b.side_to_move;

        let pawn = self.info.pawn_corrhist[side][pawn_idx];
        let knb = self.info.knb_corrhist[side][knb_idx];
        let krq = self.info.krq_corrhist[side][krq_idx];

        let u = pawn * read_param!(PAWN_CORRHIST_WEIGHT)
            + knb * read_param!(KNB_CORRHIST_WEIGHT)
            + krq * read_param!(KRQ_CORRHIST_WEIGHT);

        let v = read_param!(PAWN_CORRHIST_WEIGHT) + read_param!(KNB_CORRHIST_WEIGHT) + read_param!(KRQ_CORRHIST_WEIGHT);

        let correction = u / v;

        (raw_eval + correction / CORRHIST_GRAIN).clamp(-MATE + 1, MATE - 1)
    }

    pub fn age_corrhist(&mut self) {
        for x in [&mut self.info.pawn_corrhist, &mut self.info.knb_corrhist, &mut self.info.krq_corrhist] {
            x[0].iter_mut().for_each(|y| *y /= 2);
            x[1].iter_mut().for_each(|y| *y /= 2);
        }
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
