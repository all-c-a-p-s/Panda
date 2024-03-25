use crate::board::*;
use crate::helper::*;
use crate::magic::*;
use crate::r#move::*;

pub fn is_attacked(square: usize, colour: Colour, board: &Board) -> bool {
    //attacked BY colour
    if colour == Colour::White {
        //leapers
        if BP_ATTACKS[square] & board.bitboards[0] != 0
            || N_ATTACKS[square] & board.bitboards[1] != 0
            || K_ATTACKS[square] & board.bitboards[5] != 0
        {
            return true;
        }
        //sliders
        if get_bishop_attacks(square, board.occupancies[2]) & board.bitboards[2] != 0
            || get_rook_attacks(square, board.occupancies[2]) & board.bitboards[3] != 0
            || get_queen_attacks(square, board.occupancies[2]) & board.bitboards[4] != 0
        {
            return true;
        }
    } else {
        //leapers
        if WP_ATTACKS[square] & board.bitboards[6] != 0
            || N_ATTACKS[square] & board.bitboards[7] != 0
            || K_ATTACKS[square] & board.bitboards[11] != 0
        {
            return true;
        }
        //sliders
        if get_bishop_attacks(square, board.occupancies[2]) & board.bitboards[8] != 0
            || get_rook_attacks(square, board.occupancies[2]) & board.bitboards[9] != 0
            || get_queen_attacks(square, board.occupancies[2]) & board.bitboards[10] != 0
        {
            return true;
        }
    }
    false
}

pub fn pawn_push_moves(board: &Board, moves: MoveList) -> MoveList {
    let mut first_unused: usize = 0;
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            first_unused = i;
            break;
        }
    }
    let mut res = moves;
    match board.side_to_move {
        Colour::White => {
            let mut bitboard = board.bitboards[0];
            while bitboard > 0 {
                let lsb = lsfb(bitboard).unwrap();
                if get_bit(lsb + 8, board.occupancies[2]) == 0 {
                    if rank(lsb) == 6 {
                        //promotion
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, 4, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, 3, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, 2, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, 1, board, false);
                        first_unused += 1;
                    } else {
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, 15, board, false);
                        first_unused += 1;
                    }
                    if rank(lsb) == 1 && get_bit(lsb + 16, board.occupancies[2]) == 0 {
                        //double push
                        res.moves[first_unused] = encode_move(lsb, lsb + 16, 15, board, false);
                        first_unused += 1;
                    }
                }
                bitboard = pop_bit(lsb, bitboard);
            }
        }
        Colour::Black => {
            let mut bitboard = board.bitboards[6];
            while bitboard > 0 {
                let lsb = lsfb(bitboard).unwrap();
                if get_bit(lsb - 8, board.occupancies[2]) == 0 {
                    if rank(lsb - 8) == 0 {
                        //promotion
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, 10, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, 9, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, 8, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, 7, board, false);
                        first_unused += 1;
                    } else {
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, 15, board, false);
                        first_unused += 1;
                    }
                    if rank(lsb) == 6 && get_bit(lsb - 16, board.occupancies[2]) == 0 {
                        //double push
                        res.moves[first_unused] = encode_move(lsb, lsb - 16, 15, board, false);
                        first_unused += 1;
                    }
                }
                bitboard = pop_bit(lsb, bitboard);
            }
        }
    };
    res
}

pub fn castling_moves(board: &Board, moves: MoveList) -> MoveList {
    let mut first_unused: usize = 0;
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            first_unused = i;
            break;
        }
    }
    let mut res = moves;

    match board.side_to_move {
        Colour::White => {
            if (board.castling & 0b0000_0001) > 0 {
                //white kingside castling rights
                if get_bit(6, board.occupancies[2]) == 0
                    && get_bit(5, board.occupancies[2]) == 0
                    && !is_attacked(4, Colour::Black, board)
                    && !is_attacked(5, Colour::Black, board)
                {
                    res.moves[first_unused] = encode_move(4, 6, 15, board, true);
                    first_unused += 1;
                }
            }

            if (board.castling & 0b0000_0010) > 0 {
                //white queenside
                if get_bit(1, board.occupancies[2]) == 0
                    && get_bit(2, board.occupancies[2]) == 0
                    && get_bit(3, board.occupancies[2]) == 0
                    && !is_attacked(4, Colour::Black, board)
                    && !is_attacked(3, Colour::Black, board)
                {
                    res.moves[first_unused] = encode_move(4, 2, 15, board, true);
                }
            }
        }
        Colour::Black => {
            if (board.castling & 0b0000_0100) > 0 {
                //black kingside
                if get_bit(62, board.occupancies[2]) == 0
                    && get_bit(61, board.occupancies[2]) == 0
                    && !is_attacked(60, Colour::White, board)
                    && !is_attacked(61, Colour::White, board)
                {
                    res.moves[first_unused] = encode_move(60, 62, 15, board, true);
                    first_unused += 1;
                }
            }

            if (board.castling & 0b0000_1000) > 0 {
                //black queenside
                if get_bit(57, board.occupancies[2]) == 0
                    && get_bit(58, board.occupancies[2]) == 0
                    && get_bit(59, board.occupancies[2]) == 0
                    && !is_attacked(60, Colour::White, board)
                    && !is_attacked(59, Colour::White, board)
                {
                    res.moves[first_unused] = encode_move(60, 58, 15, board, true);
                }
            }
        }
    };
    res
}

