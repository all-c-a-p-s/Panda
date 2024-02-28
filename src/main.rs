pub mod helper;
pub mod magic;

use crate::helper::*;
use crate::magic::*;

fn main() {
    let bit = mask_king_attacks(str_to_square_idx(String::from("h4")));
    print_bitboard(bit);
}
