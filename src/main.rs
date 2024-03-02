pub mod helper;
pub mod magic;

use crate::helper::*;
use crate::magic::*;

fn main() {
    let bit = mask_queen_attacks(str_to_square_idx(String::from("d4")));
    print_bitboard(bit);
}
