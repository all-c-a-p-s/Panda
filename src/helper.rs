use std::collections::HashMap;

#[allow(dead_code)]
pub const MAX: u64 = u64::MAX;

pub const A_FILE: u64 = 0x0101010101010101;
pub const B_FILE: u64 = 0x0202020202020202;
pub const C_FILE: u64 = 0x0404040404040404;
pub const D_FILE: u64 = 0x0808080808080808;
pub const E_FILE: u64 = 0x1010101010101010;
pub const F_FILE: u64 = 0x2020202020202020;
pub const G_FILE: u64 = 0x4040404040404040;
pub const H_FILE: u64 = 0x8080808080808080;

pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_3: u64 = 0x0000000000FF0000;
pub const RANK_4: u64 = 0x00000000FF000000;
pub const RANK_5: u64 = 0x000000FF00000000;
pub const RANK_6: u64 = 0x0000FF0000000000;
pub const RANK_7: u64 = 0x00FF000000000000;
pub const RANK_8: u64 = 0xFF00000000000000;

pub const WP: usize = 0;
pub const WN: usize = 1;
pub const WB: usize = 2;
pub const WR: usize = 3;
pub const WQ: usize = 4;
pub const WK: usize = 5;

pub const BP: usize = 6;
pub const BN: usize = 7;
pub const BB: usize = 8;
pub const BR: usize = 9;
pub const BQ: usize = 10;
pub const BK: usize = 11;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const BOTH: usize = 2;

//piece types of either colour
pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;

pub const NO_SQUARE: usize = 64; //these exist because from my testing using an
pub const NO_PIECE: usize = 15; //Option<usize> slows down perft

pub const MAX_MOVES: usize = 218;

pub const A1: usize = 0;
pub const B1: usize = 1;
pub const C1: usize = 2;
pub const D1: usize = 3;
pub const E1: usize = 4;
pub const F1: usize = 5;
pub const G1: usize = 6;
pub const H1: usize = 7;
pub const A2: usize = 8;
pub const B2: usize = 9;
pub const C2: usize = 10;
pub const D2: usize = 11;
pub const E2: usize = 12;
pub const F2: usize = 13;
pub const G2: usize = 14;
pub const H2: usize = 15;
pub const A3: usize = 16;
pub const B3: usize = 17;
pub const C3: usize = 18;
pub const D3: usize = 19;
pub const E3: usize = 20;
pub const F3: usize = 21;
pub const G3: usize = 22;
pub const H3: usize = 23;
pub const A4: usize = 24;
pub const B4: usize = 25;
pub const C4: usize = 26;
pub const D4: usize = 27;
pub const E4: usize = 28;
pub const F4: usize = 29;
pub const G4: usize = 30;
pub const H4: usize = 31;
pub const A5: usize = 32;
pub const B5: usize = 33;
pub const C5: usize = 34;
pub const D5: usize = 35;
pub const E5: usize = 36;
pub const F5: usize = 37;
pub const G5: usize = 38;
pub const H5: usize = 39;
pub const A6: usize = 40;
pub const B6: usize = 41;
pub const C6: usize = 42;
pub const D6: usize = 43;
pub const E6: usize = 44;
pub const F6: usize = 45;
pub const G6: usize = 46;
pub const H6: usize = 47;
pub const A7: usize = 48;
pub const B7: usize = 49;
pub const C7: usize = 50;
pub const D7: usize = 51;
pub const E7: usize = 52;
pub const F7: usize = 53;
pub const G7: usize = 54;
pub const H7: usize = 55;
pub const A8: usize = 56;
pub const B8: usize = 57;
pub const C8: usize = 58;
pub const D8: usize = 59;
pub const E8: usize = 60;
pub const F8: usize = 61;
pub const G8: usize = 62;
pub const H8: usize = 63;

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
pub const fn set_bit(square: usize, bitboard: u64) -> u64 {
    bitboard | (1 << square)
}

#[inline(always)]
pub const fn get_bit(square: usize, bitboard: u64) -> usize {
    if bitboard & (1 << square) != 0 {
        1
    } else {
        0
    }
}

#[inline(always)]
pub const fn pop_bit(square: usize, bitboard: u64) -> u64 {
    if get_bit(square, bitboard) == 1 {
        return bitboard ^ set_bit(square, 0);
    }
    bitboard
}

#[inline(always)]
pub const fn count(bitboard: u64) -> usize {
    let mut prev: u64 = bitboard;
    let mut count: usize = 0;
    while prev > 0 {
        prev &= prev - 1; //toggle least significant bit
        count += 1;
    }
    count
}

#[inline(always)]
pub const fn lsfb(bitboard: u64) -> Option<usize> {
    if bitboard != 0 {
        Some(count(((bitboard as i64) & -(bitboard as i64)) as u64 - 1))
    } else {
        None
    }
}

pub fn square(sq: &str) -> usize {
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
    (rank - 1) * 8 + file
}

pub fn coordinate(sq: usize) -> String {
    let mut files: HashMap<usize, char> = HashMap::new();
    for (file, idx) in file_indices() {
        files.insert(idx, file); //invert hashmap
    }
    let rank: usize = sq / 8;
    let r = format!("{}", rank + 1); //+1 for zero-indexed
    let file = sq - rank * 8;
    let f = files[&file];
    format!("{}{}", f, r)
}

#[inline(always)]
pub const fn rank(sq: usize) -> usize {
    sq / 8
}

#[inline(always)]
pub const fn file(sq: usize) -> usize {
    sq % 8
}

#[inline(always)]
pub const fn piece_type(piece: usize) -> usize {
    piece % 6
}

pub fn print_bitboard(bitboard: u64) {
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
