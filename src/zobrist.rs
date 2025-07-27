use crate::rng::XorShiftU64;
use crate::types::{Piece, Square};
use crate::{Board, Colour, Move};

macro_rules! cfor {
    ($init: stmt; $cond: expr; $step: expr; $body: block) => {
        {
            $init
            #[allow(while_true)]
            while $cond {
                $body;

                $step;
            }
        }
    }
}

const fn init_hash_keys() -> ([[u64; 12]; 64], [u64; 64], [u64; 16], u64) {
    let mut rng = XorShiftU64::new();
    let mut piece_keys = [[0; 12]; 64];

    cfor!(let mut sq = 0; sq < 64; sq += 1; {
        cfor!(let mut i = 0; i < 12; i += 1; {
            let r = rng.next();
            piece_keys[sq][i] = r;
        });
    });
    let mut ep_keys = [0; 64];
    cfor!(let mut sq = 0; sq < 64; sq += 1; {
        let r = rng.next();
        ep_keys[sq] = r;
    });
    let mut castling_keys = [0; 16];
    cfor!(let mut i = 0; i < 16; i += 1; {
        let r = rng.next();
        castling_keys[i] = r;
    });
    let btm = rng.next();
    (piece_keys, ep_keys, castling_keys, btm)
}

pub static PIECE_KEYS: [[u64; 12]; 64] = init_hash_keys().0;
pub static EP_KEYS: [u64; 64] = init_hash_keys().1;
pub static CASTLING_KEYS: [u64; 16] = init_hash_keys().2;
pub const BLACK_TO_MOVE: u64 = init_hash_keys().3;

#[must_use] pub fn hash(b: &Board) -> u64 {
    let mut hash_key: u64 = 0;

    for (square, &piece) in b.pieces_array.iter().enumerate() {
        if let Some(piece) = piece {
            hash_key ^= PIECE_KEYS[square][piece];
        }
    }

    if let Some(sq) = b.en_passant {
        hash_key ^= EP_KEYS[sq];
    }

    hash_key ^= CASTLING_KEYS[b.castling as usize];

    if b.side_to_move == Colour::Black {
        hash_key ^= BLACK_TO_MOVE;
    }

    hash_key
}

