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
                let lsb = lsfb(bitboard);
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
                let lsb = lsfb(bitboard);
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
            let lsb = lsfb(bitboard); // never panics as loop will have already exited
            let mut attacks = match i {
                0 => {
                    WP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[1],
                            k => set_bit(k, board.occupancies[1]),
                        }
                } //en passant capture
                6 => {
                    BP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[0],
                            k => set_bit(k, board.occupancies[0]),
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
                let lsb_attack = lsfb(attacks);
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
            let lsb = lsfb(bitboard); // never panics as loop will have already exited
            let mut attacks = match i {
                0 => {
                    WP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[1],
                            k => set_bit(k, board.occupancies[1]),
                        }
                } //en passant capture
                6 => {
                    BP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[0],
                            k => set_bit(k, board.occupancies[0]),
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
                let lsb_attack = lsfb(attacks);
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

/* I don't think checking all edge cases separately is actually faster
 but code to detect pins mught be useful in the future
pub fn get_pin_rays(b: &Board) -> [u64; 8] {
    let mut res = [0u64; 8];
    match b.side_to_move {
        Colour::White => {
            let mut ray_count: usize = 0;
            let king_square = lsfb(b.bitboards[5]);
            let rook_rays = get_rook_attacks(king_square, b.occupancies[2]);
            let bishop_rays = get_bishop_attacks(king_square, b.occupancies[2]);
            //rays from king square

            let mut enemy_bishop_rays = 0u64;
            let mut enemy_rook_rays = 0u64;
            let mut enemy_bishops = b.bitboards[8] | b.bitboards[10];
            let mut enemy_rooks = b.bitboards[9] | b.bitboards[10];

            while enemy_bishops > 0 {
                let square = lsfb(enemy_bishops);
                enemy_bishop_rays |= get_bishop_attacks(square, b.occupancies[2]);
                enemy_bishops = pop_bit(square, enemy_bishops);
            }

            while enemy_rooks > 0 {
                let square = lsfb(enemy_rooks);
                enemy_rook_rays |= get_rook_attacks(square, b.occupancies[2]);
                enemy_rooks = pop_bit(square, enemy_rooks);
            }
            //generate rays for enemy slider attacks

            let mut pinned_by_bishop = bishop_rays & enemy_bishop_rays;
            let mut pinned_by_rook = rook_rays & enemy_rook_rays;
            //find pinned pieces

            while pinned_by_bishop > 0 {
                let pinned_square = lsfb(pinned_by_bishop);
                let ray = get_bishop_attacks(king_square, pop_bit(pinned_square, b.occupancies[2]))
                    & get_bishop_attacks(pinned_square, b.occupancies[2]);
                //ray corresponding to each pinned piece
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_bishop = pop_bit(pinned_square, pinned_by_bishop);
            }

            while pinned_by_rook > 0 {
                let pinned_square = lsfb(pinned_by_rook);
                let ray = get_rook_attacks(king_square, pop_bit(pinned_square, b.occupancies[2]))
                    & get_rook_attacks(pinned_square, b.occupancies[2]);
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_rook = pop_bit(pinned_square, pinned_by_rook);
            }
        }
        Colour::Black => {
            let mut ray_count: usize = 0;
            let king_square = lsfb(b.bitboards[11]);
            let rook_rays = get_rook_attacks(king_square, b.occupancies[2]);
            let bishop_rays = get_bishop_attacks(king_square, b.occupancies[2]);

            let mut enemy_bishop_rays = 0u64;
            let mut enemy_rook_rays = 0u64;
            let mut enemy_bishops = b.bitboards[2] | b.bitboards[4];
            let mut enemy_rooks = b.bitboards[3] | b.bitboards[4];

            while enemy_bishops > 0 {
                let square = lsfb(enemy_bishops);
                enemy_bishop_rays |= get_bishop_attacks(square, b.occupancies[2]);
                enemy_bishops = pop_bit(square, enemy_bishops);
            }

            while enemy_rooks > 0 {
                let square = lsfb(enemy_rooks);
                enemy_rook_rays |= get_rook_attacks(square, b.occupancies[2]);
                enemy_rooks = pop_bit(square, enemy_rooks);
            }

            let mut pinned_by_bishop = bishop_rays & enemy_bishop_rays;
            let mut pinned_by_rook = rook_rays & enemy_rook_rays;

            while pinned_by_bishop > 0 {
                let pinned_square = lsfb(pinned_by_bishop);
                let ray = get_bishop_attacks(king_square, pop_bit(pinned_square, b.occupancies[2]))
                    & get_bishop_attacks(pinned_square, b.occupancies[2]);
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_bishop = pop_bit(pinned_square, pinned_by_bishop);
            }

            while pinned_by_rook > 0 {
                let pinned_square = lsfb(pinned_by_rook);
                let ray = get_rook_attacks(king_square, pop_bit(pinned_square, b.occupancies[2]))
                    & get_rook_attacks(pinned_square, b.occupancies[2]);
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_rook = pop_bit(pinned_square, pinned_by_rook);
            }
        }
    }
    res
}

pub fn check_en_passant(m: Move, b: &Board) -> bool {
    //checks en passant edge case where en passant reveals check on the king
    match m.piece_moved() {
        0 => {
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[2]);
            relevant_blockers = pop_bit(m.square_to() - 8, relevant_blockers);
            get_rook_attacks(lsfb(b.bitboards[5]), relevant_blockers)
                & (b.bitboards[9] | b.bitboards[10])
                == 0
        }
        6 => {
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[2]);
            relevant_blockers = pop_bit(m.square_to() + 8, relevant_blockers);
            get_rook_attacks(lsfb(b.bitboards[11]), relevant_blockers)
                & (b.bitboards[3] | b.bitboards[4])
                == 0
        }
        _ => panic!("impossible"),
    }
}

pub fn legal_non_check_evasion(m: Move, b: &Board) -> bool {
    //separate function used to generate check evasions
    let pin_rays = get_pin_rays(b);
    for r in pin_rays {
        //check that not moving pinned piece out of pin ray
        if set_bit(m.square_from(), 0) & r > 0 && set_bit(m.square_to(), 0) & r == 0 {
            return false;
        }
    }

    if m.piece_moved() == 5 {
        //check that king isn't moving into check
        let mut black_attacks = 0u64;
        for i in 6..12 {
            let mut piece_bb = b.bitboards[i];
            let relevant_blockers = b.occupancies[2] ^ b.bitboards[5];
            //king blocking slider attacks doesn't count because it can move back
            //into a new attack from the same slider
            while piece_bb > 0 {
                let sq = lsfb(b.bitboards[i]);
                black_attacks |= match i {
                    6 => BP_ATTACKS[sq],
                    7 => N_ATTACKS[sq],
                    8 => get_bishop_attacks(sq, relevant_blockers),
                    9 => get_rook_attacks(sq, relevant_blockers),
                    10 => get_queen_attacks(sq, relevant_blockers),
                    11 => K_ATTACKS[sq],
                    _ => panic!("impossible"),
                };
                piece_bb = pop_bit(sq, piece_bb);
            }
        }
        return (m.square_to() as u64) & black_attacks == 0;
    } else if m.piece_moved() == 11 {
        let mut white_attacks = 0u64;
        for i in 0..6 {
            let mut piece_bb = b.bitboards[i];
            let relevant_blockers = b.occupancies[2] ^ b.bitboards[11];
            //king blocking slider attacks doesn't count because it can move back
            //into a new attack from the same slider
            while piece_bb > 0 {
                let sq = lsfb(b.bitboards[i]);
                white_attacks |= match i {
                    0 => BP_ATTACKS[sq],
                    1 => N_ATTACKS[sq],
                    2 => get_bishop_attacks(sq, relevant_blockers),
                    3 => get_rook_attacks(sq, relevant_blockers),
                    4 => get_queen_attacks(sq, relevant_blockers),
                    5 => K_ATTACKS[sq],
                    _ => panic!("impossible"),
                };
                piece_bb = pop_bit(sq, piece_bb);
            }
        }
        return (m.square_to() as u64) & white_attacks == 0;
    } else if m.is_en_passant() {
        //special case where en passant capture creates removes 2 pawns from 1 rank -> discovered check
        return check_en_passant(m, b);
    }
    true
}
*/

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
