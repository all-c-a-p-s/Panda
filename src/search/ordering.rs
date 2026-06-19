use crate::board::r#move::{Move, MoveList};
use crate::board::movegen::MovegenMode;
use crate::board::movegen::get_attackers;
use crate::search::thread::Thread;
use crate::search::{INFINITY, params};
use crate::util::types::{BLACK_PIECES, OccupancyIndex, Piece, PieceType, WHITE_PIECES};
use crate::{Board, Colour, MAX_MOVES, get_bishop_attacks, get_rook_attacks, lsfb, piece_type, read_param, set_bit};

const MVV: [i32; 6] = [0, 2400, 2400, 4800, 9600, 0];

//same as MG evaluation weights (haven't updated these in a while)
pub const SEE_VALUES: [i32; 6] = [85, 306, 322, 490, 925, 0];

impl Move {
    /// Static Exchange Evalutaion (SEE):
    /// If we already have an advantage of 'threshold', and play this move, will we still be ahead
    /// after the dust settles?
    #[must_use]
    pub fn see(self, b: &Board, threshold: i32) -> bool {
        // Iterative approach to SEE inspired by Ethereal.
        let sq_from = self.square_from();
        let sq_to = self.square_to();

        let mut next_victim = if self.is_promotion() {
            match b.side_to_move {
                //only consider queen promotions
                Colour::White => Piece::WQ,
                Colour::Black => Piece::BQ,
            }
        } else {
            self.piece_moved(b)
        };

        let mut balance = match b.pieces_array[sq_to] {
            Some(k) => SEE_VALUES[piece_type(k)],
            None => 0,
        } + threshold;

        if self.is_promotion() {
            balance += SEE_VALUES[PieceType::Queen] - SEE_VALUES[PieceType::Pawn];
        } else if self.is_en_passant() {
            balance += SEE_VALUES[PieceType::Pawn];
        }

        if balance < 0 {
            //bad even in best case
            return false;
        }

        balance -= SEE_VALUES[piece_type(next_victim)];

        if balance >= 0 {
            //good even in worst case
            return true;
        }

        let bishop_attackers =
            b.bitboards[Piece::WB] | b.bitboards[Piece::BB] | b.bitboards[Piece::WQ] | b.bitboards[Piece::BQ];
        let rook_attackers =
            b.bitboards[Piece::WR] | b.bitboards[Piece::BR] | b.bitboards[Piece::WQ] | b.bitboards[Piece::BQ];

        let mut occupancies =
            b.occupancies[OccupancyIndex::BothOccupancies] ^ (set_bit(sq_from, 0) | set_bit(sq_to, 0));

        let mut attackers =
            get_attackers(sq_to, Colour::White, b, occupancies) | get_attackers(sq_to, Colour::Black, b, occupancies);

        let mut colour = b.side_to_move.opponent();

        loop {
            let side_attackers = attackers
                & b.occupancies[match colour {
                    Colour::White => OccupancyIndex::WhiteOccupancies,
                    Colour::Black => OccupancyIndex::BlackOccupancies,
                }];
            //doesn't matter that actual board struct isn't getting updated because attackers
            //that get traded off will get popped from the attackers bitboard

            if side_attackers == 0 {
                break;
            }

            let pieces = match colour {
                Colour::White => WHITE_PIECES,
                Colour::Black => BLACK_PIECES,
            };

            for piece in pieces {
                if side_attackers & b.bitboards[piece] > 0 {
                    next_victim = piece;
                    break;
                }
            }

            //SAFETY: if this was zero we would have broken above
            occupancies ^= set_bit(unsafe { lsfb(side_attackers & b.bitboards[next_victim]).unwrap_unchecked() }, 0);

            if piece_type(next_victim) == PieceType::Pawn
                || piece_type(next_victim) == PieceType::Bishop
                || piece_type(next_victim) == PieceType::Queen
            {
                //only diagonal moves can reveal new diagonal attackers
                attackers |= get_bishop_attacks(sq_to as usize, occupancies) & bishop_attackers;
            }

            if piece_type(next_victim) == PieceType::Rook || piece_type(next_victim) == PieceType::Queen {
                //same for rook attacks
                attackers |= get_rook_attacks(sq_to as usize, occupancies) & rook_attackers;
            }

            attackers &= occupancies;
            colour = colour.opponent();

            balance = -balance - 1 - SEE_VALUES[piece_type(next_victim)];

            if balance >= 0 {
                //if last move was king move and opponent still has attackers, the move
                //must have been illegal
                if piece_type(next_victim) == PieceType::King
                    && (attackers
                        & b.occupancies[match colour {
                            Colour::White => OccupancyIndex::WhiteOccupancies,
                            Colour::Black => OccupancyIndex::BlackOccupancies,
                        }])
                        > 0
                {
                    colour = colour.opponent();
                }
                break;
            }
        }

        //side to move after the loop loses
        b.side_to_move != colour
    }

