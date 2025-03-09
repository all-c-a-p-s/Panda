use crate::board::*;
use crate::helper::*;
use crate::magic::*;
use crate::r#move::*;

pub struct MoveListEntry {
    pub m: Move,
    pub score: i32,
}

impl MoveListEntry {
    pub fn from(m: Move) -> Self {
        MoveListEntry { m, score: 0 }
    }
}

pub fn is_attacked(square: usize, colour: Colour, board: &Board) -> bool {
    //attacked BY colour
    match colour {
        /*in my testing this one-liner approach is no slower than checking one-by-one and
         * exiting as soon as one is true. I assume this is because of some optimisations
         * the compiler is doing under the hood.*/
        Colour::White => {
            //leapers then sliders
            BP_ATTACKS[square] & board.bitboards[WP] != 0
                || N_ATTACKS[square] & board.bitboards[WN] != 0
                || K_ATTACKS[square] & board.bitboards[WK] != 0
                || get_bishop_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[WB] | board.bitboards[WQ])
                    != 0
                || get_rook_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[WR] | board.bitboards[WQ])
                    != 0
        }
        Colour::Black => {
            //leapers then sliders
            WP_ATTACKS[square] & board.bitboards[BP] != 0
                || N_ATTACKS[square] & board.bitboards[BN] != 0
                || K_ATTACKS[square] & board.bitboards[BK] != 0
                || get_bishop_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[BB] | board.bitboards[BQ])
                    != 0
                || get_rook_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[BR] | board.bitboards[BQ])
                    != 0
        }
    }
}

pub fn get_attackers(square: usize, colour: Colour, b: &Board, occupancies: u64) -> u64 {
    //attacked BY colour
    match colour {
        Colour::White => {
            BP_ATTACKS[square] & b.bitboards[WP]
                | N_ATTACKS[square] & b.bitboards[WN]
                | K_ATTACKS[square] & b.bitboards[WK]
                | get_bishop_attacks(square, occupancies) & (b.bitboards[WB] | b.bitboards[WQ])
                | get_rook_attacks(square, occupancies) & (b.bitboards[WR] | b.bitboards[WQ])
        }
        Colour::Black => {
            WP_ATTACKS[square] & b.bitboards[BP]
                | N_ATTACKS[square] & b.bitboards[BN]
                | K_ATTACKS[square] & b.bitboards[BK]
                | get_bishop_attacks(square, occupancies) & (b.bitboards[BB] | b.bitboards[BQ])
                | get_rook_attacks(square, occupancies) & (b.bitboards[BR] | b.bitboards[BQ])
        }
    }
}

impl MoveList {
    pub fn empty() -> Self {
        MoveList {
            moves: [NULL_MOVE; MAX_MOVES],
        }
    }

