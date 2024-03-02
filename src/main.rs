pub mod helper;
pub mod magic;

use crate::helper::*;
use crate::magic::*;

fn main() {
    //    let bit = set_bit(str_to_square_idx(String::from("e4")), 0);
    let bit = mask_rook_attacks(str_to_square_idx(String::from("d4")));
    print_bitboard(bit);
}
