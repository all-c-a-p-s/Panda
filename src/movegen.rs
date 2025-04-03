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
                while let Some(lsb) = lsfb(bitboard) {
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
                while let Some(lsb) = lsfb(bitboard) {
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

    pub fn gen_pawn_captures(&mut self, board: &Board, mut first_unused: usize) -> usize {
        match board.side_to_move {
            Colour::White => {
                let mut bitboard = board.bitboards[WP];

                while let Some(lsb) = lsfb(bitboard) {
                    let mut attacks = WP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[BLACK],
                            k => set_bit(k, board.occupancies[BLACK]),
                        };

                    while let Some(lsb_attack) = lsfb(attacks) {
                        //promotions that are also captures
                        if rank(lsb) == 6 {
                            // white promotion
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, QUEEN, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, ROOK, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, BISHOP, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, KNIGHT, PROMOTION_FLAG);
                            first_unused += 1;
                        } else if lsb_attack == board.en_passant {
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, NO_PIECE, EN_PASSANT_FLAG);
                            first_unused += 1;
                        } else {
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                            first_unused += 1;
                        } //list to return here
                        attacks = pop_bit(lsb_attack, attacks);
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
            Colour::Black => {
                let mut bitboard = board.bitboards[BP];

                while let Some(lsb) = lsfb(bitboard) {
                    let mut attacks = BP_ATTACKS[lsb]
                        & match board.en_passant {
                            NO_SQUARE => board.occupancies[WHITE],
                            k => set_bit(k, board.occupancies[WHITE]),
                        };

                    while let Some(lsb_attack) = lsfb(attacks) {
                        //promotions that are also captures
                        if rank(lsb) == 1 {
                            // white promotion
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, QUEEN, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, ROOK, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, BISHOP, PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, KNIGHT, PROMOTION_FLAG);
                            first_unused += 1;
                        } else if lsb_attack == board.en_passant {
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, NO_PIECE, EN_PASSANT_FLAG);
                            first_unused += 1;
                        } else {
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                            first_unused += 1;
                        } //list to return here
                        attacks = pop_bit(lsb_attack, attacks);
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
        };
        first_unused
    }

    pub fn gen_knight_moves<const CAPS_ONLY: bool>(
        &mut self,
        board: &Board,
        mut first_unused: usize,
    ) -> usize {
        let (piece, occs, opps) = match board.side_to_move {
            Colour::White => (WN, board.occupancies[WHITE], board.occupancies[BLACK]),
            Colour::Black => (BN, board.occupancies[BLACK], board.occupancies[WHITE]),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let mut attacks = N_ATTACKS[lsb] & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                first_unused += 1;
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }

        first_unused
    }

    pub fn gen_bishop_moves<const CAPS_ONLY: bool>(
        &mut self,
        board: &Board,
        mut first_unused: usize,
    ) -> usize {
        let (piece, occs, opps) = match board.side_to_move {
            Colour::White => (WB, board.occupancies[WHITE], board.occupancies[BLACK]),
            Colour::Black => (BB, board.occupancies[BLACK], board.occupancies[WHITE]),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let mut attacks = get_bishop_attacks(lsb, board.occupancies[BOTH])
                & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                first_unused += 1;
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }

        first_unused
    }

    pub fn gen_rook_moves<const CAPS_ONLY: bool>(
        &mut self,
        board: &Board,
        mut first_unused: usize,
    ) -> usize {
        let (piece, occs, opps) = match board.side_to_move {
            Colour::White => (WR, board.occupancies[WHITE], board.occupancies[BLACK]),
            Colour::Black => (BR, board.occupancies[BLACK], board.occupancies[WHITE]),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let mut attacks = get_rook_attacks(lsb, board.occupancies[BOTH])
                & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                first_unused += 1;
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }

        first_unused
    }

    pub fn gen_queen_moves<const CAPS_ONLY: bool>(
        &mut self,
        board: &Board,
        mut first_unused: usize,
    ) -> usize {
        let (piece, occs, opps) = match board.side_to_move {
            Colour::White => (WQ, board.occupancies[WHITE], board.occupancies[BLACK]),
            Colour::Black => (BQ, board.occupancies[BLACK], board.occupancies[WHITE]),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let mut attacks = get_queen_attacks(lsb, board.occupancies[BOTH])
                & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                first_unused += 1;
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }

        first_unused
    }

    pub fn gen_king_moves<const CAPS_ONLY: bool>(
        &mut self,
        board: &Board,
        mut first_unused: usize,
    ) -> usize {
        let (piece, occs, opps) = match board.side_to_move {
            Colour::White => (WK, board.occupancies[WHITE], board.occupancies[BLACK]),
            Colour::Black => (BK, board.occupancies[BLACK], board.occupancies[WHITE]),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let mut attacks = K_ATTACKS[lsb] & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, NO_PIECE, NO_FLAG);
                first_unused += 1;
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }

        first_unused
    }

    //TODO: generate moves by piece one at a time / staged move generation
    pub fn gen_moves<const CAPS_ONLY: bool>(board: &Board) -> Self {
        let mut moves = MoveList::empty();

        let mut first_unused = 0;
        if !CAPS_ONLY {
            first_unused = moves.pawn_push_moves(board, first_unused);
            first_unused = moves.castling_moves(board, first_unused);
        }
        first_unused = moves.gen_pawn_captures(board, first_unused);
        first_unused = moves.gen_knight_moves::<CAPS_ONLY>(board, first_unused);
        first_unused = moves.gen_bishop_moves::<CAPS_ONLY>(board, first_unused);
        first_unused = moves.gen_rook_moves::<CAPS_ONLY>(board, first_unused);
        first_unused = moves.gen_queen_moves::<CAPS_ONLY>(board, first_unused);
        _ = moves.gen_king_moves::<CAPS_ONLY>(board, first_unused);

        moves
    }

    pub fn gen_legal(b: &mut Board) -> Self {
        let pseudo_legal = MoveList::gen_moves::<false>(b);
        let mut legal = MoveList {
            moves: [NULL_MOVE; MAX_MOVES],
        };
        let mut last = 0;
        for i in 0..MAX_MOVES {
            if pseudo_legal.moves[i].is_null() {
                break;
            }
            if b.is_legal(pseudo_legal.moves[i]) {
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

    //SAFETY for these: we know that an attacker exists

    match b.side_to_move {
        Colour::White => {
            let pawn_attackers = BP_ATTACKS[square] & b.bitboards[WP];
            if pawn_attackers > 0 {
                let sq_from = unsafe { lsfb(pawn_attackers).unwrap_unchecked() };
                return match rank(square) {
                    7 => encode_move(sq_from, square, QUEEN, PROMOTION_FLAG),
                    //no point considering underpromotions
                    _ => encode_move(sq_from, square, NO_PIECE, NO_FLAG),
                };
            }
            let knight_attackers = N_ATTACKS[square] & b.bitboards[WN];
            if knight_attackers > 0 {
                let sq_from = unsafe { lsfb(knight_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let king_attackers = K_ATTACKS[square] & b.bitboards[WK];
            if king_attackers > 0 {
                //only one king
                let sq_from = unsafe { lsfb(king_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let bishop_attacks = get_bishop_attacks(square, b.occupancies[BOTH]);
            //use later to get queen attackers
            let bishop_attackers = bishop_attacks & b.bitboards[WB];
            if bishop_attackers > 0 {
                let sq_from = unsafe { lsfb(bishop_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let rook_attacks = get_rook_attacks(square, b.occupancies[BOTH]);
            let rook_attackers = rook_attacks & b.bitboards[WR];
            if rook_attackers > 0 {
                let sq_from = unsafe { lsfb(rook_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let queen_attackers = (rook_attacks | bishop_attacks) & b.bitboards[WQ];
            if queen_attackers > 0 {
                let sq_from = unsafe { lsfb(queen_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
        }
        Colour::Black => {
            let pawn_attackers = WP_ATTACKS[square] & b.bitboards[BP];
            if pawn_attackers > 0 {
                let sq_from = unsafe { lsfb(pawn_attackers).unwrap_unchecked() };
                return match rank(square) {
                    0 => encode_move(sq_from, square, QUEEN, PROMOTION_FLAG),
                    //no point considering underpromotions
                    _ => encode_move(sq_from, square, NO_PIECE, NO_FLAG),
                };
            }
            let knight_attackers = N_ATTACKS[square] & b.bitboards[BN];
            if knight_attackers > 0 {
                let sq_from = unsafe { lsfb(knight_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let king_attackers = K_ATTACKS[square] & b.bitboards[BK];
            if king_attackers > 0 {
                //only one king
                let sq_from = unsafe { lsfb(king_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let bishop_attacks = get_bishop_attacks(square, b.occupancies[BOTH]);
            //use later to get queen attackers
            let bishop_attackers = bishop_attacks & b.bitboards[BB];
            if bishop_attackers > 0 {
                let sq_from = unsafe { lsfb(bishop_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let rook_attacks = get_rook_attacks(square, b.occupancies[BOTH]);
            let rook_attackers = rook_attacks & b.bitboards[BR];
            if rook_attackers > 0 {
                let sq_from = unsafe { lsfb(rook_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
            let queen_attackers = (rook_attacks | bishop_attacks) & b.bitboards[BQ];
            if queen_attackers > 0 {
                let sq_from = unsafe { lsfb(queen_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, NO_PIECE, NO_FLAG);
            }
        }
    }
    NULL_MOVE
}

pub const fn in_between(mut sq1: usize, mut sq2: usize) -> u64 {
    if sq1 == sq2 {
        return 0u64;
    } else if sq1 > sq2 {
        let temp = sq2;
        sq2 = sq1;
        sq1 = temp;
    }

    let dx = file(sq2) as i8 - file(sq1) as i8;
    let dy = rank(sq2) as i8 - rank(sq1) as i8;

    let orthogonal = dx == 0 || dy == 0;
    let diagonal = dx.abs() == dy.abs();

    if !(orthogonal || diagonal) {
        return 0u64;
    }

    let (dx, dy) = (dx.signum(), dy.signum());
    let (dx, dy) = (dx, dy * 8);

    let mut res = 0u64;

    while ((sq1 as i8 + dx + dy) as usize) < sq2 {
        res |= set_bit((sq1 as i8 + dx + dy) as usize, 0);
        sq1 = (sq1 as i8 + dx + dy) as usize;
    }
    res
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
        WP => {
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[BOTH]);
            relevant_blockers = pop_bit(m.square_to() - 8, relevant_blockers);
            //SAFETY: there MUST be a king on the board
            get_rook_attacks(
                unsafe { lsfb(b.bitboards[WK]).unwrap_unchecked() },
                relevant_blockers,
            ) & (b.bitboards[BR] | b.bitboards[BQ])
                == 0
        }
        BP => {
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[BOTH]);
            relevant_blockers = pop_bit(m.square_to() + 8, relevant_blockers);
            //SAFETY: there MUST be a king on the board
            get_rook_attacks(
                unsafe { lsfb(b.bitboards[BK]).unwrap_unchecked() },
                relevant_blockers,
            ) & (b.bitboards[WR] | b.bitboards[WQ])
                == 0
        }
        _ => unreachable!(),
    }
}
