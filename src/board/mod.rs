pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod perft;
pub mod zobrist;

pub use magic::*;
pub use r#move::*;
pub use movegen::*;
pub use perft::*;
pub use zobrist::*;

use crate::search::REPETITION_TABLE_SIZE;
use crate::util::helper::{coordinate, count, lsfb, pop_bit, set_bit, square};
use crate::util::types::OccupancyIndex;
use crate::util::types::{Piece, Square};
use crate::util::uci::pretty_piece;

pub(crate) type BitBoard = u64;
pub(crate) const EMPTY: BitBoard = 0;

#[derive(Debug, Clone, Copy)]
pub struct Board {
    // Fundamental board state
    pub bitboards: [BitBoard; 12],
    pub pieces_array: [Option<Piece>; 64],
    pub occupancies: [BitBoard; 3], //white, black, both
    pub castling: u8,               //4 bits only should be used 0001 = wk, 0010 = wq, 0100 = bk, 1000 = bq
    pub en_passant: Option<Square>,
    pub side_to_move: Colour,
    pub fifty_move: usize,

    // Used in search
    pub last_move_null: bool,
    pub hash_key: u64,
    pub pawn_hash: u64,

    // Repetition Table used indexed by fifty move state to save memory.
    // The crucial invariant is that in ANY board state (including after NMP etc)
    // self.hash_key == self.repetition_table[self.fifty_moves].
    pub repetition_table: [u64; REPETITION_TABLE_SIZE],

    // Used in movegen
    pub checkers: BitBoard,
    pub pinned: BitBoard,
}

