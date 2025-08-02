use crate::board::{BitBoard, Board, Colour};
use crate::helper::{file, get_bit, lsfb, pop_bit, rank, set_bit, BLACK, BOTH, MAX_MOVES, WHITE};
use crate::magic::{
    get_bishop_attacks, get_queen_attacks, get_rook_attacks, BP_ATTACKS, K_ATTACKS, N_ATTACKS,
    WP_ATTACKS,
};
use crate::r#move::{
    encode_move, Move, MoveList, CASTLING_FLAG, EN_PASSANT_FLAG, NO_FLAG, NULL_MOVE, PROMOTION_FLAG,
};

use crate::types::{Piece, PieceType, Square};

pub struct MoveListEntry {
    pub m: Move,
    pub score: i32,
}

impl MoveListEntry {
    #[must_use]
    pub fn from(m: Move) -> Self {
        MoveListEntry { m, score: 0 }
    }
}

#[must_use]
pub fn is_attacked(square: Square, colour: Colour, board: &Board) -> bool {
    //attacked BY colour
    let square = square as usize;
    //have to convert because get_rook_attacks() and get_bishop_attacks() need a usize
    match colour {
        Colour::White => {
            //leapers then sliders
            BP_ATTACKS[square] & board.bitboards[Piece::WP] != 0
                || N_ATTACKS[square] & board.bitboards[Piece::WN] != 0
                || K_ATTACKS[square] & board.bitboards[Piece::WK] != 0
                || get_bishop_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[Piece::WB] | board.bitboards[Piece::WQ])
                    != 0
                || get_rook_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[Piece::WR] | board.bitboards[Piece::WQ])
                    != 0
        }
        Colour::Black => {
            //leapers then sliders
            WP_ATTACKS[square] & board.bitboards[Piece::BP] != 0
                || N_ATTACKS[square] & board.bitboards[Piece::BN] != 0
                || K_ATTACKS[square] & board.bitboards[Piece::BK] != 0
                || get_bishop_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[Piece::BB] | board.bitboards[Piece::BQ])
                    != 0
                || get_rook_attacks(square, board.occupancies[BOTH])
                    & (board.bitboards[Piece::BR] | board.bitboards[Piece::BQ])
                    != 0
        }
    }
}

#[must_use]
pub fn get_attackers(square: Square, colour: Colour, b: &Board, occupancies: BitBoard) -> BitBoard {
    //attacked BY colour

    let square = square as usize;
    //have to convert because get_rook_attacks() and get_bishop_attacks() need a usize

    match colour {
        Colour::White => {
            BP_ATTACKS[square] & b.bitboards[Piece::WP]
                | N_ATTACKS[square] & b.bitboards[Piece::WN]
                | K_ATTACKS[square] & b.bitboards[Piece::WK]
                | get_bishop_attacks(square, occupancies)
                    & (b.bitboards[Piece::WB] | b.bitboards[Piece::WQ])
                | get_rook_attacks(square, occupancies)
                    & (b.bitboards[Piece::WR] | b.bitboards[Piece::WQ])
        }
        Colour::Black => {
            WP_ATTACKS[square] & b.bitboards[Piece::BP]
                | N_ATTACKS[square] & b.bitboards[Piece::BN]
                | K_ATTACKS[square] & b.bitboards[Piece::BK]
                | get_bishop_attacks(square, occupancies)
                    & (b.bitboards[Piece::BB] | b.bitboards[Piece::BQ])
                | get_rook_attacks(square, occupancies)
                    & (b.bitboards[Piece::BR] | b.bitboards[Piece::BQ])
        }
    }
}

impl MoveList {
    #[must_use]
    pub const fn empty() -> Self {
        MoveList {
            moves: [NULL_MOVE; MAX_MOVES],
        }
    }