pub fn gen_moves(board: &Board) -> MoveList {
    let (mut min, mut max) = (0usize, 6usize);
    if board.side_to_move == Colour::Black {
        min = 6;
        max = 12;
    }

    let mut moves = MoveList {
        moves: [NULL_MOVE; 218],
    };

    moves = pawn_push_moves(board, moves);
    moves = castling_moves(board, moves);

    let mut first_unused: usize = 0;
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            first_unused = i;
            break;
        }
    }

    for i in min..max {
        //pieces of colour to move
        let mut bitboard = board.bitboards[i];

        while bitboard > 0 {
            let lsb = lsfb(bitboard).unwrap(); // never panics as loop will have already exited
            let mut attacks = match i {
                0 => {
                    WP_ATTACKS[lsb]
                        & match board.en_passant {
                            None => board.occupancies[1],
                            Some(k) => set_bit(k, board.occupancies[1]),
                        }
                } //en passant capture
                6 => {
                    BP_ATTACKS[lsb]
                        & match board.en_passant {
                            None => board.occupancies[0],
                            Some(k) => set_bit(k, board.occupancies[0]),
                        }
                } //or with set en passant square if it is not 64 i.e. none
                1 | 7 => N_ATTACKS[lsb],
                2 | 8 => get_bishop_attacks(lsb, board.occupancies[2]),
                3 | 9 => get_rook_attacks(lsb, board.occupancies[2]),
                4 | 10 => get_queen_attacks(lsb, board.occupancies[2]),
                5 | 11 => K_ATTACKS[lsb],
                _ => panic!("this is impossible"),
            };
            match board.side_to_move {
                Colour::White => attacks &= !board.occupancies[0], //remove attacks on own pieces
                Colour::Black => attacks &= !board.occupancies[1],
            }
            while attacks > 0 {
                let lsb_attack = lsfb(attacks).unwrap();
                if (get_bit(lsb, board.bitboards[0]) > 0) && rank(lsb) == 6 {
                    // white promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 4, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 3, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 2, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 1, board, false);
                    first_unused += 1;
                } else if (get_bit(lsb, board.bitboards[6]) > 0) && rank(lsb) == 1 {
                    //black promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 10, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 9, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 8, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 7, board, false);
                    first_unused += 1;
                } else {
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 15, board, false);
                    first_unused += 1;
                } //list to return here
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }
    }
    moves
}

pub fn gen_captures(board: &mut Board) -> MoveList {
    //special capture-only move generation for quiescence search
    let (mut min, mut max) = (0usize, 6usize);
    if board.side_to_move == Colour::Black {
        min = 6;
        max = 12;
    }

    let mut moves = MoveList {
        moves: [NULL_MOVE; 218],
    };
    let mut first_unused = 0;
    for i in min..max {
        //pieces of colour to move
        let mut bitboard = board.bitboards[i];

        while bitboard > 0 {
            let lsb = lsfb(bitboard).unwrap(); // never panics as loop will have already exited
            let mut attacks = match i {
                0 => {
                    WP_ATTACKS[lsb]
                        & match board.en_passant {
                            None => board.occupancies[1],
                            Some(k) => set_bit(k, board.occupancies[1]),
                        }
                } //en passant capture
                6 => {
                    BP_ATTACKS[lsb]
                        & match board.en_passant {
                            None => board.occupancies[0],
                            Some(k) => set_bit(k, board.occupancies[0]),
                        }
                } //or with set en passant square if it is not 64 i.e. none
                1 | 7 => N_ATTACKS[lsb],
                2 | 8 => get_bishop_attacks(lsb, board.occupancies[2]),
                3 | 9 => get_rook_attacks(lsb, board.occupancies[2]),
                4 | 10 => get_queen_attacks(lsb, board.occupancies[2]),
                5 | 11 => K_ATTACKS[lsb],
                _ => panic!("this is impossible"),
            };
            match board.side_to_move {
                Colour::White => attacks &= board.occupancies[1], //only captures
                Colour::Black => attacks &= board.occupancies[0],
            }
            while attacks > 0 {
                let lsb_attack = lsfb(attacks).unwrap();
                if (get_bit(lsb, board.bitboards[0]) > 0) && rank(lsb) == 6 {
                    // white promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 4, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 3, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 2, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 1, board, false);
                    first_unused += 1;
                } else if (get_bit(lsb, board.bitboards[6]) > 0) && rank(lsb) == 1 {
                    //black promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 10, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 9, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 8, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 7, board, false);
                    first_unused += 1;
                } else {
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, 15, board, false);
                    first_unused += 1;
                } //list to return here
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }
    }
    let mut legal = MoveList {
        moves: [NULL_MOVE; MAX_MOVES],
    };
    let mut last = 0;
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        if is_legal(moves.moves[i], board) {
            legal.moves[last] = moves.moves[i];
            last += 1;
        }
    }
    legal
}

pub fn gen_legal(b: &mut Board) -> MoveList {
    let pseudo_legal = gen_moves(b);
    let mut legal = MoveList {
        moves: [NULL_MOVE; MAX_MOVES],
    };
    let mut last = 0;
    for i in 0..MAX_MOVES {
        if pseudo_legal.moves[i] == NULL_MOVE {
            break;
        }
        if is_legal(pseudo_legal.moves[i], b) {
            legal.moves[last] = pseudo_legal.moves[i];
            last += 1;
        }
    }
    legal
}