pub struct NullMoveUndo {
    ep: Option<Square>,
    pinned: BitBoard,
    hash_key: u64,
    hash_overwritten: u64,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Colour {
    White,
    Black,
}

impl Colour {
    #[must_use]
    pub fn opponent(&self) -> Self {
        match self {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

#[must_use]
pub fn ascii_to_piece(ascii: char) -> Piece {
    match ascii {
        'P' => Piece::WP,
        'N' => Piece::WN,
        'B' => Piece::WB,
        'R' => Piece::WR,
        'Q' => Piece::WQ,
        'K' => Piece::WK,
        'p' => Piece::BP,
        'n' => Piece::BN,
        'b' => Piece::BB,
        'r' => Piece::BR,
        'q' => Piece::BQ,
        'k' => Piece::BK,
        _ => panic!("invalid character in ascii_to_piece()"),
    }
}

impl Board {
    #[must_use]
    pub fn from(fen: &str) -> Self {
        let mut new_board = Board {
            bitboards: [EMPTY; 12],
            pieces_array: [None; 64],
            occupancies: [EMPTY; 3],
            castling: 0,
            en_passant: None,
            side_to_move: Colour::White,
            fifty_move: 0,
            last_move_null: false,
            hash_key: 0,
            pawn_hash: 0,
            repetition_table: [0; REPETITION_TABLE_SIZE],
            checkers: 0,
            pinned: 0,
        };

        let mut board_fen = String::new();
        let mut flags = 0; //index where board ends and flags start
        for i in fen.chars() {
            flags += 1;
            if i == ' ' {
                break;
            }
            board_fen += i.to_string().as_str();
        }

        let flags = (fen[flags..].split(' ')).clone().collect::<Vec<&str>>();

        match flags[0] {
            "w" => new_board.side_to_move = Colour::White,
            "b" => new_board.side_to_move = Colour::Black,
            _ => panic!("invalid colour to move flag in fen string"),
        }

        match flags[1] {
            "-" => new_board.castling = 0b0000_0000,
            "K" => new_board.castling = 0b0000_0001,
            "Q" => new_board.castling = 0b0000_0010,
            "k" => new_board.castling = 0b0000_0100,
            "q" => new_board.castling = 0b0000_1000,
            "KQ" => new_board.castling = 0b0000_0011,
            "Kk" => new_board.castling = 0b0000_0101,
            "Kq" => new_board.castling = 0b0000_1001,
            "Qk" => new_board.castling = 0b0000_0110,
            "Qq" => new_board.castling = 0b0000_1010,
            "kq" => new_board.castling = 0b0000_1100,
            "KQk" => new_board.castling = 0b0000_0111,
            "KQq" => new_board.castling = 0b0000_1011,
            "Kkq" => new_board.castling = 0b0000_1101,
            "Qkq" => new_board.castling = 0b0000_1110,
            "KQkq" => new_board.castling = 0b0000_1111,
            _ => panic!("invalid castling flag {}", flags[1]),
        }

        match flags[2] {
            "-" => new_board.en_passant = None,
            _ => new_board.en_passant = Some(square(flags[2])),
        }

        new_board.fifty_move = flags[3].to_string().parse::<usize>().unwrap();
        let _complete_moves = flags[4].to_string().parse::<usize>().unwrap();

        let mut file = 0;
        let mut rank = 7;

        for c in board_fen.bytes() {
            if c == b'/' {
                rank -= 1;
                assert!((file == 8), "invalid file count on / {file}");
                file = 0;
                continue;
            }
            assert!((file != 8), "file count 8 and no newline {c}");
            match c {
                b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' => {
                    file += (c - b'0') as usize;
                }
                b'P' | b'N' | b'B' | b'R' | b'Q' | b'K' | b'p' | b'n' | b'b' | b'r' | b'q' | b'k' => {
                    new_board.bitboards[ascii_to_piece(c as char)] = set_bit(
                        unsafe { Square::from((rank * 8 + file) as u8) },
                        new_board.bitboards[ascii_to_piece(c as char)],
                    );
                    new_board.pieces_array[rank * 8 + file] = Some(ascii_to_piece(c as char));
                    file += 1;
                }
                _ => panic!("unexpected character {c}"),
            }
        }

        new_board.occupancies[OccupancyIndex::WhiteOccupancies] = new_board.bitboards[Piece::WP]
            | new_board.bitboards[Piece::WN]
            | new_board.bitboards[Piece::WB]
            | new_board.bitboards[Piece::WR]
            | new_board.bitboards[Piece::WQ]
            | new_board.bitboards[Piece::WK];

        new_board.occupancies[OccupancyIndex::BlackOccupancies] = new_board.bitboards[Piece::BP]
            | new_board.bitboards[Piece::BN]
            | new_board.bitboards[Piece::BB]
            | new_board.bitboards[Piece::BR]
            | new_board.bitboards[Piece::BQ]
            | new_board.bitboards[Piece::BK];

        new_board.occupancies[OccupancyIndex::BothOccupancies] = new_board.occupancies
            [OccupancyIndex::WhiteOccupancies]
            | new_board.occupancies[OccupancyIndex::BlackOccupancies];

        new_board.hash_key = new_board.compute_hash();
        new_board.repetition_table[new_board.fifty_move] = new_board.hash_key;
        new_board.pawn_hash = new_board.compute_pawn_hash();
        new_board.compute_checkers_and_pins();

        new_board
    }

    pub fn print_board(&self) {
        let mut squares = String::new();
        for rank in 0..8 {
            for file in 0..8 {
                let sq = rank * 8 + file;
                squares += pretty_piece(self.pieces_array[sq]);
            }
        }

        for rank in (0..8).rev() {
            print!("{}", rank + 1);
            for file in 0..8 {
                let sq: usize = rank * 8 + file;
                print!(" {}", squares.chars().collect::<Vec<char>>()[sq]);
            }
            println!();
        }
        println!("  a b c d e f g h\n");
        match self.side_to_move {
            Colour::White => println!("White to move"),
            Colour::Black => println!("Black to move"),
        }
        let castling_rights: &str = match self.castling {
            0b0000_0000 => "NONE",
            0b0000_0001 => "K",
            0b0000_0010 => "Q",
            0b0000_0100 => "k",
            0b0000_1000 => "q",
            0b0000_0011 => "KQ",
            0b0000_0101 => "Kk",
            0b0000_1001 => "Kq",
            0b0000_0110 => "Qk",
            0b0000_1010 => "Qq",
            0b0000_1100 => "kq",
            0b0000_0111 => "KQk",
            0b0000_1011 => "KQq",
            0b0000_1101 => "Kkq",
            0b0000_1110 => "Qkq",
            0b0000_1111 => "KQkq",

            _ => panic!("invalid castling rights"),
        };
        println!("Castling: {castling_rights}");
        if let Some(ep) = self.en_passant {
            println!("En passant: {}", coordinate(ep));
        } else {
            println!("En passant: NONE");
        }

        println!("FEN: {}", self.fen());
        println!("Hash key: {:x}", self.hash_key);
    }

    #[must_use]
    pub fn is_kp_endgame(&self) -> bool {
        //used to avoid null move pruning in king and pawn endgames
        //where zugzwang is very common
        self.occupancies[OccupancyIndex::BothOccupancies]
            & !(self.bitboards[Piece::WP]
                | self.bitboards[Piece::WK]
                | self.bitboards[Piece::BP]
                | self.bitboards[Piece::BK])
            == 0
    }

    #[must_use]
    pub fn fen(&self) -> String {
        let mut fen = String::new();
        let mut empty_count = 0;

        for rank in (0..8).rev() {
            for file in 0..8 {
                let i = rank * 8 + file;
                let pc = self.pieces_array[i];

                if i % 8 == 0 && i != 56 {
                    if empty_count != 0 {
                        fen += format!("{empty_count}").as_str();
                        empty_count = 0;
                    }
                    fen += "/";
                }

                if let Some(p) = pc {
                    if empty_count != 0 {
                        fen += format!("{empty_count}").as_str();
                        empty_count = 0;
                    }
                    match p {
                        Piece::WP => fen += "P",
                        Piece::WN => fen += "N",
                        Piece::WB => fen += "B",
                        Piece::WR => fen += "R",
                        Piece::WQ => fen += "Q",
                        Piece::WK => fen += "K",
                        Piece::BP => fen += "p",
                        Piece::BN => fen += "n",
                        Piece::BB => fen += "b",
                        Piece::BR => fen += "r",
                        Piece::BQ => fen += "q",
                        Piece::BK => fen += "k",
                    }
                } else {
                    empty_count += 1;
                }
            }
        }

        if empty_count != 0 {
            fen += format!("{empty_count}").as_str();
        }

        fen += if self.side_to_move == Colour::White { " w" } else { " b" };

        fen += match self.castling {
            0b0000_0000 => " -",
            0b0000_0001 => " K",
            0b0000_0010 => " Q",
            0b0000_0100 => " k",
            0b0000_1000 => " q",
            0b0000_0011 => " KQ",
            0b0000_0101 => " Kk",
            0b0000_1001 => " Kq",
            0b0000_0110 => " Qk",
            0b0000_1010 => " Qq",
            0b0000_1100 => " kq",
            0b0000_0111 => " KQk",
            0b0000_1011 => " KQq",
            0b0000_1101 => " Kkq",
            0b0000_1110 => " Qkq",
            0b0000_1111 => " KQkq",

            _ => panic!("invalid castling rights"),
        };

        if let Some(ep) = self.en_passant {
            fen += " ";
            fen += &coordinate(ep);
        } else {
            fen += " -";
        }

        fen += format!(" {}", self.fifty_move).as_str();
        fen += " 1";

        fen
    }

    //used when we take in the board from a fen
    fn compute_checkers_and_pins(&mut self) {
        let colour = self.side_to_move;
        let our_king = unsafe {
            lsfb(
                self.bitboards[match colour {
                    Colour::White => Piece::WK,
                    Colour::Black => Piece::BK,
                }],
            )
            .unwrap_unchecked()
        };
        //SAFETY: there MUST be a king on the board

        let mut their_attackers = if colour == Colour::White {
            self.occupancies[OccupancyIndex::BlackOccupancies]
                & ((BISHOP_EDGE_RAYS[our_king] & (self.bitboards[Piece::BB] | self.bitboards[Piece::BQ]))
                    | ROOK_EDGE_RAYS[our_king] & (self.bitboards[Piece::BR] | self.bitboards[Piece::BQ]))
        } else {
            self.occupancies[OccupancyIndex::WhiteOccupancies]
                & ((BISHOP_EDGE_RAYS[our_king] & (self.bitboards[Piece::WB] | self.bitboards[Piece::WQ]))
                    | ROOK_EDGE_RAYS[our_king] & (self.bitboards[Piece::WR] | self.bitboards[Piece::WQ]))
        };

        while let Some(sq) = lsfb(their_attackers) {
            let ray_between = RAY_BETWEEN[sq][our_king] & self.occupancies[OccupancyIndex::BothOccupancies];
            match count(ray_between) {
                0 => self.checkers |= set_bit(sq, 0),
                1 => self.pinned |= ray_between,
                _ => {}
            }
            their_attackers = pop_bit(sq, their_attackers);
        }

        match colour {
            Colour::White => {
                self.checkers |= WP_ATTACKS[our_king] & self.bitboards[Piece::BP];
                self.checkers |= N_ATTACKS[our_king] & self.bitboards[Piece::BN];
            }
            Colour::Black => {
                self.checkers |= BP_ATTACKS[our_king] & self.bitboards[Piece::WP];
                self.checkers |= N_ATTACKS[our_king] & self.bitboards[Piece::WN];
            }
        }
    }

    #[must_use]
    pub fn get_piece_at(&self, sq: Square) -> Piece {
        //SAFETY: this must only be called when we know there is a piece on sq
        unsafe { self.pieces_array[sq].unwrap_unchecked() }
    }

    #[must_use]
    pub fn is_insufficient_material(&self) -> bool {
        if self.bitboards[Piece::WP] > 0
            || self.bitboards[Piece::BP] > 0
            || self.bitboards[Piece::WR] > 0
            || self.bitboards[Piece::BR] > 0
            || self.bitboards[Piece::WQ] > 0
            || self.bitboards[Piece::BQ] > 0
        {
            return false;
        }

        if self.bitboards[Piece::WB].count_ones() >= 2 || self.bitboards[Piece::BB].count_ones() >= 2 {
            return false;
        }

        if (self.bitboards[Piece::WN] > 0 && self.bitboards[Piece::WB] > 0)
            || (self.bitboards[Piece::BN] > 0 && self.bitboards[Piece::BB] > 0)
        {
            return false;
        }

        self.bitboards[Piece::WN].count_ones().max(self.bitboards[Piece::BN].count_ones()) <= 2
    }

    //make null move for NMP
    //we have to update pinners but not checkers since NMP is never done while in check
    pub fn make_null_move(&mut self) -> NullMoveUndo {
        let hash_reset = self.hash_key;
        self.side_to_move = self.side_to_move.opponent();
        self.last_move_null = true;
        self.fifty_move += 1;

        let hash_overwritten = self.repetition_table[self.fifty_move];

        let pinned_reset = self.pinned;

        let colour = self.side_to_move;

        //SAFETY: there MUST be a king on the board
        let our_king = unsafe {
            lsfb(
                self.bitboards[match colour {
                    Colour::White => Piece::WK,
                    Colour::Black => Piece::BK,
                }],
            )
            .unwrap_unchecked()
        };

        let mut their_attackers = if colour == Colour::White {
            self.occupancies[OccupancyIndex::BlackOccupancies]
                & ((BISHOP_EDGE_RAYS[our_king] & (self.bitboards[Piece::BB] | self.bitboards[Piece::BQ]))
                    | ROOK_EDGE_RAYS[our_king] & (self.bitboards[Piece::BR] | self.bitboards[Piece::BQ]))
        } else {
            self.occupancies[OccupancyIndex::WhiteOccupancies]
                & ((BISHOP_EDGE_RAYS[our_king] & (self.bitboards[Piece::WB] | self.bitboards[Piece::WQ]))
                    | ROOK_EDGE_RAYS[our_king] & (self.bitboards[Piece::WR] | self.bitboards[Piece::WQ]))
        };

        while let Some(sq) = lsfb(their_attackers) {
            let ray_between = RAY_BETWEEN[sq][our_king] & self.occupancies[OccupancyIndex::BothOccupancies];
            if count(ray_between) == 1 {
                self.pinned |= ray_between;
            }
            their_attackers = pop_bit(sq, their_attackers);
        }

        self.hash_key ^= BLACK_TO_MOVE;

        if let Some(reset) = self.en_passant {
            self.hash_key ^= EP_KEYS[reset];
            self.repetition_table[self.fifty_move] = self.hash_key;
            self.en_passant = None;
            return NullMoveUndo { ep: Some(reset), pinned: pinned_reset, hash_key: hash_reset, hash_overwritten };
        }

        self.repetition_table[self.fifty_move] = self.hash_key;

        NullMoveUndo { ep: None, pinned: pinned_reset, hash_key: hash_reset, hash_overwritten }
    }

    pub fn undo_null_move(&mut self, undo: &NullMoveUndo) {
        self.repetition_table[self.fifty_move] = undo.hash_overwritten;
        self.side_to_move = self.side_to_move.opponent();
        self.last_move_null = false;
        self.en_passant = undo.ep;
        self.fifty_move -= 1;

        self.pinned = undo.pinned;
        self.hash_key = undo.hash_key;

        self.repetition_table[self.fifty_move] = self.hash_key;
    }

    #[must_use]
    pub fn is_drawn(&self) -> bool {
        if self.fifty_move < 4 {
            return self.is_insufficient_material();
        }

        if self.fifty_move >= 100 {
            return true;
        }

        for &key in self.repetition_table.iter().take(self.fifty_move - 3).rev().step_by(2) {
            if key == self.hash_key {
                return true;
                //return true on two-fold repetition because otherwise the third
                //repetition will not be reached because the search will stop
                //after a tt hit on the second repetition
            }
        }

        self.is_insufficient_material()
    }
}
