use crate::board::{BitBoard, Board, Colour};
use crate::helper::{file, get_bit, lsfb, pop_bit, rank, set_bit, MAX_MOVES};
use crate::magic::{
    get_bishop_attacks, get_queen_attacks, get_rook_attacks, BP_ATTACKS, K_ATTACKS, N_ATTACKS,
    WP_ATTACKS,
};
use crate::r#move::{
    encode_move, Move, MoveList, CASTLING_FLAG, EN_PASSANT_FLAG, NO_FLAG, NULL_MOVE, PROMOTION_FLAG,
};

use crate::types::{CastlingType, OccupancyIndex, Piece, PieceType, Square};

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

pub const CASTLING_MASKS: [u8; 4] = [0b0000_0001, 0b0000_0010, 0b0000_0100, 0b0000_1000];
pub const CASTLING_PATHS: [BitBoard; 4] = [
    set_bit(Square::G1, 0) | set_bit(Square::F1, 0),
    set_bit(Square::D1, 0) | set_bit(Square::C1, 0) | set_bit(Square::B1, 0),
    set_bit(Square::G8, 0) | set_bit(Square::F8, 0),
    set_bit(Square::D8, 0) | set_bit(Square::C8, 0) | set_bit(Square::B8, 0),
];

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
                || get_bishop_attacks(square, board.occupancies[OccupancyIndex::BothOccupancies])
                    & (board.bitboards[Piece::WB] | board.bitboards[Piece::WQ])
                    != 0
                || get_rook_attacks(square, board.occupancies[OccupancyIndex::BothOccupancies])
                    & (board.bitboards[Piece::WR] | board.bitboards[Piece::WQ])
                    != 0
        }
        Colour::Black => {
            //leapers then sliders
            WP_ATTACKS[square] & board.bitboards[Piece::BP] != 0
                || N_ATTACKS[square] & board.bitboards[Piece::BN] != 0
                || K_ATTACKS[square] & board.bitboards[Piece::BK] != 0
                || get_bishop_attacks(square, board.occupancies[OccupancyIndex::BothOccupancies])
                    & (board.bitboards[Piece::BB] | board.bitboards[Piece::BQ])
                    != 0
                || get_rook_attacks(square, board.occupancies[OccupancyIndex::BothOccupancies])
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

    #[rustfmt::skip]
    pub fn pawn_push_moves(&mut self, board: &Board, mut first_unused: usize) -> usize {
        let (mut pawns, seventh_rank) = match board.side_to_move {
            Colour::White => (board.bitboards[Piece::WP], 6),
            Colour::Black => (board.bitboards[Piece::BP], 1),
        };

        let offset = match board.side_to_move {
            Colour::White => Square::add_unchecked,
            Colour::Black => Square::sub_unchecked,
        };

        let add_promo = |mvs: &mut [Move; MAX_MOVES],
                         pc: PieceType,
                         from: Square,
                         to: Square,
                         mut idx: usize| {
            mvs[idx] = encode_move(from, to, Some(pc), PROMOTION_FLAG);
            idx += 1;
            idx
        };

        while let Some(from) = lsfb(pawns) {
            let to = unsafe { offset(from, 8) };
            if get_bit(to, board.occupancies[OccupancyIndex::BothOccupancies]) != 0 {
                pawns = pop_bit(from, pawns);
                continue;
            }

            if rank(from) == seventh_rank {
                //promotion
                //the piece type is passed into encode_move because only 2 bits are used to encode
                //the promoted piece (and flag is used to detect if there is one)
                first_unused = add_promo(&mut self.moves, PieceType::Queen, from, to, first_unused);
                first_unused = add_promo(&mut self.moves, PieceType::Rook, from, to, first_unused);
                first_unused =
                    add_promo(&mut self.moves, PieceType::Bishop, from, to, first_unused);
                first_unused =
                    add_promo(&mut self.moves, PieceType::Knight, from, to, first_unused);
            } else {
                self.moves[first_unused] = encode_move(from, to, None, NO_FLAG);
                first_unused += 1;
            }

            let dp = unsafe { offset(from, 16) };
            if rank(from) == 7 - seventh_rank
                && get_bit(dp, board.occupancies[OccupancyIndex::BothOccupancies]) == 0
            {
                self.moves[first_unused] = encode_move(from, dp, None, NO_FLAG);
                first_unused += 1;
            }
            pawns = pop_bit(from, pawns);
        }

        first_unused
    }

    pub fn castling_moves(&mut self, board: &Board, mut first_unused: usize) -> usize {
        let add_castling =
            |mvs: &mut [Move; MAX_MOVES], from: Square, to: Square, mut idx: usize| {
                mvs[idx] = encode_move(from, to, None, CASTLING_FLAG);
                idx += 1;
                idx
            };
        match board.side_to_move {
            Colour::White => {
                if (board.castling & CASTLING_MASKS[CastlingType::WhiteKingside]) > 0 {
                    //white kingside castling rights
                    if board.occupancies[OccupancyIndex::BothOccupancies]
                        & CASTLING_PATHS[CastlingType::WhiteKingside]
                        == 0
                        && !is_attacked(Square::E1, Colour::Black, board)
                        && !is_attacked(Square::F1, Colour::Black, board)
                    //g1 checked later
                    {
                        first_unused =
                            add_castling(&mut self.moves, Square::E1, Square::G1, first_unused);
                    }
                }

                if (board.castling & CASTLING_MASKS[CastlingType::WhiteQueenside]) > 0 {
                    if board.occupancies[OccupancyIndex::BothOccupancies]
                        & CASTLING_PATHS[CastlingType::WhiteQueenside]
                        == 0
                        && !is_attacked(Square::E1, Colour::Black, board)
                        && !is_attacked(Square::D1, Colour::Black, board)
                    {
                        first_unused =
                            add_castling(&mut self.moves, Square::E1, Square::C1, first_unused);
                    }
                }
            }
            Colour::Black => {
                if (board.castling & CASTLING_MASKS[CastlingType::BlackKingside]) > 0 {
                    if board.occupancies[OccupancyIndex::BothOccupancies]
                        & CASTLING_PATHS[CastlingType::BlackKingside]
                        == 0
                        && !is_attacked(Square::E8, Colour::White, board)
                        && !is_attacked(Square::F8, Colour::White, board)
                    {
                        first_unused =
                            add_castling(&mut self.moves, Square::E8, Square::G8, first_unused);
                    }
                }

                if (board.castling & CASTLING_MASKS[CastlingType::BlackQueenside]) > 0 {
                    //black queenside
                    if board.occupancies[OccupancyIndex::BothOccupancies]
                        & CASTLING_PATHS[CastlingType::BlackQueenside]
                        == 0
                        && !is_attacked(Square::E8, Colour::White, board)
                        && !is_attacked(Square::D8, Colour::White, board)
                    {
                        first_unused =
                            add_castling(&mut self.moves, Square::E8, Square::C8, first_unused);
                    }
                }
            }
        }
        first_unused
    }

    pub fn gen_pawn_captures(&mut self, board: &Board, mut first_unused: usize) -> usize {
        let (mut pawns, seventh_rank, attacks) = match board.side_to_move {
            Colour::White => (board.bitboards[Piece::WP], 6, WP_ATTACKS),
            Colour::Black => (board.bitboards[Piece::BP], 1, BP_ATTACKS),
        };

        let opps = match board.side_to_move {
            Colour::White => OccupancyIndex::BlackOccupancies,
            Colour::Black => OccupancyIndex::WhiteOccupancies,
        };
        let add_promo = |mvs: &mut [Move; MAX_MOVES],
                         pc: PieceType,
                         from: Square,
                         to: Square,
                         mut idx: usize| {
            mvs[idx] = encode_move(from, to, Some(pc), PROMOTION_FLAG);
            idx += 1;
            idx
        };

        while let Some(from) = lsfb(pawns) {
            let targets = match board.en_passant {
                None => board.occupancies[opps],
                Some(k) => set_bit(k, board.occupancies[opps]),
            };
            let mut attacks = attacks[from] & targets;
            while let Some(to) = lsfb(attacks) {
                if rank(from) == seventh_rank {
                    first_unused =
                        add_promo(&mut self.moves, PieceType::Queen, from, to, first_unused);
                    first_unused =
                        add_promo(&mut self.moves, PieceType::Rook, from, to, first_unused);
                    first_unused =
                        add_promo(&mut self.moves, PieceType::Bishop, from, to, first_unused);
                    first_unused =
                        add_promo(&mut self.moves, PieceType::Knight, from, to, first_unused);
                } else if board.en_passant.is_some()
                    && to == unsafe { board.en_passant.unwrap_unchecked() }
                {
                    self.moves[first_unused] = encode_move(from, to, None, EN_PASSANT_FLAG);
                    first_unused += 1;
                } else {
                    self.moves[first_unused] = encode_move(from, to, None, NO_FLAG);
                    first_unused += 1;
                }
                attacks = pop_bit(to, attacks);
            }
            pawns = pop_bit(from, pawns);
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
                board.occupancies[OccupancyIndex::WhiteOccupancies],
                board.occupancies[OccupancyIndex::BlackOccupancies],
            ),
            Colour::Black => (
                Piece::BN,
                board.occupancies[OccupancyIndex::BlackOccupancies],
                board.occupancies[OccupancyIndex::WhiteOccupancies],
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
                board.occupancies[OccupancyIndex::WhiteOccupancies],
                board.occupancies[OccupancyIndex::BlackOccupancies],
            ),
            Colour::Black => (
                Piece::BB,
                board.occupancies[OccupancyIndex::BlackOccupancies],
                board.occupancies[OccupancyIndex::WhiteOccupancies],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let idx = lsb as usize;
            let mut attacks =
                get_bishop_attacks(idx, board.occupancies[OccupancyIndex::BothOccupancies])
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
                board.occupancies[OccupancyIndex::WhiteOccupancies],
                board.occupancies[OccupancyIndex::BlackOccupancies],
            ),
            Colour::Black => (
                Piece::BR,
                board.occupancies[OccupancyIndex::BlackOccupancies],
                board.occupancies[OccupancyIndex::WhiteOccupancies],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let idx = lsb as usize;
            let mut attacks =
                get_rook_attacks(idx, board.occupancies[OccupancyIndex::BothOccupancies])
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
                board.occupancies[OccupancyIndex::WhiteOccupancies],
                board.occupancies[OccupancyIndex::BlackOccupancies],
            ),
            Colour::Black => (
                Piece::BQ,
                board.occupancies[OccupancyIndex::BlackOccupancies],
                board.occupancies[OccupancyIndex::WhiteOccupancies],
            ),
        };

        let mut bitboard = board.bitboards[piece];

        while let Some(lsb) = lsfb(bitboard) {
            let idx = lsb as usize;
            let mut attacks =
                get_queen_attacks(idx, board.occupancies[OccupancyIndex::BothOccupancies])
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
                board.occupancies[OccupancyIndex::WhiteOccupancies],
                board.occupancies[OccupancyIndex::BlackOccupancies],
            ),
            Colour::Black => (
                Piece::BK,
                board.occupancies[OccupancyIndex::BlackOccupancies],
                board.occupancies[OccupancyIndex::WhiteOccupancies],
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

#[rustfmt::skip]
pub fn get_smallest_attack(b: &mut Board, square: Square) -> Move {
    //NOTE: for speed this does not take pins into account
    //attacked BY colour

    //SAFETY for these: we know that an attacker exists

    let (pawn_piece, knight_piece, king_piece, bishop_piece, rook_piece, queen_piece, 
         pawn_attacks, promotion_rank) = match b.side_to_move {
        Colour::White => (
            Piece::WP, Piece::WN, Piece::WK, Piece::WB, Piece::WR, Piece::WQ,
            BP_ATTACKS[square], 7
        ),
        Colour::Black => (
            Piece::BP, Piece::BN, Piece::BK, Piece::BB, Piece::BR, Piece::BQ,
            WP_ATTACKS[square], 0
        ),
    };

    let pawn_attackers = pawn_attacks & b.bitboards[pawn_piece];
    if pawn_attackers > 0 {
        let sq_from = unsafe { lsfb(pawn_attackers).unwrap_unchecked() };
        return match rank(square) {
            r if r == promotion_rank => encode_move(sq_from, square, Some(PieceType::Queen), PROMOTION_FLAG),
            // no point considering underpromotions
            _ => encode_move(sq_from, square, None, NO_FLAG),
        };
    }

    let knight_attackers = N_ATTACKS[square] & b.bitboards[knight_piece];
    if knight_attackers > 0 {
        let sq_from = unsafe { lsfb(knight_attackers).unwrap_unchecked() };
        return encode_move(sq_from, square, None, NO_FLAG);
    }

    let king_attackers = K_ATTACKS[square] & b.bitboards[king_piece];
    if king_attackers > 0 {
        let sq_from = unsafe { lsfb(king_attackers).unwrap_unchecked() };
        return encode_move(sq_from, square, None, NO_FLAG);
    }

    let bishop_attacks = get_bishop_attacks(
        square as usize,
        b.occupancies[OccupancyIndex::BothOccupancies],
    );
    let rook_attacks = get_rook_attacks(
        square as usize,
        b.occupancies[OccupancyIndex::BothOccupancies],
    );

    let bishop_attackers = bishop_attacks & b.bitboards[bishop_piece];
    if bishop_attackers > 0 {
        let sq_from = unsafe { lsfb(bishop_attackers).unwrap_unchecked() };
        return encode_move(sq_from, square, None, NO_FLAG);
    }

    let rook_attackers = rook_attacks & b.bitboards[rook_piece];
    if rook_attackers > 0 {
        let sq_from = unsafe { lsfb(rook_attackers).unwrap_unchecked() };
        return encode_move(sq_from, square, None, NO_FLAG);
    }

    let queen_attackers = (rook_attacks | bishop_attacks) & b.bitboards[queen_piece];
    if queen_attackers > 0 {
        let sq_from = unsafe { lsfb(queen_attackers).unwrap_unchecked() };
        return encode_move(sq_from, square, None, NO_FLAG);
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
            let mut relevant_blockers = pop_bit(
                m.square_from(),
                b.occupancies[OccupancyIndex::BothOccupancies],
            );
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
            let mut relevant_blockers = pop_bit(
                m.square_from(),
                b.occupancies[OccupancyIndex::BothOccupancies],
            );
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
