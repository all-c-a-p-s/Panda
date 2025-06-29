use crate::types::*;
use std::collections::HashMap;

use crate::BitBoard;

pub const A_FILE: BitBoard = 0x0101010101010101;
pub const B_FILE: BitBoard = 0x0202020202020202;
pub const C_FILE: BitBoard = 0x0404040404040404;
pub const D_FILE: BitBoard = 0x0808080808080808;
pub const E_FILE: BitBoard = 0x1010101010101010;
pub const F_FILE: BitBoard = 0x2020202020202020;
pub const G_FILE: BitBoard = 0x4040404040404040;
pub const H_FILE: BitBoard = 0x8080808080808080;

pub const RANK_1: BitBoard = 0x00000000000000FF;
pub const RANK_2: BitBoard = 0x000000000000FF00;
pub const RANK_3: BitBoard = 0x0000000000FF0000;
pub const RANK_4: BitBoard = 0x00000000FF000000;
pub const RANK_5: BitBoard = 0x000000FF00000000;
pub const RANK_6: BitBoard = 0x0000FF0000000000;
pub const RANK_7: BitBoard = 0x00FF000000000000;
pub const RANK_8: BitBoard = 0xFF00000000000000;

//these are the indices used for the board's occupancy arrays
pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const BOTH: usize = 2;

//max number of legal moves possible in a position (that has been found)
pub const MAX_MOVES: usize = 218;

pub fn file_indices() -> HashMap<char, usize> {
    let mut files = HashMap::new();
    files.insert('a', 0);
    files.insert('b', 1);
    files.insert('c', 2);
    files.insert('d', 3);
    files.insert('e', 4);
    files.insert('f', 5);
    files.insert('g', 6);
    files.insert('h', 7);
    files
}

#[inline(always)]
pub const fn set_bit(square: Square, bitboard: BitBoard) -> BitBoard {
    bitboard | (1 << square as u8)
}

#[inline(always)]
pub const fn get_bit(square: Square, bitboard: BitBoard) -> usize {
    if bitboard & (1 << square as u8) != 0 {
        1
    } else {
        0
    }
}

#[inline(always)]
pub const fn pop_bit(square: Square, bitboard: BitBoard) -> BitBoard {
    if get_bit(square, bitboard) == 1 {
        return bitboard ^ set_bit(square, 0);
    }
    bitboard
}

#[inline(always)]
pub const fn count(bitboard: BitBoard) -> usize {
    let mut prev: BitBoard = bitboard;
    let mut count: usize = 0;
    while prev > 0 {
        prev &= prev - 1; //toggle least significant bit
        count += 1;
    }
    count
}

#[inline(always)]
pub const fn lsfb(bitboard: BitBoard) -> Option<Square> {
    if bitboard != 0 {
        Some(unsafe { Square::from(bitboard.trailing_zeros() as u8) })
    } else {
        None
    }
}

pub fn square(sq: &str) -> Square {
    let square = sq.to_string();
    if square.len() != 2 {
        panic!("invalid square name")
    }
    let last: char = match square.chars().last() {
        Some(c) => c,
        None => panic!("failed to get last character"),
    };
    let rank = match char::to_digit(last, 10) {
        Some(r) => r,
        None => panic!("failed to convert rank to int"),
    } as usize;
    let first: char = square.chars().collect::<Vec<char>>()[0];
    let file: usize = file_indices()[&first];
    unsafe { Square::from(((rank - 1) * 8 + file) as u8) }
}

pub fn coordinate(sq: Square) -> String {
    let mut files: HashMap<usize, char> = HashMap::new();
    for (file, idx) in file_indices() {
        files.insert(idx, file); //invert hashmap
    }
    let rank: u8 = sq as u8 / 8;
    let r = format!("{}", rank + 1); //+1 for zero-indexed
    let file = unsafe { sq.sub_unchecked(rank * 8) } as usize;
    let f = files[&file];
    format!("{}{}", f, r)
}

#[inline(always)]
pub const fn rank(sq: Square) -> usize {
    sq as usize / 8
}

#[inline(always)]
pub const fn file(sq: Square) -> usize {
    sq as usize % 8
}

#[inline(always)]
pub const fn piece_type(piece: Piece) -> PieceType {
    unsafe { PieceType::from(piece as u8 % 6) }
}

pub fn print_bitboard(bitboard: BitBoard) {
    let mut board_ranks: Vec<String> = Vec::new();
    for rank in 0..8 {
        let mut rank_str = String::new();
        for file in 0..8 {
            let square = rank * 8 + file;
            let mut d: String = String::from("0 ");
            if (bitboard & (1 << square)) != 0 {
                d = String::from("1 ")
            }
            rank_str = format!("{}{}", rank_str, d);
        }
        board_ranks.push(rank_str);
    }

    for i in (0..board_ranks.len()).rev() {
        print!("{} ", i + 1);
        print!("{}", board_ranks[i]);
        println!()
    }
    println!("  a b c d e f g h");
}

// example macro usage here: https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=6931d3e71060f2320e9944f799c51755

#[macro_export]
macro_rules! tuneable_params {
    ($($name:ident, $t: ty, $val:expr, $min:expr, $max:expr;)*) => {
        pub fn list_params() {
            $(
                println!("option name {} type spin default {} min {} max {}",
                    stringify!($name),
                    $val,
                    $min,
                    $max,
                );
            )*
        }

        pub mod params {
            $(
                pub static mut $name: $t = $val;
            )*
        }
    };
}
pub(crate) use tuneable_params;

#[macro_export]
macro_rules! read_param {
    ($name:ident) => {
        unsafe { params::$name }
    };
}
pub(crate) use read_param;

#[macro_export]
macro_rules! set_param {
    ($name:ident, $val:expr) => {
        unsafe { params::$name = $val }
    };
}