    pub fn pawn_push_moves(&mut self, board: &Board, mut first_unused: usize) -> usize {
        match board.side_to_move {
            Colour::White => {
                let mut bitboard = board.bitboards[Piece::WP];
                while let Some(lsb) = lsfb(bitboard) {
                    let up = unsafe { lsb.add_unchecked(8) };
                    if get_bit(up, board.occupancies[BOTH]) == 0 {
                        if rank(lsb) == 6 {
                            //promotion
                            //the piece type is passed into encode_move because only 2 bits are used to encode
                            //the promoted piece (and flag is used to detect if there is one)
                            self.moves[first_unused] =
                                encode_move(lsb, up, Some(PieceType::Queen), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, up, Some(PieceType::Rook), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, up, Some(PieceType::Bishop), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, up, Some(PieceType::Knight), PROMOTION_FLAG);
                            first_unused += 1;
                            //add all different possible promotions to move list
                        } else {
                            //regular pawn push
                            self.moves[first_unused] = encode_move(lsb, up, None, NO_FLAG);
                            first_unused += 1;
                        }
                        if rank(lsb) == 1
                            && get_bit(unsafe { lsb.add_unchecked(16) }, board.occupancies[BOTH])
                                == 0
                        {
                            //double push (we already know that lsb+8 is not occupied)
                            self.moves[first_unused] =
                                encode_move(lsb, unsafe { lsb.add_unchecked(16) }, None, NO_FLAG);
                            first_unused += 1;
                        }
                    }
                    bitboard = pop_bit(lsb, bitboard);
                    //pop pawns from bitboard
                }
            }
            Colour::Black => {
                let mut bitboard = board.bitboards[Piece::BP];
                while let Some(lsb) = lsfb(bitboard) {
                    let down = unsafe { lsb.sub_unchecked(8) };
                    if get_bit(down, board.occupancies[BOTH]) == 0 {
                        if rank(down) == 0 {
                            //promotion
                            self.moves[first_unused] =
                                encode_move(lsb, down, Some(PieceType::Queen), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, down, Some(PieceType::Rook), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, down, Some(PieceType::Bishop), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, down, Some(PieceType::Knight), PROMOTION_FLAG);
                            first_unused += 1;
                        } else {
                            self.moves[first_unused] = encode_move(lsb, down, None, NO_FLAG);
                            first_unused += 1;
                        }
                        if rank(lsb) == 6
                            && get_bit(unsafe { lsb.sub_unchecked(16) }, board.occupancies[BOTH])
                                == 0
                        {
                            //double push
                            self.moves[first_unused] =
                                encode_move(lsb, unsafe { lsb.sub_unchecked(16) }, None, NO_FLAG);
                            first_unused += 1;
                        }
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
        }
        first_unused
    }

    pub fn castling_moves(&mut self, board: &Board, mut first_unused: usize) -> usize {
        match board.side_to_move {
            Colour::White => {
                if (board.castling & 0b0000_0001) > 0 {
                    //white kingside castling rights
                    if get_bit(Square::F1, board.occupancies[BOTH]) == 0
                        && get_bit(Square::G1, board.occupancies[BOTH]) == 0
                        && !is_attacked(Square::E1, Colour::Black, board)
                        && !is_attacked(Square::F1, Colour::Black, board)
                    //g1 checked later
                    {
                        self.moves[first_unused] =
                            encode_move(Square::E1, Square::G1, None, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }

                if (board.castling & 0b0000_0010) > 0 {
                    //white queenside
                    if get_bit(Square::B1, board.occupancies[BOTH]) == 0
                        && get_bit(Square::C1, board.occupancies[BOTH]) == 0
                        && get_bit(Square::D1, board.occupancies[BOTH]) == 0
                        && !is_attacked(Square::E1, Colour::Black, board)
                        && !is_attacked(Square::D1, Colour::Black, board)
                    {
                        self.moves[first_unused] =
                            encode_move(Square::E1, Square::C1, None, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }
            }
            Colour::Black => {
                if (board.castling & 0b0000_0100) > 0 {
                    //black kingside
                    if get_bit(Square::G8, board.occupancies[BOTH]) == 0
                        && get_bit(Square::F8, board.occupancies[BOTH]) == 0
                        && !is_attacked(Square::E8, Colour::White, board)
                        && !is_attacked(Square::F8, Colour::White, board)
                    {
                        self.moves[first_unused] =
                            encode_move(Square::E8, Square::G8, None, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }

                if (board.castling & 0b0000_1000) > 0 {
                    //black queenside
                    if get_bit(Square::D8, board.occupancies[BOTH]) == 0
                        && get_bit(Square::C8, board.occupancies[BOTH]) == 0
                        && get_bit(Square::B8, board.occupancies[BOTH]) == 0
                        && !is_attacked(Square::E8, Colour::White, board)
                        && !is_attacked(Square::D8, Colour::White, board)
                    {
                        self.moves[first_unused] =
                            encode_move(Square::E8, Square::C8, None, CASTLING_FLAG);
                        first_unused += 1;
                    }
                }
            }
        }
        first_unused
    }

    pub fn gen_pawn_captures(&mut self, board: &Board, mut first_unused: usize) -> usize {
        match board.side_to_move {
            Colour::White => {
                let mut bitboard = board.bitboards[Piece::WP];

                while let Some(lsb) = lsfb(bitboard) {
                    let mut attacks = WP_ATTACKS[lsb]
                        & match board.en_passant {
                            None => board.occupancies[BLACK],
                            Some(k) => set_bit(k, board.occupancies[BLACK]),
                        };

                    while let Some(lsb_attack) = lsfb(attacks) {
                        //promotions that are also captures
                        if rank(lsb) == 6 {
                            // white promotion
                            self.moves[first_unused] = encode_move(
                                lsb,
                                lsb_attack,
                                Some(PieceType::Queen),
                                PROMOTION_FLAG,
                            );
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, Some(PieceType::Rook), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] = encode_move(
                                lsb,
                                lsb_attack,
                                Some(PieceType::Bishop),
                                PROMOTION_FLAG,
                            );
                            first_unused += 1;
                            self.moves[first_unused] = encode_move(
                                lsb,
                                lsb_attack,
                                Some(PieceType::Knight),
                                PROMOTION_FLAG,
                            );
                            first_unused += 1;
                        } else if board.en_passant.is_some()
                            && lsb_attack == unsafe { board.en_passant.unwrap_unchecked() }
                        {
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, None, EN_PASSANT_FLAG);
                            first_unused += 1;
                        } else {
                            self.moves[first_unused] = encode_move(lsb, lsb_attack, None, NO_FLAG);
                            first_unused += 1;
                        } //list to return here
                        attacks = pop_bit(lsb_attack, attacks);
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
            Colour::Black => {
                let mut bitboard = board.bitboards[Piece::BP];

                while let Some(lsb) = lsfb(bitboard) {
                    let mut attacks = BP_ATTACKS[lsb]
                        & match board.en_passant {
                            None => board.occupancies[WHITE],
                            Some(k) => set_bit(k, board.occupancies[WHITE]),
                        };

                    while let Some(lsb_attack) = lsfb(attacks) {
                        //promotions that are also captures
                        if rank(lsb) == 1 {
                            // white promotion
                            self.moves[first_unused] = encode_move(
                                lsb,
                                lsb_attack,
                                Some(PieceType::Queen),
                                PROMOTION_FLAG,
                            );
                            first_unused += 1;
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, Some(PieceType::Rook), PROMOTION_FLAG);
                            first_unused += 1;
                            self.moves[first_unused] = encode_move(
                                lsb,
                                lsb_attack,
                                Some(PieceType::Bishop),
                                PROMOTION_FLAG,
                            );
                            first_unused += 1;
                            self.moves[first_unused] = encode_move(
                                lsb,
                                lsb_attack,
                                Some(PieceType::Knight),
                                PROMOTION_FLAG,
                            );
                            first_unused += 1;
                        } else if board.en_passant.is_some()
                            && lsb_attack == unsafe { board.en_passant.unwrap_unchecked() }
                        {
                            self.moves[first_unused] =
                                encode_move(lsb, lsb_attack, None, EN_PASSANT_FLAG);
                            first_unused += 1;
                        } else {
                            self.moves[first_unused] = encode_move(lsb, lsb_attack, None, NO_FLAG);
                            first_unused += 1;
                        } //list to return here
                        attacks = pop_bit(lsb_attack, attacks);
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
        }
        first_unused
    }

    pub fn gen_knight_moves<const CAPS_ONLY: bool>(
        &mut self,
        board: &Board,
        mut first_unused: usize,
    ) -> usize {
        let (piece, occs, opps) = match board.side_to_move {
            Colour::White => (
                Piece::WN,
                board.occupancies[WHITE],
                board.occupancies[BLACK],
            ),
            Colour::Black => (
                Piece::BN,
                board.occupancies[BLACK],
                board.occupancies[WHITE],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let mut attacks = N_ATTACKS[lsb] & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, None, NO_FLAG);
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
            Colour::White => (
                Piece::WB,
                board.occupancies[WHITE],
                board.occupancies[BLACK],
            ),
            Colour::Black => (
                Piece::BB,
                board.occupancies[BLACK],
                board.occupancies[WHITE],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let idx = lsb as usize;
            let mut attacks = get_bishop_attacks(idx, board.occupancies[BOTH])
                & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, None, NO_FLAG);
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
            Colour::White => (
                Piece::WR,
                board.occupancies[WHITE],
                board.occupancies[BLACK],
            ),
            Colour::Black => (
                Piece::BR,
                board.occupancies[BLACK],
                board.occupancies[WHITE],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let idx = lsb as usize;
            let mut attacks = get_rook_attacks(idx, board.occupancies[BOTH])
                & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, None, NO_FLAG);
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
            Colour::White => (
                Piece::WQ,
                board.occupancies[WHITE],
                board.occupancies[BLACK],
            ),
            Colour::Black => (
                Piece::BQ,
                board.occupancies[BLACK],
                board.occupancies[WHITE],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let idx = lsb as usize;
            let mut attacks = get_queen_attacks(idx, board.occupancies[BOTH])
                & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, None, NO_FLAG);
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
            Colour::White => (
                Piece::WK,
                board.occupancies[WHITE],
                board.occupancies[BLACK],
            ),
            Colour::Black => (
                Piece::BK,
                board.occupancies[BLACK],
                board.occupancies[WHITE],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let mut attacks = K_ATTACKS[lsb] & if CAPS_ONLY { opps } else { !occs };

            while let Some(lsb_attack) = lsfb(attacks) {
                self.moves[first_unused] = encode_move(lsb, lsb_attack, None, NO_FLAG);
                first_unused += 1;
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }

        first_unused
    }

    #[must_use]
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

pub fn get_smallest_attack(b: &mut Board, square: Square) -> Move {
    //NOTE: for speed this does not take pins into account
    //attacked BY colour

    //SAFETY for these: we know that an attacker exists

    match b.side_to_move {
        Colour::White => {
            let pawn_attackers = BP_ATTACKS[square] & b.bitboards[Piece::WP];
            if pawn_attackers > 0 {
                let sq_from = unsafe { lsfb(pawn_attackers).unwrap_unchecked() };
                return match rank(square) {
                    7 => encode_move(sq_from, square, Some(PieceType::Queen), PROMOTION_FLAG),
                    //no point considering underpromotions
                    _ => encode_move(sq_from, square, None, NO_FLAG),
                };
            }
            let knight_attackers = N_ATTACKS[square] & b.bitboards[Piece::WN];
            if knight_attackers > 0 {
                let sq_from = unsafe { lsfb(knight_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let king_attackers = K_ATTACKS[square] & b.bitboards[Piece::WK];
            if king_attackers > 0 {
                //only one king
                let sq_from = unsafe { lsfb(king_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let bishop_attacks = get_bishop_attacks(square as usize, b.occupancies[BOTH]);
            //use later to get queen attackers
            let bishop_attackers = bishop_attacks & b.bitboards[Piece::WB];
            if bishop_attackers > 0 {
                let sq_from = unsafe { lsfb(bishop_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let rook_attacks = get_rook_attacks(square as usize, b.occupancies[BOTH]);
            let rook_attackers = rook_attacks & b.bitboards[Piece::WR];
            if rook_attackers > 0 {
                let sq_from = unsafe { lsfb(rook_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let queen_attackers = (rook_attacks | bishop_attacks) & b.bitboards[Piece::WQ];
            if queen_attackers > 0 {
                let sq_from = unsafe { lsfb(queen_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
        }
        Colour::Black => {
            let pawn_attackers = WP_ATTACKS[square] & b.bitboards[Piece::BP];
            if pawn_attackers > 0 {
                let sq_from = unsafe { lsfb(pawn_attackers).unwrap_unchecked() };
                return match rank(square) {
                    0 => encode_move(sq_from, square, Some(PieceType::Queen), PROMOTION_FLAG),
                    //no point considering underpromotions
                    _ => encode_move(sq_from, square, None, NO_FLAG),
                };
            }
            let knight_attackers = N_ATTACKS[square] & b.bitboards[Piece::BN];
            if knight_attackers > 0 {
                let sq_from = unsafe { lsfb(knight_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let king_attackers = K_ATTACKS[square] & b.bitboards[Piece::BK];
            if king_attackers > 0 {
                //only one king
                let sq_from = unsafe { lsfb(king_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let bishop_attacks = get_bishop_attacks(square as usize, b.occupancies[BOTH]);
            //use later to get queen attackers
            let bishop_attackers = bishop_attacks & b.bitboards[Piece::BB];
            if bishop_attackers > 0 {
                let sq_from = unsafe { lsfb(bishop_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let rook_attacks = get_rook_attacks(square as usize, b.occupancies[BOTH]);
            let rook_attackers = rook_attacks & b.bitboards[Piece::BR];
            if rook_attackers > 0 {
                let sq_from = unsafe { lsfb(rook_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
            let queen_attackers = (rook_attacks | bishop_attacks) & b.bitboards[Piece::BQ];
            if queen_attackers > 0 {
                let sq_from = unsafe { lsfb(queen_attackers).unwrap_unchecked() };
                return encode_move(sq_from, square, None, NO_FLAG);
            }
        }
    }
    NULL_MOVE
}

pub static RAY_BETWEEN: [[BitBoard; 64]; 64] = {
    const fn in_between(mut sq1: Square, mut sq2: Square) -> BitBoard {
        if sq1 as usize == sq2 as usize {
            return 0u64;
        } else if sq1 as usize > sq2 as usize {
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

        while ((sq1 as i8 + dx + dy) as usize) < sq2 as usize {
            res |= set_bit(unsafe { Square::from((sq1 as i8 + dx + dy) as u8) }, 0);
            sq1 = unsafe { Square::from((sq1 as i8 + dx + dy) as u8) };
        }
        res
    }

    let mut res = [[0u64; 64]; 64];
    let mut from = 0;
    while from < 64 {
        let mut to = 0;
        while to < 64 {
            res[from][to] = in_between(unsafe { Square::from(from as u8) }, unsafe {
                Square::from(to as u8)
            });
            to += 1;
        }
        from += 1;
    }
    res
};

#[must_use]
pub fn check_en_passant(m: Move, b: &Board) -> bool {
    //checks en passant edge case where en passant reveals check on the king
    match m.piece_moved(b) {
        Piece::WP => {
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[BOTH]);
            relevant_blockers =
                pop_bit(unsafe { m.square_to().sub_unchecked(8) }, relevant_blockers);
            //SAFETY: there MUST be a king on the board
            get_rook_attacks(
                unsafe { lsfb(b.bitboards[Piece::WK]).unwrap_unchecked() } as usize,
                relevant_blockers,
            ) & (b.bitboards[Piece::BR] | b.bitboards[Piece::BQ])
                == 0
        }
        Piece::BP => {
            let mut relevant_blockers = pop_bit(m.square_from(), b.occupancies[BOTH]);
            relevant_blockers =
                pop_bit(unsafe { m.square_to().add_unchecked(8) }, relevant_blockers);
            //SAFETY: there MUST be a king on the board
            get_rook_attacks(
                unsafe { lsfb(b.bitboards[Piece::BK]).unwrap_unchecked() } as usize,
                relevant_blockers,
            ) & (b.bitboards[Piece::WR] | b.bitboards[Piece::WQ])
                == 0
        }
        _ => unreachable!(),
    }
}
