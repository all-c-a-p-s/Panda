pub mod helper;
pub mod magic;

use crate::helper::*;
use crate::magic::*;

fn main() {
    let mut blockers: u64 = 0;
    println!("{}", coordinate(square("e3")));
    blockers = set_bit(square("b6"), blockers);
    blockers = set_bit(square("d6"), blockers);
    blockers = set_bit(square("e6"), blockers);
    blockers = set_bit(square("f6"), blockers);
    blockers = set_bit(square("g6"), blockers);
    println!("{}", count(blockers));
    let b = lsfb(blockers).unwrap();
    println!("{}", b);
    let bit = mask_queen_attacks(square("d4"), blockers);
    print_bitboard(bit);
}
