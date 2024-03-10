use crate::board::*;
use crate::helper::*;
use crate::magic::*;

pub fn is_attacked(square: usize, colour: Colour, board: &Board) -> bool {
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

pub fn pawn_push_moves(board: &Board) {
    match board.side_to_move {
        Colour::White => {
            let mut bitboard = board.bitboards[0];
            while bitboard > 0 {
                let lsb = lsfb(bitboard).unwrap();
                if get_bit(lsb + 8, board.occupancies[2]) == 0 {
                    println!("{}{}", coordinate(lsb), coordinate(lsb + 8));
                    if rank(lsb) == 1 && get_bit(lsb + 16, board.occupancies[2]) == 0 {
                        println!("{}{}", coordinate(lsb), coordinate(lsb + 16));
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
        }
        Colour::Black => {
            let mut bitboard = board.bitboards[6];
            while bitboard > 0 {
                let lsb = lsfb(bitboard).unwrap();
                if get_bit(lsb - 8, board.occupancies[2]) == 0 {
                    println!("{}{}", coordinate(lsb), coordinate(lsb - 8));
                    if rank(lsb) == 6 && get_bit(lsb - 16, board.occupancies[2]) == 0 {
                        println!("{}{}", coordinate(lsb), coordinate(lsb - 16));
                    }
                    bitboard = pop_bit(lsb, bitboard);
                }
            }
        }
    }
}

pub fn castling_moves(board: &Board) {
    match board.side_to_move {
        Colour::White => {
            if (board.castling & 0b0000_0001) > 0 {
                //white kingside castling rights
                if get_bit(6, board.occupancies[2]) == 0
                    && get_bit(5, board.occupancies[2]) == 0
                    && !is_attacked(4, Colour::Black, board)
                    && !is_attacked(5, Colour::Black, board)
                {
                    println!("e1g1");
                }
            }

            if (board.castling & 0b0000_0010) > 0 {
                //white queenside
                if get_bit(2, board.occupancies[2]) == 0
                    && get_bit(3, board.occupancies[2]) == 0
                    && !is_attacked(4, Colour::Black, board)
                    && !is_attacked(3, Colour::Black, board)
                {
                    println!("e1c1");
                }
            }
        }
        Colour::Black => {
            if (board.castling & 0b0000_0100) > 0 {
                //black kingside
                if get_bit(62, board.occupancies[2]) == 0
                    && get_bit(61, board.occupancies[2]) == 0
                    && !is_attacked(60, Colour::White, board)
                    && !is_attacked(61, Colour::White, board)
                {
                    println!("e8g8");
                }
            }

            if (board.castling & 0b0000_1000) > 0 {
                //black queenside
                if get_bit(58, board.occupancies[2]) == 0
                    && get_bit(59, board.occupancies[2]) == 0
                    && !is_attacked(60, Colour::White, board)
                    && !is_attacked(59, Colour::White, board)
                {
                    println!("e8c8");
                }
            }
        }
    }
}

pub fn gen_moves(board: &Board) {
    let (mut min, mut max) = (0usize, 6usize);
    if board.side_to_move == Colour::Black {
        min = 6;
        max = 12;
    }

    pawn_push_moves(board);
    castling_moves(board);

    for i in min..max {
        //pieces of colour to move
        let mut bitboard = board.bitboards[i];

        while bitboard > 0 {
            let lsb = lsfb(bitboard).unwrap(); // never panics as loop will have already exited
            let mut attacks = match i {
                0 => {
                    WP_ATTACKS[lsb] & {
                        if board.en_passant != 64 {
                            board.occupancies[1] | set_bit(board.en_passant, 0)
                        } else {
                            board.occupancies[1]
                        }
                    }
                } //en passant capture
                6 => {
                    BP_ATTACKS[lsb] & {
                        if board.en_passant != 64 {
                            board.occupancies[1] | set_bit(board.en_passant, 0)
                        } else {
                            board.occupancies[1]
                        }
                    }
                } //or with set en passant square if it is not 64 i.e. none
                1 | 7 => N_ATTACKS[lsb],
                2 | 8 => get_bishop_attacks(lsb, board.occupancies[2]),
                3 | 9 => get_rook_attacks(lsb, board.occupancies[2]),
                4 | 10 => get_queen_attacks(lsb, board.occupancies[2]),
                5 | 11 => K_ATTACKS[lsb],
                _ => panic!("this is impossible"),
            };
            match board.side_to_move {
                Colour::White => attacks &= !board.occupancies[0], //remove attacks on own pieces
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
