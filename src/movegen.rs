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

            let mut pinned_by_bishop = bishop_rays & enemy_bishop_rays; //<- error here!
            let mut pinned_by_rook = rook_rays & enemy_rook_rays; //<- error here!
                                                                  //find pinned pieces

            while pinned_by_bishop > 0 {
                let pinned_square = lsfb(pinned_by_bishop);
                let ray = (get_bishop_attacks(king_square, 0)
                    & get_bishop_attacks(pinned_square, 0))
                    | set_bit(pinned_square, 0);
                //ray corresponding to each pinned piece
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_bishop = pop_bit(pinned_square, pinned_by_bishop);
            }

            while pinned_by_rook > 0 {
                let pinned_square = lsfb(pinned_by_rook);
                let ray = (get_rook_attacks(king_square, 0) & get_rook_attacks(pinned_square, 0))
                    | set_bit(pinned_square, 0);
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

            let mut pinned_by_bishop = bishop_rays & enemy_bishop_rays; //<- error here!
            let mut pinned_by_rook = rook_rays & enemy_rook_rays; //<- error here!

            while pinned_by_bishop > 0 {
                let pinned_square = lsfb(pinned_by_bishop);
                let ray = (get_bishop_attacks(king_square, 0)
                    & get_bishop_attacks(pinned_square, 0))
                    | set_bit(pinned_square, 0);
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_bishop = pop_bit(pinned_square, pinned_by_bishop);
            }

            while pinned_by_rook > 0 {
                let pinned_square = lsfb(pinned_by_rook);
                let ray = (get_rook_attacks(king_square, 0) & get_rook_attacks(pinned_square, 0))
                    | set_bit(pinned_square, 0);
                res[ray_count] = ray;
                ray_count += 1;
                pinned_by_rook = pop_bit(pinned_square, pinned_by_rook);
            }
        }
    }
    res
}

//taken from Viridithas
const fn in_between(sq1: usize, sq2: usize) -> u64 {
    const M1: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    const A2A7: u64 = 0x0001_0101_0101_0100;
    const B2G7: u64 = 0x0040_2010_0804_0200;
    const H1B7: u64 = 0x0002_0408_1020_4080;

    let btwn = (M1 << sq1) ^ (M1 << sq2);
    let file = ((sq2 & 7).wrapping_add((sq1 & 7).wrapping_neg())) as u64;
    let rank = (((sq2 | 7).wrapping_sub(sq1)) >> 3) as u64;
    let mut line = ((file & 7).wrapping_sub(1)) & A2A7;
    line += 2 * ((rank & 7).wrapping_sub(1) >> 58);
    line += ((rank.wrapping_sub(file) & 15).wrapping_sub(1)) & B2G7;
    line += ((rank.wrapping_add(file) & 15).wrapping_sub(1)) & H1B7;
    line = line.wrapping_mul(btwn & btwn.wrapping_neg());
    line & btwn
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

    if m.piece_moved(&b) == 5 {
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
    } else if m.piece_moved(&b) == 11 {
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