    pub fn pawn_push_moves(&mut self, board: &Board, mut first_unused: usize) -> usize {
        match board.side_to_move {
            Colour::White => {
                let mut bitboard = board.bitboards[WP];
                while bitboard > 0 {
                    let lsb = lsfb(bitboard);
                    if get_bit(lsb + 8, board.occupancies[BOTH]) == 0 {
                        if rank(lsb) == 6 {
                            //promotion
                            //the piece type is passed into encode_move because only 2 bits are used to encode
                            //the promoted piece (and flag is used to detect if there is one)
                            self.moves[first_unused] =
                                encode_move(lsb, lsb + 8, QUEEN, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb + 8, ROOK, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb + 8, BISHOP, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb + 8, KNIGHT, PROMOTION_FLAG);
                            first_unused += 1;
                            //add all different possible promotions to move list
                        } else {
                            //regular pawn push
                            self.moves[first_unused] = encode_move(lsb, lsb + 8, NO_PIECE, NO_FLAG);
                            first_unused += 1;
                        }
                        if rank(lsb) == 1 && get_bit(lsb + 16, board.occupancies[BOTH]) == 0 {
                            //double push (we already know that lsb+8 is not occupied)
                            self.moves[first_unused] =
                                encode_move(lsb, lsb + 16, NO_PIECE, NO_FLAG);
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
                            self.moves[first_unused] =
                                encode_move(lsb, lsb - 8, QUEEN, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb - 8, ROOK, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb - 8, BISHOP, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb - 8, KNIGHT, PROMOTION_FLAG);
                            first_unused += 1;
                        } else {
                            self.moves[first_unused] = encode_move(lsb, lsb - 8, NO_PIECE, NO_FLAG);
                            first_unused += 1;
                        }
                        if rank(lsb) == 6 && get_bit(lsb - 16, board.occupancies[BOTH]) == 0 {
                            //double push
                            self.moves[first_unused] =
                                encode_move(lsb, lsb - 16, NO_PIECE, NO_FLAG);
                            first_unused += 1;
                        }
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
        };
        first_unused
    }

    pub fn castling_moves(&mut self, board: &Board, mut first_unused: usize) -> usize {
        match board.side_to_move {
            Colour::White => {
                if (board.castling & 0b0000_0001) > 0 {
                    //white kingside castling rights
                    if get_bit(F1, board.occupancies[BOTH]) == 0
                        && get_bit(G1, board.occupancies[BOTH]) == 0
                        && !is_attacked(E1, Colour::Black, board)
                        && !is_attacked(F1, Colour::Black, board)
                    //g1 checked later
                    {
                        self.moves[first_unused] = encode_move(E1, G1, NO_PIECE, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }

                if (board.castling & 0b0000_0010) > 0 {
                    //white queenside
                    if get_bit(B1, board.occupancies[BOTH]) == 0
                        && get_bit(C1, board.occupancies[BOTH]) == 0
                        && get_bit(D1, board.occupancies[BOTH]) == 0
                        && !is_attacked(E1, Colour::Black, board)
                        && !is_attacked(D1, Colour::Black, board)
                    {
                        self.moves[first_unused] = encode_move(E1, C1, NO_PIECE, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }
            }
            Colour::Black => {
                if (board.castling & 0b0000_0100) > 0 {
                    //black kingside
                    if get_bit(G8, board.occupancies[BOTH]) == 0
                        && get_bit(F8, board.occupancies[BOTH]) == 0
                        && !is_attacked(E8, Colour::White, board)
                        && !is_attacked(F8, Colour::White, board)
                    {
                        self.moves[first_unused] = encode_move(E8, G8, NO_PIECE, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }

                if (board.castling & 0b0000_1000) > 0 {
                    //black queenside
                    if get_bit(D8, board.occupancies[BOTH]) == 0
                        && get_bit(C8, board.occupancies[BOTH]) == 0
                        && get_bit(B8, board.occupancies[BOTH]) == 0
                        && !is_attacked(E8, Colour::White, board)
                        && !is_attacked(D8, Colour::White, board)
                    {
                        self.moves[first_unused] = encode_move(E8, C8, NO_PIECE, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }
            }
        };
        first_unused
    }

    pub fn gen_moves(board: &Board) -> Self {
        let (min, max) = match board.side_to_move {
            Colour::White => (WP, BP),
            Colour::Black => (BP, 12),
        };

        let mut moves = MoveList::empty();

        let mut first_unused = moves.pawn_push_moves(board, 0);
        first_unused = moves.castling_moves(board, first_unused);

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
                    if piece == WP && rank(lsb) == 6 {
                        // white promotion
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, QUEEN, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, ROOK, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, BISHOP, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, KNIGHT, PROMOTION_FLAG);
                        first_unused += 1;
                    } else if piece == BP && rank(lsb) == 1 {
                        //black promotion
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, QUEEN, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, ROOK, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, BISHOP, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, KNIGHT, PROMOTION_FLAG);
                        first_unused += 1;
                    } else if lsb_attack == board.en_passant && piece_type(piece) == PAWN {
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, NO_PIECE, EN_PASSANT_FLAG);
                        first_unused += 1;
                    } else {
                        moves.moves[first_unused] = encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                        first_unused += 1;
                    } //list to return here
                    attacks = pop_bit(lsb_attack, attacks);
                }
                bitboard = pop_bit(lsb, bitboard);
            }
        }
        moves
    }

    pub fn gen_captures(board: &mut Board) -> Self {
        //special capture-only move generation for quiescence search
        //NOTE: this generates pseudo-legal captures, and they are checked to be legal in place
        let (min, max) = match board.side_to_move {
            Colour::White => (WP, BP),
            Colour::Black => (BP, 12),
        };

        let mut moves = MoveList::empty();

        let mut first_unused = 0;
        for piece in min..max {
            //pieces of colour to move
            let mut bitboard = board.bitboards[piece];

            while bitboard > 0 {
                let lsb = lsfb(bitboard); // never panics as loop will have already exited
                let mut attacks = match piece {
                    //ensuring captures is handled piece-by-piece instead of by colour after
                    //because otherwise en passant captures get removed
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
                    WN => N_ATTACKS[lsb] & board.occupancies[BLACK],
                    BN => N_ATTACKS[lsb] & board.occupancies[WHITE],
                    WB => {
                        get_bishop_attacks(lsb, board.occupancies[BOTH]) & board.occupancies[BLACK]
                    }
                    BB => {
                        get_bishop_attacks(lsb, board.occupancies[BOTH]) & board.occupancies[WHITE]
                    }
                    WR => get_rook_attacks(lsb, board.occupancies[BOTH]) & board.occupancies[BLACK],
                    BR => get_rook_attacks(lsb, board.occupancies[BOTH]) & board.occupancies[WHITE],
                    WQ => {
                        get_queen_attacks(lsb, board.occupancies[BOTH]) & board.occupancies[BLACK]
                    }
                    BQ => {
                        get_queen_attacks(lsb, board.occupancies[BOTH]) & board.occupancies[WHITE]
                    }
                    WK => K_ATTACKS[lsb] & board.occupancies[BLACK],
                    BK => K_ATTACKS[lsb] & board.occupancies[WHITE],
                    _ => panic!("this is impossible"),
                };
                while attacks > 0 {
                    let lsb_attack = lsfb(attacks);
                    if (get_bit(lsb, board.bitboards[WP]) > 0) && rank(lsb) == 6 {
                        // white promotion
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, QUEEN, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, ROOK, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, BISHOP, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, KNIGHT, PROMOTION_FLAG);
                        first_unused += 1;
                    } else if (get_bit(lsb, board.bitboards[BP]) > 0) && rank(lsb) == 1 {
                        //black promotion
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, QUEEN, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, ROOK, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, BISHOP, PROMOTION_FLAG);
                        first_unused += 1;
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, KNIGHT, PROMOTION_FLAG);
                        first_unused += 1;
                    } else if lsb_attack == board.en_passant && piece_type(piece) == PAWN {
                        moves.moves[first_unused] =
                            encode_move(lsb, lsb_attack, NO_PIECE, EN_PASSANT_FLAG);
                        first_unused += 1;
                    } else {
                        moves.moves[first_unused] = encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                        first_unused += 1;
                    } //list to return here
                    attacks = pop_bit(lsb_attack, attacks);
                }
                bitboard = pop_bit(lsb, bitboard);
            }
        }
        moves
    }

    pub fn gen_legal(b: &mut Board) -> Self {
        let pseudo_legal = MoveList::gen_moves(b);
        let mut legal = MoveList {
            moves: [NULL_MOVE; MAX_MOVES],
        };
        let mut last = 0;
        for i in 0..MAX_MOVES {
            if pseudo_legal.moves[i].is_null() {
                break;
            }
            if is_legal(pseudo_legal.moves[i], b) {
                legal.moves[last] = pseudo_legal.moves[i];
                last += 1;
            }
        }
        legal
    }
}

pub fn get_smallest_attack(b: &mut Board, square: usize) -> Move {
    //NOTE: for speed this does not take pins into account
    //attacked BY colour
    match b.side_to_move {
        Colour::White => {
            let pawn_attackers = BP_ATTACKS[square] & b.bitboards[WP];
            if pawn_attackers > 0 {
                let sq_from = lsfb(pawn_attackers);
                return match rank(square) {
                    7 => encode_move(sq_from, square, QUEEN, PROMOTION_FLAG),
                    //no point considering underpromotions
                    _ => encode_move(sq_from, square, NO_PIECE, NO_FLAG),
                };
            }
            let knight_attackers = N_ATTACKS[square] & b.bitboards[WN];
            if knight_attackers > 0 {
                let sq_from = lsfb(knight_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let king_attackers = K_ATTACKS[square] & b.bitboards[WK];
            if king_attackers > 0 {
                //only one king
                let sq_from = lsfb(king_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let bishop_attacks = get_bishop_attacks(square, b.occupancies[BOTH]);
            //use later to get queen attackers
            let bishop_attackers = bishop_attacks & b.bitboards[WB];
            if bishop_attackers > 0 {
                let sq_from = lsfb(bishop_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let rook_attacks = get_rook_attacks(square, b.occupancies[BOTH]);
            let rook_attackers = rook_attacks & b.bitboards[WR];
            if rook_attackers > 0 {
                let sq_from = lsfb(rook_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let queen_attackers = (rook_attacks | bishop_attacks) & b.bitboards[WQ];
            if queen_attackers > 0 {
                let sq_from = lsfb(queen_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
        }
        Colour::Black => {
            let pawn_attackers = WP_ATTACKS[square] & b.bitboards[BP];
            if pawn_attackers > 0 {
                let sq_from = lsfb(pawn_attackers);
                return match rank(square) {
                    0 => encode_move(sq_from, square, QUEEN, PROMOTION_FLAG),
                    //no point considering underpromotions
                    _ => encode_move(sq_from, square, NO_PIECE, NO_FLAG),
                };
            }
            let knight_attackers = N_ATTACKS[square] & b.bitboards[BN];
            if knight_attackers > 0 {
                let sq_from = lsfb(knight_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let king_attackers = K_ATTACKS[square] & b.bitboards[BK];
            if king_attackers > 0 {
                //only one king
                let sq_from = lsfb(king_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let bishop_attacks = get_bishop_attacks(square, b.occupancies[BOTH]);
            //use later to get queen attackers
            let bishop_attackers = bishop_attacks & b.bitboards[BB];
            if bishop_attackers > 0 {
                let sq_from = lsfb(bishop_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let rook_attacks = get_rook_attacks(square, b.occupancies[BOTH]);
            let rook_attackers = rook_attacks & b.bitboards[BR];
            if rook_attackers > 0 {
                let sq_from = lsfb(rook_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let queen_attackers = (rook_attacks | bishop_attacks) & b.bitboards[BQ];
            if queen_attackers > 0 {
                let sq_from = lsfb(queen_attackers);
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
        }
    }
    NULL_MOVE
}

/* I don't think checking all edge cases separately is actually faster
but code to detect pins might be useful in the future */

pub fn get_pin_rays(b: &Board) -> Vec<u64> {
    let mut res = vec![];
    match b.side_to_move {
        Colour::White => {
            let king_sq = lsfb(b.bitboards[WK]);
            let (mut tb, mut tr) = (
                b.bitboards[BB] | b.bitboards[BQ],
                b.bitboards[BR] | b.bitboards[BQ],
            );

            while tb > 0 {
                let sq = lsfb(tb);
                if rank(sq) == rank(king_sq) || file(sq) == file(king_sq) {
                    tb = pop_bit(sq, tb);
                    continue;
                }
                let between = RAY_BETWEEN[sq][king_sq];
                let maybe_pin_ray = (between ^ b.bitboards[WK]) ^ set_bit(sq, 0);
                if count(maybe_pin_ray & b.occupancies[WHITE]) == 1
                    && count(maybe_pin_ray & b.occupancies[BLACK]) == 0
                {
                    res.push(between);
                }

                tb = pop_bit(sq, tb);
            }

            while tr > 0 {
                let sq = lsfb(tr);
                if rank(sq) != rank(king_sq) && file(sq) != file(king_sq) {
                    tr = pop_bit(sq, tr);
                    continue;
                }
                let between = RAY_BETWEEN[sq][king_sq];
                let maybe_pin_ray = (between ^ b.bitboards[WK]) ^ set_bit(sq, 0);
                if count(maybe_pin_ray & b.occupancies[WHITE]) == 1
                    && count(maybe_pin_ray & b.occupancies[BLACK]) == 0
                {
                    res.push(between);
                }

                tr = pop_bit(sq, tr);
            }
        }
        Colour::Black => {
            let king_sq = lsfb(b.bitboards[BK]);
            let (mut tb, mut tr) = (
                b.bitboards[WB] | b.bitboards[WQ],
                b.bitboards[WR] | b.bitboards[WQ],
            );

            while tb > 0 {
                let sq = lsfb(tb);
                if rank(sq) == rank(king_sq) || file(sq) == file(king_sq) {
                    tb = pop_bit(sq, tb);
                    continue;
                }
                let between = RAY_BETWEEN[sq][king_sq];
                let maybe_pin_ray = (between ^ b.bitboards[BK]) ^ set_bit(sq, 0);
                if count(maybe_pin_ray & b.occupancies[BLACK]) == 1
                    && count(maybe_pin_ray & b.occupancies[WHITE]) == 0
                {
                    res.push(between);
                }

                tb = pop_bit(sq, tb);
            }

            while tr > 0 {
                let sq = lsfb(tr);
                if rank(sq) != rank(king_sq) && file(sq) != file(king_sq) {
                    tr = pop_bit(sq, tr);
                    continue;
                }
                let between = RAY_BETWEEN[sq][king_sq];
                let maybe_pin_ray = (between ^ b.bitboards[BK]) ^ set_bit(sq, 0);
                if count(maybe_pin_ray & b.occupancies[BLACK]) == 1
                    && count(maybe_pin_ray & b.occupancies[WHITE]) == 0
                {
                    res.push(between);
                }

                tr = pop_bit(sq, tr);
            }
        }
    }
    res
}

pub const fn in_between(sq1: usize, sq2: usize) -> u64 {
    if sq1 == sq2 {
        return 1u64 << sq1;
    }

    let file1 = file(sq1);
    let rank1 = rank(sq1);
    let file2 = file(sq2);
    let rank2 = rank(sq2);

    if rank1 == rank2 {
        let min_file;
        let max_file;
        if file1 < file2 {
            min_file = file1;
            max_file = file2;
        } else {
            min_file = file2;
            max_file = file1;
        }

        let mut bitboard: u64 = 0;
        let mut file = min_file;
        while file <= max_file {
            let sq = rank1 * 8 + file;
            bitboard |= 1u64 << sq;
            file += 1;
        }
        return bitboard;
    }

    if file1 == file2 {
        let min_rank;
        let max_rank;
        if rank1 < rank2 {
            min_rank = rank1;
            max_rank = rank2;
        } else {
            min_rank = rank2;
            max_rank = rank1;
        }

        let mut bitboard: u64 = 0;
        let mut rank = min_rank;
        while rank <= max_rank {
            let sq = rank * 8 + file1;
            bitboard |= 1u64 << sq;
            rank += 1;
        }
        return bitboard;
    }

    let file_diff = if file1 > file2 {
        file1 - file2
    } else {
        file2 - file1
    };
    let rank_diff = if rank1 > rank2 {
        rank1 - rank2
    } else {
        rank2 - rank1
    };

    if file_diff == rank_diff {
        let file_step: i8 = if file2 > file1 { 1 } else { -1 };
        let rank_step: i8 = if rank2 > rank1 { 1 } else { -1 };

        let mut f = file1 as i8;
        let mut r = rank1 as i8;
        let mut bitboard: u64 = 0;

        while f >= 0 && f <= 7 && r >= 0 && r <= 7 {
            let sq = (r * 8 + f) as u8;
            bitboard |= 1u64 << sq;

            if f == file2 as i8 && r == rank2 as i8 {
                return bitboard;
            }

            f += file_step;
            r += rank_step;
        }
    }

    0
}

pub static RAY_BETWEEN: [[u64; 64]; 64] = {
    let mut res = [[0u64; 64]; 64];
    let mut from = A1;
    while from < 64 {
        let mut to = A1;
        while to < 64 {
            res[from][to] = in_between(from, to);
            to += 1;
        }
        from += 1;
    }
    res
};

pub fn check_en_passant(m: Move, b: &Board) -> bool {
    //checks en passant edge case where en passant reveals check on the king
    match m.piece_moved(&b) {
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
        _ => unreachable!(),
    }
}

pub fn legal_non_check_evasion(m: Move, b: &Board, pin_rays: &[u64]) -> bool {
    //separate function used to generate check evasions
    for r in pin_rays {
        //check that not moving pinned piece out of pin ray
        if set_bit(m.square_from(), 0) & r > 0
            && set_bit(m.square_to(), 0) & r == 0
            && piece_type(m.piece_moved(b)) != KING
        {
            return false;
        }
    }

    if m.piece_moved(&b) == WK {
        //check that king isn't moving into check
        let mut black_attacks = 0u64;
        for i in BP..=BK {
            let mut piece_bb = b.bitboards[i];
            let relevant_blockers = b.occupancies[BOTH] ^ b.bitboards[WK];
            //king blocking slider attacks doesn't count because it can move back
            //into a new attack from the same slider
            while piece_bb > 0 {
                let sq = lsfb(piece_bb);
                black_attacks |= match i {
                    6 => BP_ATTACKS[sq],
                    7 => N_ATTACKS[sq],
                    8 => get_bishop_attacks(sq, relevant_blockers),
                    9 => get_rook_attacks(sq, relevant_blockers),
                    10 => get_queen_attacks(sq, relevant_blockers),
                    11 => K_ATTACKS[sq],
                    _ => unreachable!(),
                };
                piece_bb = pop_bit(sq, piece_bb);
            }
        }
        return set_bit(m.square_to(), 0) & black_attacks == 0;
    } else if m.piece_moved(&b) == BK {
        let mut white_attacks = 0u64;
        for i in WP..=WK {
            let mut piece_bb = b.bitboards[i];
            let relevant_blockers = b.occupancies[BOTH] ^ b.bitboards[BK];
            //king blocking slider attacks doesn't count because it can move back
            //into a new attack from the same slider
            while piece_bb > 0 {
                let sq = lsfb(piece_bb);
                white_attacks |= match i {
                    0 => WP_ATTACKS[sq],
                    1 => N_ATTACKS[sq],
                    2 => get_bishop_attacks(sq, relevant_blockers),
                    3 => get_rook_attacks(sq, relevant_blockers),
                    4 => get_queen_attacks(sq, relevant_blockers),
                    5 => K_ATTACKS[sq],
                    _ => unreachable!(),
                };
                piece_bb = pop_bit(sq, piece_bb);
            }
        }
        return set_bit(m.square_to(), 0) & white_attacks == 0;
    } else if m.is_en_passant() {
        //special case where en passant capture creates removes 2 pawns from 1 rank -> discovered check
        return check_en_passant(m, b);
    }
    true
}
