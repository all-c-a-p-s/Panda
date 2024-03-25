use crate::{board::Colour, helper::get_bit, r#move::Move, Board};
use lazy_static::lazy_static;

use crate::rng::random_hash_u64;

pub enum EntryFlag {
    Exact,
    Beta,
    Alpha,
}

pub struct TTEntry {
    pub depth: usize,
    pub eval: i32,
    pub flag: EntryFlag,
}

lazy_static! {
    pub static ref PIECE_KEYS: [[u64; 12]; 64] = {
        let mut res: [[u64; 12]; 64] = [[0u64; 12]; 64];
        let mut square = 0;
        while square < 64 {
            let mut piece = 0;
            while piece < 12 {
                res[square][piece] = random_hash_u64();
                piece += 1;
            }
            square += 1;
        }
        res
    };
    static ref EP_KEYS: [u64; 64] = {
        let mut res: [u64; 64] = [0u64; 64];
        let mut square = 0;
        while square < 64 {
            res[square] = random_hash_u64();
            square += 1;
        }
        res
    };
    static ref CASTLING_KEYS: [u64; 16] =  {
        let mut res: [u64; 16] = [0u64; 16];
        let mut combination = 0; //castling encoded by 4 binary bits -> 16 combinations
        while combination < 16 {
            res[combination] = random_hash_u64();
            combination += 1;
        }
        res
    };

    static ref BLACK_TO_MOVE: u64 = random_hash_u64();
}

pub fn hash(b: &Board) -> u64 {
    let mut hash_key: u64 = 0;

    for square in 0..64 {
        for i in 0..12 {
            if get_bit(square, b.bitboards[i]) == 1 {
                hash_key ^= PIECE_KEYS[square][i];
            }
        }
    }

    if let Some(k) = b.en_passant {
        hash_key ^= EP_KEYS[k];
    }

    hash_key ^= CASTLING_KEYS[b.castling as usize];

    if b.side_to_move == Colour::Black {
        hash_key ^= *BLACK_TO_MOVE;
    }

    hash_key
}

pub fn incemental_hash_update(hash_key: u64, m: &Move, b: &Board) -> u64 {
    //call with board state before move was made
    let mut res = hash_key;

    res ^= PIECE_KEYS[m.square_from()][m.piece_moved()];
    res ^= PIECE_KEYS[m.square_to()][m.piece_moved()];

    if m.piece_moved() == 5 {
        res ^= CASTLING_KEYS[b.castling as usize];
        res ^= CASTLING_KEYS[(b.castling & 0b00000011) as usize];
    } else if m.piece_moved() == 11 {
        res ^= CASTLING_KEYS[b.castling as usize];
        res ^= CASTLING_KEYS[(b.castling & 0b00001100) as usize];
    }

    if m.piece_moved() == 3 && m.square_to() == 7 && (b.castling & 0b00001000 > 0) {
        res ^= CASTLING_KEYS[0b00001000]
    } else if m.piece_moved() == 3 && m.square_to() == 0 && (b.castling & 0b00000100 > 0) {
        res ^= CASTLING_KEYS[0b00000100]
    } else if m.piece_moved() == 9 && m.square_to() == 63 && (b.castling & 0b00000010 > 0) {
        res ^= CASTLING_KEYS[0b00000010]
    } else if m.piece_moved() == 9 && m.square_to() == 56 && (b.castling & 0b00000001 > 0) {
        res ^= CASTLING_KEYS[0b00000001]
    }

    if m.is_capture() {
        //not including en passant
        for i in 0..12 {
            if get_bit(m.square_to(), b.bitboards[i]) == 1 {
                res ^= PIECE_KEYS[i][m.square_to()];
                if i == 3 && m.square_to() == 7 && (b.castling & 0b00001000 > 0) {
                    res ^= CASTLING_KEYS[0b00001000]
                } else if i == 3 && m.square_to() == 0 && (b.castling & 0b00000100 > 0) {
                    res ^= CASTLING_KEYS[0b00000100]
                } else if i == 9 && m.square_to() == 63 && (b.castling & 0b00000010 > 0) {
                    res ^= CASTLING_KEYS[0b00000010]
                } else if i == 9 && m.square_to() == 56 && (b.castling & 0b00000001 > 0) {
                    res ^= CASTLING_KEYS[0b00000001]
                }
            }
        }
    }

    if m.is_en_passant() {
        match m.piece_moved() {
            0 => {
                res ^= PIECE_KEYS[m.square_to() - 8][6];
            }
            6 => {
                res ^= PIECE_KEYS[m.square_to() + 8][0];
            }
            _ => panic!("non-pawn is capturing en passant 🤔"),
        }
    }

    if m.promoted_piece() != 15 {
        res ^= PIECE_KEYS[m.square_to()][m.piece_moved()];
        //undo operation from before (works bc XOR is its own inverse)
        res ^= PIECE_KEYS[m.square_to()][m.promoted_piece()];
    }

    res
}