/// This updates everything about the hash key EXCEPT castling rights,
/// which it is more efficient to simply do after making the move
#[must_use] pub fn hash_update(hash_key: u64, m: &Move, b: &Board) -> u64 {
    //call with board state before move was made
    let mut res = hash_key;

    let sq_to = m.square_to();
    let sq_from = m.square_from();
    let piece = m.piece_moved(b);

    res ^= PIECE_KEYS[sq_from][piece];
    res ^= PIECE_KEYS[sq_to][piece];

    if let Some(sq) = b.en_passant {
        res ^= EP_KEYS[sq];
    }

    if m.is_capture(b) {
        //not including en passant
        let captured_piece = b.get_piece_at(sq_to);
        res ^= PIECE_KEYS[sq_to][captured_piece];
    }

    if m.is_castling() {
        match sq_to {
            Square::C1 => {
                res ^= PIECE_KEYS[Square::A1][Piece::WR];
                res ^= PIECE_KEYS[Square::D1][Piece::WR];
            }
            Square::G1 => {
                res ^= PIECE_KEYS[Square::H1][Piece::WR];
                res ^= PIECE_KEYS[Square::F1][Piece::WR];
            }
            Square::C8 => {
                res ^= PIECE_KEYS[Square::A8][Piece::BR];
                res ^= PIECE_KEYS[Square::D8][Piece::BR];
            }
            Square::G8 => {
                res ^= PIECE_KEYS[Square::H8][Piece::BR];
                res ^= PIECE_KEYS[Square::F8][Piece::BR];
            }
            _ => unreachable!(),
        }
    }

    if m.is_en_passant() {
        match piece {
            Piece::WP => {
                res ^= PIECE_KEYS[unsafe { sq_to.sub_unchecked(8) }][Piece::BP];
            }
            Piece::BP => {
                res ^= PIECE_KEYS[unsafe { sq_to.add_unchecked(8) }][Piece::WP];
            }
            _ => unreachable!(),
        }
    }

    if m.is_promotion() {
        res ^= PIECE_KEYS[sq_to][piece];
        //undo operation from before (works bc XOR is its own inverse)
        let promoted_piece = match piece {
            Piece::WP => m.promoted_piece().to_white_piece(),
            Piece::BP => m.promoted_piece().to_white_piece().opposite(), //only type is encoded in the move
            _ => unreachable!(),
        };
        res ^= PIECE_KEYS[sq_to][promoted_piece];
    }

    if m.is_double_push(b) {
        match piece {
            Piece::WP => res ^= EP_KEYS[unsafe { sq_to.sub_unchecked(8) }],
            Piece::BP => res ^= EP_KEYS[unsafe { sq_to.add_unchecked(8) }],
            _ => unreachable!(),
        }
    }

    res ^= BLACK_TO_MOVE;
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    pub fn hash_update_test(depth: usize, b: &mut Board) -> usize {
        let hash_before_move = hash(b);

        let mut total = 0;
        let moves = MoveList::gen_legal(b);
        let mut added = 0;
        for i in 0..MAX_MOVES {
            if moves.moves[i].is_null() {
                if depth == 1 {
                    return i;
                }
                break;
            }
            if depth != 1 {
                let mut updated_hash = hash_update(hash_before_move, &moves.moves[i], b);
                let castling_rights_before = b.castling;
                //must be done before making move
                let Ok(commit) = b.try_move(moves.moves[i]) else {
                    panic!("invalid move {}", moves.moves[i].uci());
                };

                updated_hash ^= CASTLING_KEYS[castling_rights_before as usize];
                updated_hash ^= CASTLING_KEYS[b.castling as usize];

                let hash_after_move = hash(b);

                if updated_hash != hash_after_move {
                    b.print_board();
                    moves.moves[i].print_move();
                    panic!("hash update failed");
                }

                added = hash_update_test(depth - 1, b);
                b.undo_move(moves.moves[i], &commit);
            }
            total += added;
        }

        total
    }

    macro_rules! hasht {
        ($fen: expr, $depth: expr, $tgt: expr, $idx: expr) => {
            let mut b = Board::from($fen);
            let res = hash_update_test($depth, &mut b);
            assert_eq!(res, $tgt);
            println!("Hash Test {}: Passed", $idx);
        };
    }

    #[test]
    #[rustfmt::skip]
    pub fn full_hash_test() {
        init_all();
        let start = Instant::now();

        hasht!(STARTPOS, 5, 4_865_609, 1);
        hasht!("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690, 2);
        hasht!("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674_624, 3);
        hasht!("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5, 15_833_292, 4);
        hasht!("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194, 5);
        hasht!("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551, 6);
        hasht!("2bqr3/rp2ppbk/2p2np1/p1Pp3p/N2P1B1P/6P1/PPQRPPB1/4R1K1 b - - 8 21", 5, 40_034_887, 7);
        hasht!("r2q1rk1/1p2npb1/p1n1b1pp/2ppp3/PP2P3/2PP1N1P/2N2PP1/R1BQRBK1 b - b3 0 13", 5, 74_778_465, 8);
        hasht!("4k3/8/8/8/8/8/8/4K2R w K - 0 1", 6, 764_643, 9);
        hasht!("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", 6, 179_862_938, 10);
        hasht!("4k3/8/8/8/8/8/8/4K2R b K - 0 1", 6, 899_442, 11);
        hasht!("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", 6, 199_002, 12);
        hasht!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", 6, 37_665_329, 13);
        hasht!("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", 6, 28_859_283, 14);
        hasht!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", 6, 71_179_139, 15);
        hasht!("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", 6, 1_030_499, 16);
        hasht!("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", 6, 37_665_329, 17);
        hasht!("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6, 71_179_139, 18);
        hasht!("k7/8/3p4/8/8/4P3/8/7K w - - 0 1", 6, 28_662, 19);
        hasht!("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", 6, 157_093, 20);
        hasht!("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", 6, 3_370_175, 21);
        hasht!("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", 6, 524_966_748, 22);
        hasht!("7k/RR6/8/8/8/8/rr6/7K w - - 0 1", 6, 44_956_585, 23);
        hasht!("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", 6, 22_823_890, 24);
        hasht!("8/4K3/5P2/1p6/4N2p/1k3n2/6p1/8 w - - 0 53", 6, 19_350_596, 25);

        let duration: Duration = start.elapsed();
        println!("Hash Test completed in: {duration:?}");
    }
}
