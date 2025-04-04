use std::mem;

use crate::eval::MIRROR;
use crate::*;

const NUM_FEATURES: usize = 768;
const HL_SIZE: usize = 128;

const CR_MIN: i16 = 0;
const CR_MAX: i16 = 255;

const QA: i32 = 255;
const QAB: i32 = 255 * 64;

const SCALE: i32 = 400;

// The code in this file is very heavily inspired on the excellent and clear NNUE code form Carp and Viridithas,
// without which I wouldn't have been able to figure out how to implement NNUE.

#[repr(C)]
struct Network {
    feature_weights: [i16; NUM_FEATURES * HL_SIZE],
    feature_biases: [i16; HL_SIZE],
    output_weights: [i16; HL_SIZE * 2],
    output_bias: i16,
}

static MODEL: Network = unsafe {
    mem::transmute(*include_bytes!(
        "/Users/seba/rs/Panda/src/nets/quantised0002.bin"
    ))
};

type SideAccumulator = [i16; HL_SIZE];

const fn nnue_index(piece: usize, sq: usize) -> (usize, usize) {
    const PIECE_STEP: usize = 64;

    let white_idx = PIECE_STEP * piece + sq;
    let black_idx = PIECE_STEP * ((piece + 6) % 12) + MIRROR[sq];

    (white_idx * HL_SIZE, black_idx * HL_SIZE)
}

const ON: bool = true;
const OFF: bool = false;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Accumulator {
    white: SideAccumulator,
    black: SideAccumulator,
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            white: MODEL.feature_biases,
            black: MODEL.feature_biases,
        }
    }
}

impl Accumulator {
    pub fn from_board(board: &Board) -> Self {
        let mut a = Accumulator::default();

        let mut occs = board.occupancies[BOTH];
        while let Some(sq) = lsfb(occs) {
            a.set_weight::<ON>(board.pieces_array[sq], sq);
            occs = pop_bit(sq, occs);
        }

        a
    }

    // update values of hidden layer nodes depending after a bit is changed
    fn set<const STATE: bool>(&mut self, idx: (usize, usize)) {
        fn s<const STATE: bool>(acc: &mut SideAccumulator, idx: usize) {
            for (x, &w) in acc
                .iter_mut()
                .zip(&MODEL.feature_weights[idx..idx + HL_SIZE])
            {
                *x += if STATE { w } else { -w };
            }
        }

        s::<STATE>(&mut self.white, idx.0);
        s::<STATE>(&mut self.black, idx.1);
    }

    //update state of HL node corresponding to state of one feature
    pub fn set_weight<const STATE: bool>(&mut self, piece: usize, square: usize) {
        self.set::<STATE>(nnue_index(piece, square));
    }

    pub fn quiet_update(&mut self, piece: usize, from: usize, to: usize) {
        self.set_weight::<OFF>(piece, from);
        self.set_weight::<ON>(piece, to);
    }

    pub fn capture_update(&mut self, _piece: usize, victim: usize, _from: usize, to: usize) {
        self.set_weight::<OFF>(victim, to);
    }

    //promotions that are also captures handled in capture_update()
    pub fn promotion_update(&mut self, piece: usize, promotion: usize, from: usize, to: usize) {
        self.set_weight::<OFF>(piece, from);
        self.set_weight::<ON>(promotion, to);
    }

    pub fn ep_update(&mut self, piece: usize, victim: usize, _from: usize, to: usize) {
        let ep = match piece {
            WP => to - 8,
            BP => to + 8,
            _ => unreachable!(),
        };

        self.set_weight::<OFF>(victim, ep);
    }

    //update to king already done
    pub fn castling_update(&mut self, _piece: usize, _from: usize, to: usize) {
        match to {
            C1 => self.quiet_update(WR, A1, D1),
            G1 => self.quiet_update(WR, H1, F1),
            C8 => self.quiet_update(BR, A8, D8),
            G8 => self.quiet_update(BR, H8, F8),
            _ => unreachable!(),
        }
    }

    //piece already moved
    pub fn undo_ep(&mut self, piece: usize, victim: usize, _from: usize, to: usize) {
        let ep = match piece {
            WP => to - 8,
            BP => to + 8,
            _ => unreachable!(),
        };
        self.set_weight::<ON>(victim, ep);
    }

    //update to king already done
    pub fn undo_castling(&mut self, _piece: usize, _from: usize, to: usize) {
        match to {
            C1 => self.quiet_update(WR, D1, A1),
            G1 => self.quiet_update(WR, F1, H1),
            C8 => self.quiet_update(BR, D8, A8),
            G8 => self.quiet_update(BR, F8, H8),
            _ => unreachable!(),
        }
    }

    pub fn undo_move(
        &mut self,
        piece: usize,
        victim: usize,
        promotion: usize,
        from: usize,
        to: usize,
    ) {
        if victim != NO_PIECE {
            self.set_weight::<ON>(victim, to);
        }

        if promotion != NO_PIECE {
            self.set_weight::<OFF>(promotion, to);
            self.set_weight::<ON>(piece, from);
        } else {
            self.quiet_update(piece, to, from);
        }
    }

    pub fn set_to_position(&mut self, board: &Board) {
        for piece in WP..=BK {
            let mut occ = board.bitboards[piece];
            while let Some(sq) = lsfb(occ) {
                self.set_weight::<ON>(piece, sq);
                occ = pop_bit(sq, occ);
            }
        }
    }

    pub fn evaluate(&self, side: Colour) -> i32 {
        let (us, them) = match side {
            Colour::White => (self.white.iter(), self.black.iter()),
            Colour::Black => (self.black.iter(), self.white.iter()),
        };

        let mut out = 0;
        for (&value, &weight) in us.zip(&MODEL.output_weights[..HL_SIZE]) {
            out += squared_crelu(value) * (weight as i32);
        }
        for (&value, &weight) in them.zip(&MODEL.output_weights[HL_SIZE..]) {
            out += squared_crelu(value) * (weight as i32);
        }

        (out / QA + MODEL.output_bias as i32) * SCALE / QAB
    }
}

fn squared_crelu(value: i16) -> i32 {
    let v = value.clamp(CR_MIN, CR_MAX) as i32;

    v * v
}