    /// Still doesn't account for pins but computes the material balance after a move if both sides
    /// play to maximise material.
    #[must_use]
    pub fn exact_see(self, b: &Board) -> i32 {
        let sq_from = self.square_from();
        let sq_to = self.square_to();

        let mut next_victim = if self.is_promotion() {
            match b.side_to_move {
                //only consider queen promotions
                Colour::White => Piece::WQ,
                Colour::Black => Piece::BQ,
            }
        } else {
            self.piece_moved(b)
        };

        // balance always relative to us
        let mut balance = match b.pieces_array[sq_to] {
            Some(k) => SEE_VALUES[piece_type(k)],
            None => 0,
        };

        if self.is_promotion() {
            balance += SEE_VALUES[PieceType::Queen] - SEE_VALUES[PieceType::Pawn];
        } else if self.is_en_passant() {
            balance += SEE_VALUES[PieceType::Pawn];
        }

        let bishop_attackers =
            b.bitboards[Piece::WB] | b.bitboards[Piece::BB] | b.bitboards[Piece::WQ] | b.bitboards[Piece::BQ];
        let rook_attackers =
            b.bitboards[Piece::WR] | b.bitboards[Piece::BR] | b.bitboards[Piece::WQ] | b.bitboards[Piece::BQ];

        let mut occupancies =
            b.occupancies[OccupancyIndex::BothOccupancies] ^ (set_bit(sq_from, 0) | set_bit(sq_to, 0));

        let mut attackers =
            get_attackers(sq_to, Colour::White, b, occupancies) | get_attackers(sq_to, Colour::Black, b, occupancies);

        let mut colour = b.side_to_move.opponent();

        let mut dp = [-INFINITY; 32];

        dp[0] = balance;

        let mut i = 1;

        loop {
            let side_attackers = attackers
                & b.occupancies[match colour {
                    Colour::White => OccupancyIndex::WhiteOccupancies,
                    Colour::Black => OccupancyIndex::BlackOccupancies,
                }];

            //doesn't matter that actual board struct isn't getting updated because attackers
            //that get traded off will get popped from the attackers bitboard

            if side_attackers == 0 {
                i -= 1;
                break;
            }

            if piece_type(next_victim) == PieceType::King {
                dp[i] = INFINITY;
                break;
            }

            dp[i] = -dp[i - 1] + SEE_VALUES[piece_type(next_victim)];

            let pieces = match colour {
                Colour::White => WHITE_PIECES,
                Colour::Black => BLACK_PIECES,
            };

            for piece in pieces {
                if side_attackers & b.bitboards[piece] > 0 {
                    next_victim = piece;
                    break;
                }
            }

            //SAFETY: if this was zero we would have broken above
            occupancies ^= set_bit(unsafe { lsfb(side_attackers & b.bitboards[next_victim]).unwrap_unchecked() }, 0);

            if piece_type(next_victim) == PieceType::Pawn
                || piece_type(next_victim) == PieceType::Bishop
                || piece_type(next_victim) == PieceType::Queen
            {
                //only diagonal moves can reveal new diagonal attackers
                attackers |= get_bishop_attacks(sq_to as usize, occupancies) & bishop_attackers;
            }

            if piece_type(next_victim) == PieceType::Rook || piece_type(next_victim) == PieceType::Queen {
                //same for rook attacks
                attackers |= get_rook_attacks(sq_to as usize, occupancies) & rook_attackers;
            }

            attackers &= occupancies;
            colour = colour.opponent();

            i += 1;
        }

        while i > 0 {
            i -= 1;
            dp[i] = dp[i].min(-dp[i + 1]);
        }

        dp[0]
    }

