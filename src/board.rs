use crate::helper::*;
use crate::magic::*;
use crate::movegen::*;
use crate::nnue::*;
use crate::types::{Piece, Square};
use crate::zobrist::hash;

pub(crate) type BitBoard = u64;
pub(crate) const EMPTY: BitBoard = 0;

#[derive(Debug, Clone, Copy)]
pub struct Board {
    //Fundamental board state
    pub bitboards: [BitBoard; 12],
    pub pieces_array: [Option<Piece>; 64], //used to speed up move generation
    pub occupancies: [BitBoard; 3],        //white, black, both
    pub castling: u8, //4 bits only should be used 0001 = wk, 0010 = wq, 0100 = bk, 1000 = bq
    pub en_passant: Option<Square>,
    pub side_to_move: Colour,
    pub fifty_move: u8,

    //Used in search
    pub ply: usize,
    pub last_move_null: bool,
    pub hash_key: u64,

    //Used in movegen
    pub checkers: BitBoard,
    pub pinned: BitBoard,

    //Used in evaluation
    pub nnue: Accumulator,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Colour {
    White,
    Black,
}

impl Colour {
    pub fn opponent(&self) -> Self {
        match self {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

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
    pub fn from(fen: &str) -> Self {
        let mut new_board = Board {
            bitboards: [EMPTY; 12],
            pieces_array: [None; 64],
            occupancies: [EMPTY; 3],
            castling: 0,
            en_passant: None,
            side_to_move: Colour::White,
            fifty_move: 0,
            ply: 0,
            last_move_null: false,
            hash_key: 0,
            checkers: 0,
            pinned: 0,
            nnue: Accumulator::default(),
        };

        let mut board_fen: String = String::new();
        let mut flags: usize = 0; //index where board ends and flags start
        for i in fen.chars() {
            flags += 1;
            if i == ' ' {
                break;
            }
            board_fen += i.to_string().as_str()
        }

        let flags: Vec<&str> = (fen[flags..].split(' ')).clone().collect::<Vec<&str>>();

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

        new_board.fifty_move = flags[3].to_string().parse::<u8>().unwrap();
        let complete_moves: usize = flags[4].to_string().parse::<usize>().unwrap();
        new_board.ply = (complete_moves - 1) * 2;
        if new_board.side_to_move == Colour::Black {
            new_board.ply += 1;
        }

        let mut file: usize = 0;
        let mut rank: usize = 7;

        for c in board_fen.chars() {
            if c == '/' {
                rank -= 1;
                if file != 8 {
                    panic!("invalid file count on / {}", file)
                }
                file = 0;
                continue;
            }
            if file == 8 {
                panic!("file count 8 and no newline {}", c)
            }
            match c {
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    file += <u32 as std::convert::TryInto<usize>>::try_into(c.to_digit(10).unwrap())
                        .unwrap()
                }
                'P' | 'N' | 'B' | 'R' | 'Q' | 'K' | 'p' | 'n' | 'b' | 'r' | 'q' | 'k' => {
                    new_board.bitboards[ascii_to_piece(c)] = set_bit(
                        unsafe { Square::from((rank * 8 + file) as u8) },
                        new_board.bitboards[ascii_to_piece(c)],
                    );
                    new_board.pieces_array[rank * 8 + file] = Some(ascii_to_piece(c));
                    file += 1
                }
                _ => panic!("unexpected character {}", c),
            }
        }

        new_board.occupancies[WHITE] = new_board.bitboards[Piece::WP]
            | new_board.bitboards[Piece::WN]
            | new_board.bitboards[Piece::WB]
            | new_board.bitboards[Piece::WR]
            | new_board.bitboards[Piece::WQ]
            | new_board.bitboards[Piece::WK];

        new_board.occupancies[BLACK] = new_board.bitboards[Piece::BP]
            | new_board.bitboards[Piece::BN]
            | new_board.bitboards[Piece::BB]
            | new_board.bitboards[Piece::BR]
            | new_board.bitboards[Piece::BQ]
            | new_board.bitboards[Piece::BK];

        new_board.occupancies[BOTH] = new_board.occupancies[WHITE] | new_board.occupancies[BLACK];

        new_board.hash_key = hash(&new_board);
        new_board.compute_checkers_and_pins();
        new_board.nnue = Accumulator::from_board(&new_board);

        new_board
    }

    pub fn print_board(&self) {
        let mut squares = String::new();
        for rank in 0..8 {
            for file in 0..8 {
                let sq = rank * 8 + file;
                let sq = unsafe { Square::from(sq as u8) };
                let mut empty = true;
                for i in 0..self.bitboards.len() {
                    if (self.bitboards[i] & set_bit(sq, 0)) != 0 {
                        match i {
                            0 => squares += "P",
                            1 => squares += "N",
                            2 => squares += "B",
                            3 => squares += "R",
                            4 => squares += "Q",
                            5 => squares += "K",

                            6 => squares += "p",
                            7 => squares += "n",
                            8 => squares += "b",
                            9 => squares += "r",
                            10 => squares += "q",
                            11 => squares += "k",

                            _ => panic!("this is impossible"),
                        };
                        empty = false;
                        break;
                    }
                }
                if empty {
                    squares += ".";
                }
            }
        }

        for rank in (0..8).rev() {
            print!("{}", rank + 1);
            for file in 0..8 {
                let sq: usize = rank * 8 + file;
                print!(" {}", squares.chars().collect::<Vec<char>>()[sq])
            }
            println!()
        }
        println!("  a b c d e f g h\n");
        match self.side_to_move {
            Colour::White => println!("White to move"),
            Colour::Black => println!("Black to move"),
        };
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
        println!("Castling: {}", castling_rights);
        if let Some(ep) = self.en_passant {
            println!("En passant: {}", coordinate(ep));
        } else {
            println!("En passant: NONE");
        }

        println!("FEN: {}", self.fen());
    }

    pub fn is_kp_endgame(&self) -> bool {
        //used to avoid null move pruning in king and pawn endgames
        //where zugzwang is very common
        self.occupancies[BOTH]
            ^ (self.bitboards[Piece::WP]
                | self.bitboards[Piece::WK]
                | self.bitboards[Piece::BP]
                | self.bitboards[Piece::BK])
            == 0
    }

    pub fn fen(&self) -> String {
        let mut fen = String::new();
        let mut empty_count = 0;

        for rank in (0..8).rev() {
            for file in 0..8 {
                let i = rank * 8 + file;
                let pc = self.pieces_array[i];

                if i % 8 == 0 && i != 56 {
                    if empty_count != 0 {
                        fen += format!("{}", empty_count).as_str();
                        empty_count = 0;
                    }
                    fen += "/";
                }
                if pc.is_none() {
                    empty_count += 1;
                } else {
                    if empty_count != 0 {
                        fen += format!("{}", empty_count).as_str();
                        empty_count = 0;
                    }
                    match pc {
                        Some(Piece::WP) => fen += "P",
                        Some(Piece::WN) => fen += "N",
                        Some(Piece::WB) => fen += "B",
                        Some(Piece::WR) => fen += "R",
                        Some(Piece::WQ) => fen += "Q",
                        Some(Piece::WK) => fen += "K",
                        Some(Piece::BP) => fen += "p",
                        Some(Piece::BN) => fen += "n",
                        Some(Piece::BB) => fen += "b",
                        Some(Piece::BR) => fen += "r",
                        Some(Piece::BQ) => fen += "q",
                        Some(Piece::BK) => fen += "k",
                        _ => unreachable!(),
                    }
                }
            }
        }

        if empty_count != 0 {
            fen += format!("{}", empty_count).as_str();
        }

        fen += if self.side_to_move == Colour::White {
            " w"
        } else {
            " b"
        };

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
        };

        fen += format!(" {}", self.fifty_move).as_str();
        fen += format!(" {}", self.ply % 2 + 1).as_str();

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
            self.occupancies[BLACK]
                & ((BISHOP_EDGE_RAYS[our_king]
                    & (self.bitboards[Piece::BB] | self.bitboards[Piece::BQ]))
                    | ROOK_EDGE_RAYS[our_king]
                        & (self.bitboards[Piece::BR] | self.bitboards[Piece::BQ]))
        } else {
            self.occupancies[WHITE]
                & ((BISHOP_EDGE_RAYS[our_king]
                    & (self.bitboards[Piece::WB] | self.bitboards[Piece::WQ]))
                    | ROOK_EDGE_RAYS[our_king]
                        & (self.bitboards[Piece::WR] | self.bitboards[Piece::WQ]))
        };

        while let Some(sq) = lsfb(their_attackers) {
            let ray_between = RAY_BETWEEN[sq][our_king] & self.occupancies[BOTH];
            match count(ray_between) {
                0 => self.checkers |= set_bit(sq, 0),
                1 => self.pinned |= ray_between,
                _ => {}
            }
            their_attackers = pop_bit(sq, their_attackers);
        }
    }

    pub fn get_piece_at(&self, sq: Square) -> Piece {
        //SAFETY: this must only be called when we know there is a piece on sq
        unsafe { self.pieces_array[sq].unwrap_unchecked() }
    }
}
