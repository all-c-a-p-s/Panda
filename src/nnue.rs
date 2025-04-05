use std::mem;

use crate::eval::MIRROR;
use crate::*;

use crate::types::*;

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

const fn nnue_index(piece: Piece, sq: Square) -> (usize, usize) {
    const PIECE_STEP: usize = 64;

    let white_idx = PIECE_STEP * piece as usize + sq as usize;
    let black_idx = PIECE_STEP * ((piece as usize + 6) % 12) + MIRROR[sq as usize];

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
            a.set_weight::<ON>(unsafe { board.pieces_array[sq].unwrap_unchecked() }, sq);
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
    pub fn set_weight<const STATE: bool>(&mut self, piece: Piece, square: Square) {
        self.set::<STATE>(nnue_index(piece, square));
    }

    pub fn quiet_update(&mut self, piece: Piece, from: Square, to: Square) {
        self.set_weight::<OFF>(piece, from);
        self.set_weight::<ON>(piece, to);
    }

    pub fn capture_update(
        &mut self,
        _piece: Piece,
        victim: Option<Piece>,
        _from: Square,
        to: Square,
    ) {
        //SAFETY: this is only called when there is a capture
        self.set_weight::<OFF>(unsafe { victim.unwrap_unchecked() }, to);
    }

    //promotions that are also captures handled in capture_update()
    pub fn promotion_update(
        &mut self,
        piece: Piece,
        promotion: Option<Piece>,
        from: Square,
        to: Square,
    ) {
        self.set_weight::<OFF>(piece, from);
        self.set_weight::<ON>(unsafe { promotion.unwrap_unchecked() }, to);
    }

    pub fn ep_update(&mut self, piece: Piece, victim: Option<Piece>, _from: Square, to: Square) {
        let ep = match piece {
            Piece::WP => unsafe { to.sub_unchecked(8) },
            Piece::BP => unsafe { to.add_unchecked(8) },
            _ => unreachable!(),
        };

        self.set_weight::<OFF>(unsafe { victim.unwrap_unchecked() }, ep);
    }

    //update to king already done
    pub fn castling_update(&mut self, _piece: Piece, _from: Square, to: Square) {
        match to {
            Square::C1 => self.quiet_update(Piece::WR, Square::A1, Square::D1),
            Square::G1 => self.quiet_update(Piece::WR, Square::H1, Square::F1),
            Square::C8 => self.quiet_update(Piece::BR, Square::A8, Square::D8),
            Square::G8 => self.quiet_update(Piece::BR, Square::H8, Square::F8),
            _ => unreachable!(),
        }
    }

    //piece already moved
    pub fn undo_ep(&mut self, piece: Piece, victim: Option<Piece>, _from: Square, to: Square) {
        let ep = match piece {
            Piece::WP => unsafe { to.sub_unchecked(8) },
            Piece::BP => unsafe { to.add_unchecked(8) },
            _ => unreachable!(),
        };
        self.set_weight::<ON>(unsafe { victim.unwrap_unchecked() }, ep);
    }

    //update to king already done
    pub fn undo_castling(&mut self, _piece: Piece, _from: Square, to: Square) {
        match to {
            Square::C1 => self.quiet_update(Piece::WR, Square::D1, Square::A1),
            Square::G1 => self.quiet_update(Piece::WR, Square::F1, Square::H1),
            Square::C8 => self.quiet_update(Piece::BR, Square::D8, Square::A8),
            Square::G8 => self.quiet_update(Piece::BR, Square::F8, Square::H8),
            _ => unreachable!(),
        }
    }

    pub fn undo_move(
        &mut self,
        piece: Piece,
        victim: Option<Piece>,
        promotion: Option<Piece>,
        from: Square,
        to: Square,
    ) {
        if victim != None {
            self.set_weight::<ON>(unsafe { victim.unwrap_unchecked() }, to);
        }

        if promotion != None {
            self.set_weight::<OFF>(unsafe { promotion.unwrap_unchecked() }, to);
            self.set_weight::<ON>(piece, from);
        } else {
            self.quiet_update(piece, to, from);
        }
    }

    pub fn set_to_position(&mut self, board: &Board) {
        for piece in PIECES {
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