    /// Scores a move based on this order
    /// - TT Move
    /// - Queen Promotion
    /// - Good Captures (sorted by MVV-caphist)
    /// - Killers     |
    /// - Quiets      |----- these are also subject to continuation bonuses
    /// - Losing Captures
    /// - Underpromotion
    pub fn score_move(
        self,
        b: &mut Board,
        s: &Thread,
        hash_move: &Move,
        depth: u8,
        pv_node: bool,
        cutnode: bool,
    ) -> i32 {
        let sq = self.square_to();

        if self.is_null() {
            -INFINITY
            //important for this to come before checking hash move
            //otherwise null move can get given hash move score
        } else if self == *hash_move {
            read_param!(HASH_MOVE_SCORE)
        } else if self.is_capture(b) || self.is_en_passant() {
            //we are already in the segment of good/bad captures
            //and we only care about scores relative to the rest of the segment
            //so no need to add good/bad capture bonus

            let victim_type = if self.is_en_passant() { PieceType::Pawn } else { piece_type(self.piece_captured(b)) };
            let pc = self.piece_moved(b);
            let hist = s.info.caphist[pc][sq][victim_type];

            //at high depths we can put more effort into our move ordering because there's a
            //greater reward for optimal order
            if depth >= 16 && (pv_node || cutnode) {
                let v = self.exact_see(b);

                v * 10 + hist
            } else {
                hist + MVV[victim_type]
            }
        } else if self.is_promotion() {
            match self.promoted_piece() {
                //promotions sorted by likelihood to be good
                PieceType::Queen => read_param!(QUEEN_PROMOTION),
                PieceType::Knight => read_param!(UNDER_PROMOTION),
                PieceType::Rook => read_param!(UNDER_PROMOTION),
                PieceType::Bishop => read_param!(UNDER_PROMOTION),
                _ => unreachable!(),
            }
        } else {
            let mut score = s.get_overall_history(self, b, self.piece_moved(b));

            if s.info.killer_moves[s.ply] == Some(self) {
                score += read_param!(FIRST_KILLER_MOVE);
            }
            score
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum MovePickerStage {
    HashMove,
    NoisyQueenPromotions,
    QuietQueenPromotions,
    GoodCaps,
    Killers,
    Quiets,
    BadCaps,
    NoisyUnderpromotions,
    QuietUnderpromotions,
}

pub struct MovePicker {
    pub stage: MovePickerStage,
    pub done_probcut: bool,
    pub doing_probcut: bool,
    pub generated: bool,
    pub idx: usize,
    pub skip_quiets: bool,
    pub adjusted_down: bool,
    scores: [i32; MAX_MOVES],
}

impl MovePicker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            stage: MovePickerStage::HashMove,
            done_probcut: false,
            doing_probcut: false,
            generated: false,
            idx: 0,
            scores: [0; MAX_MOVES],
            adjusted_down: false,
            skip_quiets: false,
        }
    }

    pub fn for_qsearch() -> Self {
        Self {
            stage: MovePickerStage::HashMove,
            done_probcut: false,
            doing_probcut: false,
            generated: false,
            idx: 0,
            skip_quiets: true,
            adjusted_down: false,
            scores: [0; MAX_MOVES],
        }
    }

    pub fn skip_quiets(&mut self, movelist: &MoveList) {
        self.skip_quiets = true;

        if self.stage == MovePickerStage::Quiets {
            self.stage = MovePickerStage::BadCaps;
            self.idx = movelist.used;
            self.generated = false;
        }
    }

