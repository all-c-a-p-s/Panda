use crate::rng::XorShiftU64;
use crate::*;

use crate::helper::*;

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

#[allow(unused_mut)]
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
    let key = rng.next();
    let btm = key;
    (piece_keys, ep_keys, castling_keys, btm)
}

pub static PIECE_KEYS: [[u64; 12]; 64] = init_hash_keys().0;
pub static EP_KEYS: [u64; 64] = init_hash_keys().1;
pub static CASTLING_KEYS: [u64; 16] = init_hash_keys().2;
pub const BLACK_TO_MOVE: u64 = init_hash_keys().3;

pub fn hash(b: &Board) -> u64 {
    let mut hash_key: u64 = 0;

    for square in 0..64 {
        let piece = b.pieces_array[square];
        if piece != NO_PIECE {
            hash_key ^= PIECE_KEYS[square][piece];
        }
    }

    if b.en_passant != NO_SQUARE {
        hash_key ^= EP_KEYS[b.en_passant];
    }

    hash_key ^= CASTLING_KEYS[b.castling as usize];

    if b.side_to_move == Colour::Black {
        hash_key ^= BLACK_TO_MOVE;
    }

    hash_key
}

pub fn hash_update(hash_key: u64, m: &Move, b: &Board) -> u64 {
    //call with board state before move was made
    let mut res = hash_key;

    let sq_to = m.square_to();
    let sq_from = m.square_from();
    let piece = m.piece_moved(&b);

    res ^= PIECE_KEYS[sq_from][piece];
    res ^= PIECE_KEYS[sq_to][piece];

    if b.en_passant != NO_SQUARE {
        res ^= EP_KEYS[b.en_passant];
    }

    if piece == WK {
        res ^= CASTLING_KEYS[b.castling as usize];
        res ^= CASTLING_KEYS[(b.castling & 0b00001100) as usize];
    } else if piece == BK {
        res ^= CASTLING_KEYS[b.castling as usize];
        res ^= CASTLING_KEYS[(b.castling & 0b00000011) as usize];
    }

    if piece == WR && sq_from == H1 && (b.castling & 0b00000001 > 0) {
        res ^= CASTLING_KEYS[b.castling as usize];
        let new_castling = b.castling ^ 0b00000001;
        res ^= CASTLING_KEYS[new_castling as usize];
    } else if piece == WR && sq_from == A1 && (b.castling & 0b00000010 > 0) {
        res ^= CASTLING_KEYS[b.castling as usize];
        let new_castling = b.castling ^ 0b00000010;
        res ^= CASTLING_KEYS[new_castling as usize];
    } else if piece == BR && sq_from == H8 && (b.castling & 0b00000100 > 0) {
        res ^= CASTLING_KEYS[b.castling as usize];
        let new_castling = b.castling ^ 0b00000100;
        res ^= CASTLING_KEYS[new_castling as usize];
    } else if piece == BR && sq_from == A8 && (b.castling & 0b00001000 > 0) {
        res ^= CASTLING_KEYS[b.castling as usize];
        let new_castling = b.castling ^ 0b00001000;
        res ^= CASTLING_KEYS[new_castling as usize];
    }

    if m.is_capture(b) {
        //not including en passant
        let captured_piece = b.pieces_array[sq_to];
        res ^= PIECE_KEYS[sq_to][captured_piece];
        if captured_piece == WR && sq_to == H1 && (b.castling & 0b00000001 > 0) {
            res ^= CASTLING_KEYS[b.castling as usize];
            let new_castling = b.castling ^ 0b00000001;
            res ^= CASTLING_KEYS[new_castling as usize];
        } else if captured_piece == WR && sq_to == A1 && (b.castling & 0b00000010 > 0) {
            res ^= CASTLING_KEYS[b.castling as usize];
            let new_castling = b.castling ^ 0b00000010;
            res ^= CASTLING_KEYS[new_castling as usize];
        } else if captured_piece == BR && sq_to == H8 && (b.castling & 0b000000100 > 0) {
            res ^= CASTLING_KEYS[b.castling as usize];
            let new_castling = b.castling ^ 0b00000100;
            res ^= CASTLING_KEYS[new_castling as usize];
        } else if captured_piece == BR && sq_to == A8 && (b.castling & 0b00001000 > 0) {
            res ^= CASTLING_KEYS[b.castling as usize];
            let new_castling = b.castling ^ 0b00001000;
            res ^= CASTLING_KEYS[new_castling as usize];
        }
    }

    if m.is_castling() {
        match sq_to {
            C1 => {
                res ^= PIECE_KEYS[A1][WR];
                res ^= PIECE_KEYS[D1][WR];
            }
            G1 => {
                res ^= PIECE_KEYS[H1][WR];
                res ^= PIECE_KEYS[F1][WR];
            }
            C8 => {
                res ^= PIECE_KEYS[A8][BR];
                res ^= PIECE_KEYS[D8][BR];
            }
            G8 => {
                res ^= PIECE_KEYS[H8][BR];
                res ^= PIECE_KEYS[F8][BR];
            }
            _ => unreachable!(),
        }
    }

    if m.is_en_passant() {
        match piece {
            WP => {
                res ^= PIECE_KEYS[sq_to - 8][BP];
            }
            BP => {
                res ^= PIECE_KEYS[sq_to + 8][WP];
            }
            _ => unreachable!(),
        }
    }

    if m.is_promotion() {
        res ^= PIECE_KEYS[sq_to][piece];
        //undo operation from before (works bc XOR is its own inverse)
        let promoted_piece = match piece {
            WP => m.promoted_piece(),
            BP => m.promoted_piece() + 6, //only type is encoded in the move
            _ => unreachable!(),
        };
        res ^= PIECE_KEYS[sq_to][promoted_piece];
    }

    if m.is_double_push(&b) {
        match piece {
            WP => res ^= EP_KEYS[sq_to - 8],
            BP => res ^= EP_KEYS[sq_to + 8],
            _ => unreachable!(),
        }
    }

    res ^= BLACK_TO_MOVE;
    res
}

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
            let updated_hash = hash_update(hash_before_move, &moves.moves[i], b);
            //must be done before making move
            let Ok(commit) = b.try_move(moves.moves[i]) else {
                panic!("invalid move {}", moves.moves[i].uci());
            };

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

pub fn full_hash_test() {
    let res1 = hash_update_test(5, &mut Board::from(STARTPOS));
    assert_eq!(res1, 4_865_609);
    println!("Test 1 - Passed");
    let res2 = hash_update_test(
        5,
        &mut Board::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
    );
    assert_eq!(res2, 193_690_690);
    println!("Test 2 - Passed");
    let res3 = hash_update_test(
        5,
        &mut Board::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    );
    assert_eq!(res3, 674_624);
    println!("Test 3 - Passed");
    let res4 = hash_update_test(
        5,
        &mut Board::from("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
    );
    assert_eq!(res4, 15_833_292);
    println!("Test 4 - Passed");
    let res5 = hash_update_test(
        5,
        &mut Board::from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
    );
    assert_eq!(res5, 89_941_194);
    println!("Test 5 - Passed");
}
