use std::mem;

use crate::eval::MIRROR;
use crate::util::STARTPOS;
use crate::{Board, Colour, lsfb, pop_bit};

use crate::util::types::{OccupancyIndex, Piece, Square};

// ON or OFF for each piece / colour / square
const NUM_FEATURES: usize = 6 * 2 * 64;
const HL_SIZE: usize = 512;
const OUTPUT_BUCKETS: usize = 8;

const CR_MIN: i16 = 0;
const CR_MAX: i16 = 255;

const QA: i32 = 255;
const QAB: i32 = 255 * 64;

const SCALE: i32 = 400;

// The code in this file is very heavily inspired on the excellent and clear NNUE code form Carp and Viridithas,
// without which I wouldn't have been able to figure out how to implement NNUE.

#[repr(C, align(64))]
struct Network {
    feature_weights: [i16; NUM_FEATURES * HL_SIZE],
    feature_biases: [i16; HL_SIZE],
    output_weights: [i16; OUTPUT_BUCKETS * HL_SIZE * 2],
    output_biases: [i16; OUTPUT_BUCKETS],
}

static MODEL: Network = unsafe { mem::transmute(*include_bytes!("../nets/output_buckets.bin")) };

pub fn output_bucket(board: &Board) -> usize {
    let divisor = 32usize.div_ceil(OUTPUT_BUCKETS);
    let pcs = board.occupancies[OccupancyIndex::BothOccupancies].count_ones() as usize;

    (pcs - 2) / divisor
}

type SideAccumulator = [i16; HL_SIZE];

#[must_use]
pub const fn nnue_index(piece: Piece, sq: Square) -> (usize, usize) {
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
        Self { white: MODEL.feature_biases, black: MODEL.feature_biases }
    }
}

impl Accumulator {
    #[must_use]
    pub fn from_board(board: &Board) -> Self {
        let mut a = Accumulator::default();

        let mut occs = board.occupancies[OccupancyIndex::BothOccupancies];
        while let Some(sq) = lsfb(occs) {
            a.set_weight::<ON>(board.get_piece_at(sq), sq);
            occs = pop_bit(sq, occs);
        }

        a
    }

    pub fn from_startpos() -> Self {
        Self::from_board(&Board::from(STARTPOS))
    }

    // update values of hidden layer nodes depending after a bit is changed
    fn set<const STATE: bool>(&mut self, idx: (usize, usize)) {
        fn s<const STATE: bool>(acc: &mut SideAccumulator, idx: usize) {
            for (x, &w) in acc.iter_mut().zip(&MODEL.feature_weights[idx..idx + HL_SIZE]) {
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

    pub fn capture_update(&mut self, _piece: Piece, victim: Piece, _from: Square, to: Square) {
        //SAFETY: this is only called when there is a capture
        self.set_weight::<OFF>(victim, to);
    }

    //promotions that are also captures handled in capture_update()
    pub fn promotion_update(&mut self, piece: Piece, promotion: Option<Piece>, from: Square, to: Square) {
        self.set_weight::<OFF>(piece, from);
        self.set_weight::<ON>(unsafe { promotion.unwrap_unchecked() }, to);
    }

    pub fn ep_update(&mut self, piece: Piece, victim: Piece, _from: Square, to: Square) {
        let ep = match piece {
            Piece::WP => unsafe { to.sub_unchecked(8) },
            Piece::BP => unsafe { to.add_unchecked(8) },
            _ => unreachable!(),
        };

        self.set_weight::<OFF>(victim, ep);
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
        if victim.is_some() {
            self.set_weight::<ON>(unsafe { victim.unwrap_unchecked() }, to);
        }

        if promotion.is_some() {
            self.set_weight::<OFF>(unsafe { promotion.unwrap_unchecked() }, to);
            self.set_weight::<ON>(piece, from);
        } else {
            self.quiet_update(piece, to, from);
        }
    }

    #[must_use]
    pub fn evaluate(&self, side: Colour, bucket: usize) -> i32 {
        let (us, them) = match side {
            Colour::White => (self.white.iter(), self.black.iter()),
            Colour::Black => (self.black.iter(), self.white.iter()),
        };

        let bucket = bucket.min(OUTPUT_BUCKETS - 1);

        let weights_start = bucket * HL_SIZE * 2;
        let us_weights = &MODEL.output_weights[weights_start..weights_start + HL_SIZE];
        let them_weights = &MODEL.output_weights[weights_start + HL_SIZE..weights_start + 2 * HL_SIZE];

        let mut out = 0;

        for (&value, &weight) in us.zip(us_weights) {
            out += squared_crelu(value) * weight as i32;
        }

        for (&value, &weight) in them.zip(them_weights) {
            out += squared_crelu(value) * weight as i32;
        }

        (out / QA + MODEL.output_biases[bucket] as i32) * SCALE / QAB
    }
}

fn squared_crelu(value: i16) -> i32 {
    let v = value.clamp(CR_MIN, CR_MAX) as i32;

    v * v
}