    /// Gets next pseudo-legal move based on our ordering.
    /// Note each case in the function falls through.
    #[allow(clippy::too_many_arguments)]
    pub fn get_next(
        &mut self,
        hash_move: Move,
        killer: Option<Move>,
        counter: Option<Move>,
        b: &mut Board,
        movelist: &mut MoveList,
        good_caps: &mut MoveList,
        bad_caps: &mut MoveList,
        s: &Thread,
        depth: u8,
        pv_node: bool,
        cutnode: bool,
    ) -> Option<Move> {
        if self.stage == MovePickerStage::HashMove {
            self.stage = MovePickerStage::NoisyQueenPromotions;

            if !hash_move.is_null() && b.is_pseudo_legal(hash_move) {
                //still check pseudo-legal in case of hash collision
                return Some(hash_move);
            }
        }

        if self.stage == MovePickerStage::NoisyQueenPromotions {
            if !self.generated {
                movelist.gen_moves(b, MovegenMode::NoisyQueenPromotions);
                if self.idx < movelist.used {
                    self.score_between(
                        movelist,
                        self.idx,
                        movelist.used - 1,
                        b,
                        &hash_move,
                        s,
                        depth,
                        pv_node,
                        cutnode,
                    );
                }
                self.generated = true;
            }

            // This is a bit hacky: if we've done probcut, then the "used" count will be too high
            // for the promotion stage since it will also include good captures. Let's temporarily
            // decrease it for now so we don't have to worry about considering those yet.
            // Technically, this means noisy promotions will get mixed with quiet promotions after
            // probcut, but I don't think that should matter too much.
            if self.done_probcut && !self.adjusted_down {
                movelist.used -= good_caps.used;
                self.adjusted_down = true;
            }

            if self.idx < movelist.used {
                let mv = self.get_next_between(self.idx, movelist.used - 1, movelist);
                self.idx += 1;
                return Some(mv);
            } else {
                self.stage = MovePickerStage::QuietQueenPromotions;
                self.generated = self.done_probcut;
            }
        }

        if self.stage == MovePickerStage::QuietQueenPromotions {
            if !self.generated {
                movelist.gen_moves(b, MovegenMode::QuietQueenPromotions);
                if self.idx < movelist.used {
                    self.score_between(
                        movelist,
                        self.idx,
                        movelist.used - 1,
                        b,
                        &hash_move,
                        s,
                        depth,
                        pv_node,
                        cutnode,
                    );
                }
                self.generated = true;
            }

            if self.idx < movelist.used {
                let mv = self.get_next_between(self.idx, movelist.used - 1, movelist);
                self.idx += 1;

                return Some(mv);
            } else {
                self.stage = MovePickerStage::GoodCaps;
                self.generated = self.done_probcut;
            }
        }

        if self.stage == MovePickerStage::GoodCaps {
            if !self.generated {
                let mut caps = MoveList::empty();
                caps.gen_moves(b, MovegenMode::CapsOnly);
                (*good_caps, *bad_caps) = caps.separate_captures(b);
                movelist.extend_from(good_caps);
                if self.idx < movelist.used {
                    self.score_between(
                        movelist,
                        self.idx,
                        movelist.used - 1,
                        b,
                        &hash_move,
                        s,
                        depth,
                        pv_node,
                        cutnode,
                    );
                }
                self.generated = true;
            }

            // Rectifying after the above "hacky" comment.
            if self.done_probcut && self.adjusted_down {
                movelist.used += good_caps.used;
                self.adjusted_down = false;
            }

            if self.idx < movelist.used {
                let mv = self.get_next_between(self.idx, movelist.used - 1, movelist);
                self.idx += 1;

                return Some(mv);
            } else if self.doing_probcut {
                // signal to break the probcut loop
                self.doing_probcut = false;
                self.done_probcut = true;
                return None;
            } else {
                self.stage = MovePickerStage::Killers;
                self.generated = false;
            }
        }

        if self.stage == MovePickerStage::Killers {
            self.stage = if self.skip_quiets { MovePickerStage::BadCaps } else { MovePickerStage::Quiets };

            if let Some(mv) = killer
                && b.is_pseudo_legal(mv)
            {
                return Some(mv);
            }

            if let Some(mv) = counter
                && b.is_pseudo_legal(mv)
            {
                //this way we only try counter if we don't have a killer
                //could also try always trying counter
                return Some(mv);
            }
        }

        if !self.skip_quiets && self.stage == MovePickerStage::Quiets {
            if !self.generated {
                movelist.gen_moves(b, MovegenMode::QuietsOnly);
                if self.idx < movelist.used {
                    self.score_between(
                        movelist,
                        self.idx,
                        movelist.used - 1,
                        b,
                        &hash_move,
                        s,
                        depth,
                        pv_node,
                        cutnode,
                    );
                }
                self.generated = true;
            }

            if self.idx < movelist.used {
                let mv = self.get_next_between(self.idx, movelist.used - 1, movelist);
                self.idx += 1;
                return Some(mv);
            } else {
                self.stage = MovePickerStage::BadCaps;
                // don't set to true for probcut because we have generated the moves
                // but we haven't actually put them into the movelist and scored them
                self.generated = false;
            }
        }

        if self.stage == MovePickerStage::BadCaps {
            if !self.generated {
                movelist.extend_from(bad_caps);
                if self.idx < movelist.used {
                    self.score_between(
                        movelist,
                        self.idx,
                        movelist.used - 1,
                        b,
                        &hash_move,
                        s,
                        depth,
                        pv_node,
                        cutnode,
                    );
                }
                self.generated = true;
            }

            if self.idx < movelist.used {
                let mv = self.get_next_between(self.idx, movelist.used - 1, movelist);
                self.idx += 1;
                return Some(mv);
            } else {
                self.stage = MovePickerStage::NoisyUnderpromotions;
                self.generated = false;
            }
        }

        if !self.skip_quiets {
            if self.stage == MovePickerStage::NoisyUnderpromotions {
                if !self.generated {
                    movelist.gen_moves(b, MovegenMode::NoisyUnderpromotions);
                    if self.idx < movelist.used {
                        self.score_between(
                            movelist,
                            self.idx,
                            movelist.used - 1,
                            b,
                            &hash_move,
                            s,
                            depth,
                            pv_node,
                            cutnode,
                        );
                    }
                    self.generated = true;
                }

                if self.idx < movelist.used {
                    let mv = self.get_next_between(self.idx, movelist.used - 1, movelist);
                    self.idx += 1;
                    return Some(mv);
                } else {
                    self.stage = MovePickerStage::QuietUnderpromotions;
                    self.generated = false;
                }
            }

            if self.stage == MovePickerStage::QuietUnderpromotions {
                if !self.generated {
                    movelist.gen_moves(b, MovegenMode::QuietUnderpromotions);
                    if self.idx < movelist.used {
                        self.score_between(
                            movelist,
                            self.idx,
                            movelist.used - 1,
                            b,
                            &hash_move,
                            s,
                            depth,
                            pv_node,
                            cutnode,
                        );
                    }
                    self.generated = true;
                }

                if self.idx < movelist.used {
                    let mv = self.get_next_between(self.idx, movelist.used - 1, movelist);
                    self.idx += 1;
                    return Some(mv);
                }
            }
        }

        None
    }

