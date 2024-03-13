pub mod board;
pub mod eval;
pub mod helper;
pub mod magic;
pub mod r#move;
pub mod movegen;
pub mod perft;
pub mod rng;
pub mod search;

use crate::board::*;
use crate::helper::*;
use crate::magic::*;
use crate::movegen::*;
use crate::r#move::*;
use crate::search::*;

fn init_all() {
    // initialise all constants
    init_slider_attacks();
}

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn parse_move(input: &str, board: Board) -> Move {
    let sq_from = square(&input[0..2]);
    let sq_to = square(&input[2..4]);
    if input.len() == 5 {
        let promoted_piece = match input.chars().collect::<Vec<char>>()[4] {
            'Q' => match board.side_to_move {
                Colour::White => 4,
                Colour::Black => 10,
            },
            'R' => match board.side_to_move {
                Colour::White => 3,
                Colour::Black => 9,
            },
            'B' => match board.side_to_move {
                Colour::White => 2,
                Colour::Black => 8,
            },
            'N' => match board.side_to_move {
                Colour::White => 1,
                Colour::Black => 7,
            },
            _ => panic!(
                "invalid promoted piece {}",
                input.chars().collect::<Vec<char>>()[4]
            ),
        };
        return encode_move(sq_from, sq_to, promoted_piece, &board, false);
    }
    let mut castling = false;
    if (sq_from == square("e1")
        && get_bit(square("e1"), board.bitboards[5]) == 1
        && (sq_to == square("g1") || sq_to == square("c1")))
        || (sq_from == square("e8")
            && get_bit(square("e8"), board.bitboards[11]) == 1
            && (sq_to == square("g8") || sq_to == square("c8")))
    {
        castling = true;
    }
    encode_move(sq_from, sq_to, 15, &board, castling)
}

fn main() {
    init_all();
    let mut pos = fen_to_board(STARTPOS);

    let mut colour_input = String::new();
    std::io::stdin().read_line(&mut colour_input).unwrap();
    colour_input.retain(|c| !c.is_whitespace());

    let user_colour = match colour_input.as_str() {
        "w" => Colour::White,
        "b" => Colour::Black,
        _ => panic!("invalid colour input {}", colour_input.as_str()),
    };
    match user_colour {
        Colour::White => {
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                input.retain(|c| !c.is_whitespace());
                let user_move = parse_move(input.as_str(), pos);
                pos = make_move(user_move, pos);
                let best = best_move(pos);
                if best == NULL_MOVE {
                    break;
                }
                pos = make_move(best, pos);
                println!("{}{}", coordinate(best.square_from()), coordinate(best.square_to()));
                print_board(&pos);
            }
        }
        Colour::Black => {
            loop {
                let best = best_move(pos);
                if best == NULL_MOVE {
                    break;
                }
                pos = make_move(best, pos);                 
                println!("{}{}", coordinate(best.square_from()), coordinate(best.square_to()));
                print_board(&pos);
                let mut input = String::new();
                input.retain(|c| !c.is_whitespace());
                std::io::stdin().read_line(&mut input).unwrap();
                let user_move = parse_move(input.as_str(), pos);
                pos = make_move(user_move, pos);                             
            }
        }
    }
}
