use crate::helper::*;

#[derive(Debug)]
pub struct Board {
    bitboards: [u64; 12],
    castling: u8, //4 bits only should be used 0001 = wk, 0010 = wq, 0100 = bk, 1000 = bq
    en_passant: usize, //ep square index
    side_to_move: Colour,
    fifty_move_count: u8,
    ply: usize,
}

#[derive(PartialEq, Debug)]
pub enum Colour {
    White,
    Black,
}

pub fn ascii_to_piece_index(ascii: char) -> usize {
    match ascii {
        'P' => 0,
        'N' => 1,
        'B' => 2,
        'R' => 3,
        'Q' => 4,
        'K' => 5,
        'p' => 6,
        'n' => 7,
        'b' => 8,
        'r' => 9,
        'q' => 10,
        'k' => 11,
        _ => panic!("invalid character in ascii_to_piece_index"),
    }
}

pub fn fen_to_board(fen: &str) -> Board {
    let mut new_board = Board {
        bitboards: [0u64; 12],
        castling: 0,
        en_passant: 64,
        side_to_move: Colour::White,
        fifty_move_count: 0,
        ply: 0,
    };



    let mut board_fen: String = String::new();
    let mut flags: usize = 0; //index where board ends and flags start
    for i in fen.chars() {
        flags += 1;
        if i == ' ' {
            break
        }
        board_fen += i.to_string().as_str()
    }

    let flags: Vec<&str> = (fen[flags..].split(' ')).clone().collect::<Vec<&str>>();

    match flags[0] {
        "w" => new_board.side_to_move = Colour::White,
        "b" => new_board.side_to_move = Colour::Black,
        _ => panic!("invalid colour to move flag in fen string")
    }

    match flags[1] {
        "-" => new_board.castling = 0b0000_0000,
        "K" => new_board.castling = 0b0000_0001,
        "Q" => new_board.castling = 0b0000_0010,
        "k" => new_board.castling = 0b0000_0100,
        "q" => new_board.castling = 0b0000_1000,
        "KQ" => new_board.castling = 0b0000_0011,
        "Kk" => new_board.castling = 0b0000_0101,
        "Kq" => new_board.castling = 0b0000_1001,
        "Qk" => new_board.castling = 0b0000_0110,
        "Qq" => new_board.castling = 0b0000_1100,
        "KQk" => new_board.castling = 0b0000_0111,
        "KQq" => new_board.castling = 0b0000_1011,
        "Kkq" => new_board.castling = 0b0000_1101,
        "Qkq" => new_board.castling = 0b0000_1110,
        "KQkq" => new_board.castling = 0b0000_1111,
        _ => panic!("invalid castling flag {}", flags[1])
    }

    match flags[2] {
        "-" => new_board.en_passant = 64,
        _ => new_board.en_passant = square(flags[2]),
    }

    new_board.fifty_move_count = flags[3].to_string().parse::<u8>().unwrap();
    let complete_moves: usize = flags[4].to_string().parse::<usize>().unwrap();
    new_board.ply = (complete_moves - 1) * 2;
    if new_board.side_to_move == Colour::Black {
        new_board.ply += 1;
    }

    let mut file_count: usize = 0;
    let mut rank_count: usize = 0;

    for c in board_fen.chars() {
        if c == '/' {
            rank_count -= 1;
            if file_count != 8 {
                panic!("invalid file count {}", file_count)
            }
            file_count = 0;
            continue;
        }
        if file_count == 8 {
            panic!("file count 7 and no newline {}", c)
        }
        match c {
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                file_count +=
                    <u32 as std::convert::TryInto<usize>>::try_into(c.to_digit(10).unwrap())
                        .unwrap()
            }
            'P' | 'N' | 'B' | 'R' | 'Q' | 'K' | 'p' | 'n' | 'b' | 'r' | 'q' | 'k' => {
                new_board.bitboards[ascii_to_piece_index(c)] =
                    set_bit(rank_count * 8 + file_count, new_board.bitboards[ascii_to_piece_index(c)]);
                file_count += 1
            }
            _ => panic!("unexpected character {}", c),
        }
    }

    new_board
}