    #[allow(clippy::too_many_arguments)]
    pub fn score_between(
        &mut self,
        movelist: &mut MoveList,
        l: usize,
        r: usize,
        b: &mut Board,
        hash_move: &Move,
        s: &Thread,
        depth: u8,
        pv_node: bool,
        cutnode: bool,
    ) {
        for i in l..=r {
            self.scores[i] = movelist.moves[i].score_move(b, s, hash_move, depth, pv_node, cutnode);
        }
    }

    fn get_next_between(&mut self, l: usize, r: usize, movelist: &mut MoveList) -> Move {
        let mut best = -INFINITY;
        let mut idx = l;
        for i in l..=r {
            if self.scores[i] > best {
                best = self.scores[i];
                idx = i;
            }
        }

        movelist.moves.swap(idx, l);
        self.scores.swap(idx, l);

        movelist.moves[l]
    }
}

impl MoveList {
    /// Returns good caps, bad caps
    pub fn separate_captures(&mut self, b: &mut Board) -> (Self, Self) {
        let (mut good_caps, mut bad_caps) = (MoveList::empty(), MoveList::empty());
        for &c in self.moves.iter().take(self.used) {
            if c.see(b, 0) {
                good_caps.moves[good_caps.used] = c;
                good_caps.used += 1;
            } else {
                bad_caps.moves[bad_caps.used] = c;
                bad_caps.used += 1;
            }
        }

        (good_caps, bad_caps)
    }

    pub fn extend_from(&mut self, other: &Self) {
        for &mv in other.moves.iter().take(other.used) {
            self.moves[self.used] = mv;
            self.used += 1;
        }
    }
}
