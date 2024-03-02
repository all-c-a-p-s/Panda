use std::collections::HashMap;

#[allow(dead_code)]
pub const MAX: u64 = u64::MAX;

pub const A_FILE: u64 = 0x8080808080808080;
pub const B_FILE: u64 = 0x4040404040404040;
pub const C_FILE: u64 = 0x2020202020202020;
pub const D_FILE: u64 = 0x1010101010101010;
pub const E_FILE: u64 = 0x0808080808080808;
pub const F_FILE: u64 = 0x0404040404040404;
pub const G_FILE: u64 = 0x0202020202020202;
pub const H_FILE: u64 = 0x0101010101010101;

pub const RANK_1: u64 = 0xFF00000000000000;
pub const RANK_2: u64 = 0x00FF000000000000;
pub const RANK_3: u64 = 0x0000FF0000000000;
pub const RANK_4: u64 = 0x000000FF00000000;
pub const RANK_5: u64 = 0x00000000FF000000;
pub const RANK_6: u64 = 0x0000000000FF0000;
pub const RANK_7: u64 = 0x000000000000FF00;
pub const RANK_8: u64 = 0x00000000000000FF;

/*
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}
*/

#[derive(PartialEq)]
pub enum SideToMove {
    White,
    Black,
}

pub fn files_indexes() -> HashMap<char, usize> {
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
        return bitboard ^ set_bit(square, bitboard);
    }
    bitboard
}

pub fn str_to_square_idx(square: String) -> usize {
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
    let file: usize = files_indexes()[&first];
    (rank - 1) * 8 + file
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
