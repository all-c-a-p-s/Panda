use crate::board::*;
use crate::helper::*;
use crate::magic::*;
use crate::r#move::*;

pub fn is_attacked(square: usize, colour: Colour, board: &Board) -> bool {
    //attacked BY colour
    match colour {
        Colour::White => {
            //leapers then sliders
            BP_ATTACKS[square] & board.bitboards[WP] != 0
                || N_ATTACKS[square] & board.bitboards[WN] != 0
                || K_ATTACKS[square] & board.bitboards[WK] != 0
                || get_bishop_attacks(square, board.occupancies[BOTH]) & board.bitboards[WB] != 0
                || get_rook_attacks(square, board.occupancies[BOTH]) & board.bitboards[WR] != 0
                || get_queen_attacks(square, board.occupancies[BOTH]) & board.bitboards[WQ] != 0
        }
        Colour::Black => {
            //leapers then sliders
            WP_ATTACKS[square] & board.bitboards[BP] != 0
                || N_ATTACKS[square] & board.bitboards[BN] != 0
                || K_ATTACKS[square] & board.bitboards[BK] != 0
                || get_bishop_attacks(square, board.occupancies[BOTH]) & board.bitboards[BB] != 0
                || get_rook_attacks(square, board.occupancies[BOTH]) & board.bitboards[BR] != 0
                || get_queen_attacks(square, board.occupancies[BOTH]) & board.bitboards[BQ] != 0
        }
    }
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
            let mut bitboard = board.bitboards[WP];
            while bitboard > 0 {
                let lsb = lsfb(bitboard);
                if get_bit(lsb + 8, board.occupancies[BOTH]) == 0 {
                    if rank(lsb) == 6 {
                        //promotion
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, WQ, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, WR, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, WB, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, WN, board, false);
                        first_unused += 1;
                        //add all different possible promotions to move list
                    } else {
                        //regular pawn push
                        res.moves[first_unused] = encode_move(lsb, lsb + 8, NO_PIECE, board, false);
                        first_unused += 1;
                    }
                    if rank(lsb) == 1 && get_bit(lsb + 16, board.occupancies[BOTH]) == 0 {
                        //double push (we already know that lsb+8 is not occupied)
                        res.moves[first_unused] =
                            encode_move(lsb, lsb + 16, NO_PIECE, board, false);
                        first_unused += 1;
                    }
                }
                bitboard = pop_bit(lsb, bitboard);
                //pop pawns from bitboard
            }
        }
        Colour::Black => {
            let mut bitboard = board.bitboards[BP];
            while bitboard > 0 {
                let lsb = lsfb(bitboard);
                if get_bit(lsb - 8, board.occupancies[BOTH]) == 0 {
                    if rank(lsb - 8) == 0 {
                        //promotion
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, BQ, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, BR, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, BB, board, false);
                        first_unused += 1;
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, BN, board, false);
                        first_unused += 1;
                    } else {
                        res.moves[first_unused] = encode_move(lsb, lsb - 8, NO_PIECE, board, false);
                        first_unused += 1;
                    }
                    if rank(lsb) == 6 && get_bit(lsb - 16, board.occupancies[BOTH]) == 0 {
                        //double push
                        res.moves[first_unused] =
                            encode_move(lsb, lsb - 16, NO_PIECE, board, false);
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
                if get_bit(6, board.occupancies[BOTH]) == 0
                    && get_bit(5, board.occupancies[BOTH]) == 0
                    && !is_attacked(4, Colour::Black, board)
                    && !is_attacked(5, Colour::Black, board)
                {
                    res.moves[first_unused] = encode_move(4, 6, NO_PIECE, board, true);
                    first_unused += 1;
                }
            }

            if (board.castling & 0b0000_0010) > 0 {
                //white queenside
                if get_bit(1, board.occupancies[BOTH]) == 0
                    && get_bit(2, board.occupancies[BOTH]) == 0
                    && get_bit(3, board.occupancies[BOTH]) == 0
                    && !is_attacked(4, Colour::Black, board)
                    && !is_attacked(3, Colour::Black, board)
                {
                    res.moves[first_unused] = encode_move(4, 2, NO_PIECE, board, true);
                }
            }
        }
        Colour::Black => {
            if (board.castling & 0b0000_0100) > 0 {
                //black kingside
                if get_bit(62, board.occupancies[BOTH]) == 0
                    && get_bit(61, board.occupancies[BOTH]) == 0
                    && !is_attacked(60, Colour::White, board)
                    && !is_attacked(61, Colour::White, board)
                {
                    res.moves[first_unused] = encode_move(60, 62, 15, board, true);
                    first_unused += 1;
                }
            }

            if (board.castling & 0b0000_1000) > 0 {
                //black queenside
                if get_bit(57, board.occupancies[BOTH]) == 0
                    && get_bit(58, board.occupancies[BOTH]) == 0
                    && get_bit(59, board.occupancies[BOTH]) == 0
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
    let (mut min, mut max) = (WP, BP);
    if board.side_to_move == Colour::Black {
        min = BP;
        max = 12;
    }

    let mut moves = MoveList {
        moves: [NULL_MOVE; MAX_MOVES],
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

    for piece in min..max {
        //pieces of colour to move
        let mut bitboard = board.bitboards[piece];

        while bitboard > 0 {
            let lsb = lsfb(bitboard); // never panics as loop will have already exited
            let mut attacks = match piece {
                WP => {
                    WP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[BLACK],
                            k => set_bit(k, board.occupancies[BLACK]),
                        }
                } //en passant capture
                BP => {
                    BP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[WHITE],
                            k => set_bit(k, board.occupancies[WHITE]),
                        }
                } //or with set en passant square if it is not 64 i.e. none
                WN | BN => N_ATTACKS[lsb],
                WB | BB => get_bishop_attacks(lsb, board.occupancies[BOTH]),
                WR | BR => get_rook_attacks(lsb, board.occupancies[BOTH]),
                WQ | BQ => get_queen_attacks(lsb, board.occupancies[BOTH]),
                WK | BK => K_ATTACKS[lsb],
                _ => panic!("this is impossible"),
            };
            match board.side_to_move {
                Colour::White => attacks &= !board.occupancies[WHITE], //remove attacks on own pieces
                Colour::Black => attacks &= !board.occupancies[BLACK],
            }
            while attacks > 0 {
                let lsb_attack = lsfb(attacks);
                //promotions that are also captures
                if (get_bit(lsb, board.bitboards[WP]) > 0) && rank(lsb) == 6 {
                    // white promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WQ, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WR, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WB, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WN, board, false);
                    first_unused += 1;
                } else if (get_bit(lsb, board.bitboards[BP]) > 0) && rank(lsb) == 1 {
                    //black promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BQ, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BR, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BB, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BN, board, false);
                    first_unused += 1;
                } else {
                    moves.moves[first_unused] =
                        encode_move(lsb, lsb_attack, NO_PIECE, board, false);
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
    let (mut min, mut max) = (WP, BP);
    if board.side_to_move == Colour::Black {
        min = BP;
        max = 12;
    }

    let mut moves = MoveList {
        moves: [NULL_MOVE; MAX_MOVES],
    };
    let mut first_unused = 0;
    for piece in min..max {
        //pieces of colour to move
        let mut bitboard = board.bitboards[piece];

        while bitboard > 0 {
            let lsb = lsfb(bitboard); // never panics as loop will have already exited
            let mut attacks = match piece {
                WP => {
                    WP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[BLACK],
                            k => set_bit(k, board.occupancies[BLACK]),
                        }
                } //en passant capture
                BP => {
                    BP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[WHITE],
                            k => set_bit(k, board.occupancies[WHITE]),
                        }
                } //or with set en passant square if it is not 64 i.e. none
                WN | BN => N_ATTACKS[lsb],
                WB | BB => get_bishop_attacks(lsb, board.occupancies[BOTH]),
                WR | BR => get_rook_attacks(lsb, board.occupancies[BOTH]),
                WQ | BQ => get_queen_attacks(lsb, board.occupancies[BOTH]),
                WK | BK => K_ATTACKS[lsb],
                _ => panic!("this is impossible"),
            };
            match board.side_to_move {
                Colour::White => attacks &= board.occupancies[BLACK], //only captures
                Colour::Black => attacks &= board.occupancies[WHITE],
            }
            while attacks > 0 {
                let lsb_attack = lsfb(attacks);
                if (get_bit(lsb, board.bitboards[WP]) > 0) && rank(lsb) == 6 {
                    // white promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WQ, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WR, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WB, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, WN, board, false);
                    first_unused += 1;
                } else if (get_bit(lsb, board.bitboards[BP]) > 0) && rank(lsb) == 1 {
                    //black promotion
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BQ, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BR, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BB, board, false);
                    first_unused += 1;
                    moves.moves[first_unused] = encode_move(lsb, lsb_attack, BN, board, false);
                    first_unused += 1;
                } else {
                    moves.moves[first_unused] =
                        encode_move(lsb, lsb_attack, NO_PIECE, board, false);
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
            let king_square = lsfb(b.bitboards[WK]);
            let rook_rays = get_rook_attacks(king_square, b.occupancies[BOTH]);
            let bishop_rays = get_bishop_attacks(king_square, b.occupancies[BOTH]);
            //rays from king square

            let mut enemy_bishop_rays = 0u64;
            let mut enemy_rook_rays = 0u64;
            let mut enemy_bishops = b.bitboards[BB] | b.bitboards[BQ];
            let mut enemy_rooks = b.bitboards[BR] | b.bitboards[BQ];

            while enemy_bishops > 0 {
                let square = lsfb(enemy_bishops);
                enemy_bishop_rays |= get_bishop_attacks(square, b.occupancies[BOTH]);
                enemy_bishops = pop_bit(square, enemy_bishops);
            }

            while enemy_rooks > 0 {
                let square = lsfb(enemy_rooks);
                enemy_rook_rays |= get_rook_attacks(square, b.occupancies[BOTH]);
                enemy_rooks = pop_bit(square, enemy_rooks);
            }
            //generate rays for enemy slider attacks

            let mut pinned_by_bishop = bishop_rays & enemy_bishop_rays;
            let mut pinned_by_rook = rook_rays & enemy_rook_rays;
            //find pinned pieces

            while pinned_by_bishop > 0 {
                let pinned_square = lsfb(pinned_by_bishop);
                let ray = get_bishop_attacks(king_square, pop_bit(pinned_square, b.occupancies[BOTH]))
                    & get_bishop_attacks(pinned_square, b.occupancies[BOTH]);
                //ray corresponding to each pinned piece
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_bishop = pop_bit(pinned_square, pinned_by_bishop);
            }

            while pinned_by_rook > 0 {
                let pinned_square = lsfb(pinned_by_rook);
                let ray = get_rook_attacks(king_square, pop_bit(pinned_square, b.occupancies[BOTH]))
                    & get_rook_attacks(pinned_square, b.occupancies[BOTH]);
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_rook = pop_bit(pinned_square, pinned_by_rook);
            }
        }
        Colour::Black => {
            let mut ray_count: usize = 0;
            let king_square = lsfb(b.bitboards[BK]);
            let rook_rays = get_rook_attacks(king_square, b.occupancies[BOTH]);
            let bishop_rays = get_bishop_attacks(king_square, b.occupancies[BOTH]);

            let mut enemy_bishop_rays = 0u64;
            let mut enemy_rook_rays = 0u64;
            let mut enemy_bishops = b.bitboards[WB] | b.bitboards[WQ];
            let mut enemy_rooks = b.bitboards[WR] | b.bitboards[WQ];

            while enemy_bishops > 0 {
                let square = lsfb(enemy_bishops);
                enemy_bishop_rays |= get_bishop_attacks(square, b.occupancies[BOTH]);
                enemy_bishops = pop_bit(square, enemy_bishops);
            }

            while enemy_rooks > 0 {
                let square = lsfb(enemy_rooks);
                enemy_rook_rays |= get_rook_attacks(square, b.occupancies[BOTH]);
                enemy_rooks = pop_bit(square, enemy_rooks);
            }

            let mut pinned_by_bishop = bishop_rays & enemy_bishop_rays;
            let mut pinned_by_rook = rook_rays & enemy_rook_rays;

            while pinned_by_bishop > 0 {
                let pinned_square = lsfb(pinned_by_bishop);
                let ray = get_bishop_attacks(king_square, pop_bit(pinned_square, b.occupancies[BOTH]))
                    & get_bishop_attacks(pinned_square, b.occupancies[BOTH]);
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_bishop = pop_bit(pinned_square, pinned_by_bishop);
            }

            while pinned_by_rook > 0 {
                let pinned_square = lsfb(pinned_by_rook);
                let ray = get_rook_attacks(king_square, pop_bit(pinned_square, b.occupancies[BOTH]))
                    & get_rook_attacks(pinned_square, b.occupancies[BOTH]);
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
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[BOTH]);
            relevant_blockers = pop_bit(m.square_to() - 8, relevant_blockers);
            get_rook_attacks(lsfb(b.bitboards[WK]), relevant_blockers)
                & (b.bitboards[BR] | b.bitboards[BQ])
                == 0
        }
        6 => {
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[BOTH]);
            relevant_blockers = pop_bit(m.square_to() + 8, relevant_blockers);
            get_rook_attacks(lsfb(b.bitboards[BK]), relevant_blockers)
                & (b.bitboards[WR] | b.bitboards[WQ])
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
            let relevant_blockers = b.occupancies[BOTH] ^ b.bitboards[WK];
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
            let relevant_blockers = b.occupancies[BOTH] ^ b.bitboards[BK];
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
