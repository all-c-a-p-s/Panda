pub mod helper;
pub mod magic;
pub mod rng;

use crate::helper::*;
use crate::magic::*;
use crate::rng::*;

fn main() {
    let blockers: u64 = 0;
    let bit = mask_queen_attacks(square("d4"), blockers);
    let occupancy = set_occupancy(50000, count(bit), bit);
    print_bitboard(occupancy);
    println!();
    for _ in 0..5 {
        println!("{}", generator_random_number())
    }
}
