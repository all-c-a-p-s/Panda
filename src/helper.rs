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

pub const NO_SQUARE: usize = 64; //these exist because from my testing using an
pub const NO_PIECE: usize = 15; //Option<usize> slows down perft

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

pub const fn set_bit(square: usize, bitboard: u64) -> u64 {
    bitboard | (1 << square)
}

pub const fn get_bit(square: usize, bitboard: u64) -> usize {
    if bitboard & (1 << square) != 0 {
        1
    } else {
        0
    }
}

pub const fn pop_bit(square: usize, bitboard: u64) -> u64 {
    if get_bit(square, bitboard) == 1 {
        return bitboard ^ set_bit(square, 0);
    }
    bitboard
}

pub const fn count(bitboard: u64) -> usize {
    let mut prev: u64 = bitboard;
    let mut count: usize = 0;
    while prev > 0 {
        prev &= prev - 1; //toggle least significant bit
        count += 1;
    }
    count
}

pub const fn lsfb(bitboard: u64) -> usize {
    if bitboard != 0 {
        return count(((bitboard as i64) & -(bitboard as i64)) as u64 - 1);
    }
    NO_SQUARE
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

pub const fn rank(sq: usize) -> usize {
    sq / 8
}

pub const fn file(sq: usize) -> usize {
    sq % 8
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
