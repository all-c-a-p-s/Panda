use crate::board::*;
use crate::helper::*;
use crate::magic::*;

pub fn is_attacked(square: usize, colour: Colour, board: Board) -> bool {
    //attacked BY colour
    if colour == Colour::White {
        //leapers
        if BP_ATTACKS[square] & board.bitboards[0] != 0
            || N_ATTACKS[square] & board.bitboards[1] != 0
            || K_ATTACKS[square] & board.bitboards[5] != 0
        {
            return true;
        }
        //sliders
        if get_bishop_attacks(square, board.occupancies[2]) & board.bitboards[2] != 0
            || get_rook_attacks(square, board.occupancies[2]) & board.bitboards[3] != 0
            || get_queen_attacks(square, board.occupancies[2]) & board.bitboards[4] != 0
        {
            return true;
        }
    } else {
        //leapers
        if WP_ATTACKS[square] & board.bitboards[6] != 0
            || N_ATTACKS[square] & board.bitboards[7] != 0
            || K_ATTACKS[square] & board.bitboards[11] != 0
        {
            return true;
        }
        //sliders
        if get_bishop_attacks(square, board.occupancies[2]) & board.bitboards[8] != 0
            || get_rook_attacks(square, board.occupancies[2]) & board.bitboards[9] != 0
            || get_queen_attacks(square, board.occupancies[2]) & board.bitboards[10] != 0
        {
            return true;
        }
    }
    false
}

pub fn gen_moves(board: Board) {
    let (mut min, mut max) = (0usize, 6usize);
    if board.side_to_move == Colour::Black {
        min = 6;
        max = 12;
    }
    for i in min..max {
        //pieces of colour to move
        let mut bitboard = board.bitboards[i];

        if i == 0 {
            //white pawn quiet moves
            let mut copy = bitboard;
            while copy > 0 {
                let lsb = lsfb(copy).unwrap();
                if (set_bit(lsb + 8, 0) & !board.occupancies[2]) > 0 {
                    //not blocked
                    println!("{}{}", coordinate(lsb), coordinate(lsb + 8));
                }
                if rank(lsb) == 1 && (set_bit(lsb + 16, 0) & !board.occupancies[2]) > 0 {
                    //not blocked
                    println!("{}{}", coordinate(lsb), coordinate(lsb + 16));
                }
                copy = pop_bit(lsb, copy);
            }
        } else if i == 6 {
            //black pawn quiet moves
            let mut copy = bitboard;
            while copy > 0 {
                let lsb = lsfb(copy).unwrap();
                if (set_bit(lsb - 8, 0) & !board.occupancies[2]) > 0 {
                    //not blocked
                    println!("{}{}", coordinate(lsb), coordinate(lsb - 8));
                }
                if rank(lsb) == 6 && (set_bit(lsb - 16, 0) & !board.occupancies[2]) > 0 {
                    //not blocked
                    println!("{}{}", coordinate(lsb), coordinate(lsb - 16));
                }
                copy = pop_bit(lsb, copy);
            }
        }

        while bitboard > 0 {
            let lsb = lsfb(bitboard).unwrap(); // never panics as loop will have already exited
            let mut attacks = match i {
                0 => WP_ATTACKS[lsb] & board.occupancies[1],
                6 => BP_ATTACKS[lsb] & board.occupancies[0],
                1 | 7 => N_ATTACKS[lsb],
                2 | 8 => get_bishop_attacks(lsb, board.occupancies[2]),
                3 | 9 => get_rook_attacks(lsb, board.occupancies[2]),
                4 | 10 => get_queen_attacks(lsb, board.occupancies[2]),
                5 | 11 => K_ATTACKS[lsb],
                _ => panic!("this is impossible"),
            };
            match board.side_to_move {
                Colour::White => attacks &= !board.occupancies[0],
                Colour::Black => attacks &= !board.occupancies[1],
            }
            while attacks > 0 {
                let lsb_attack = lsfb(attacks).unwrap();
                println!("{}{}", coordinate(lsb), coordinate(lsb_attack)); //later add move to some
                                                                           //list to return here
                attacks = pop_bit(lsb_attack, attacks);
            }
            bitboard = pop_bit(lsb, bitboard);
        }
    }
}
