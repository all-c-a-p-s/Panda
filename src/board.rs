use crate::helper::*;

#[derive(Debug)]
pub struct Board {
    pub bitboards: [u64; 12],
    pub castling: u8, //4 bits only should be used 0001 = wk, 0010 = wq, 0100 = bk, 1000 = bq
    pub en_passant: usize, //ep square index
    pub side_to_move: Colour,
    pub fifty_move: u8,
    pub ply: usize,
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
        _ => panic!("invalid character in ascii_to_piece_index()"),
    }
}

pub fn fen_to_board(fen: &str) -> Board {
    let mut new_board = Board {
        bitboards: [0u64; 12],
        castling: 0,
        en_passant: 64,
        side_to_move: Colour::White,
        fifty_move: 0,
        ply: 0,
    };

    let mut board_fen: String = String::new();
    let mut flags: usize = 0; //index where board ends and flags start
    for i in fen.chars() {
        flags += 1;
        if i == ' ' {
            break;
        }
        board_fen += i.to_string().as_str()
    }

    let flags: Vec<&str> = (fen[flags..].split(' ')).clone().collect::<Vec<&str>>();

    match flags[0] {
        "w" => new_board.side_to_move = Colour::White,
        "b" => new_board.side_to_move = Colour::Black,
        _ => panic!("invalid colour to move flag in fen string"),
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
        _ => panic!("invalid castling flag {}", flags[1]),
    }

    match flags[2] {
        "-" => new_board.en_passant = 64,
        _ => new_board.en_passant = square(flags[2]),
    }

    new_board.fifty_move = flags[3].to_string().parse::<u8>().unwrap();
    let complete_moves: usize = flags[4].to_string().parse::<usize>().unwrap();
    new_board.ply = (complete_moves - 1) * 2;
    if new_board.side_to_move == Colour::Black {
        new_board.ply += 1;
    }

    let mut file: usize = 0;
    let mut rank: usize = 7;

    for c in board_fen.chars() {
        if c == '/' {
            rank -= 1;
            if file != 8 {
                panic!("invalid file count on / {}", file)
            }
            file = 0;
            continue;
        }
        if file == 8 {
            panic!("file count 8 and no newline {}", c)
        }
        match c {
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                file += <u32 as std::convert::TryInto<usize>>::try_into(c.to_digit(10).unwrap())
                    .unwrap()
            }
            'P' | 'N' | 'B' | 'R' | 'Q' | 'K' | 'p' | 'n' | 'b' | 'r' | 'q' | 'k' => {
                new_board.bitboards[ascii_to_piece_index(c)] = set_bit(
                    rank * 8 + file,
                    new_board.bitboards[ascii_to_piece_index(c)],
                );
                file += 1
            }
            _ => panic!("unexpected character {}", c),
        }
    }

    new_board
}

pub fn print_board(b: Board) {
    let mut squares = String::new();
    for rank in 0..8 {
        for file in 0..8 {
            let sq = rank * 8 + file;
            let mut empty = true;
            for i in 0..b.bitboards.len() {
                if (b.bitboards[i] & set_bit(sq, 0)) != 0 {
                    match i {
                        0 => squares += "P",
                        1 => squares += "N",
                        2 => squares += "B",
                        3 => squares += "R",
                        4 => squares += "Q",
                        5 => squares += "K",

                        6 => squares += "p",
                        7 => squares += "n",
                        8 => squares += "b",
                        9 => squares += "r",
                        10 => squares += "q",
                        11 => squares += "k",

                        _ => panic!("this is impossible"),
                    };
                    empty = false;
                    break;
                }
            }
            if empty {
                squares += ".";
            }
        }
    }

    for rank in (0..8).rev() {
        print!("{}", rank + 1);
        for file in 0..8 {
            let sq: usize = rank * 8 + file;
            print!(" {}", squares.chars().collect::<Vec<char>>()[sq])
        }
        println!()
    }
    println!("  a b c d e f g h\n");
    match b.side_to_move {
        Colour::White => println!("white to move"),
        Colour::Black => println!("black to move"),
    };
    let castling_rights: &str = match b.castling {
        0b0000_0000 => "NONE",
        0b0000_0001 => "K",
        0b0000_0010 => "Q",
        0b0000_0100 => "k",
        0b0000_1000 => "q",
        0b0000_0011 => "KQ",
        0b0000_0101 => "Kk",
        0b0000_1001 => "Kq",
        0b0000_0110 => "Qk",
        0b0000_1010 => "Qq",
        0b0000_1100 => "kq",
        0b0000_0111 => "KQk",
        0b0000_1011 => "KQq",
        0b0000_1101 => "Kkq",
        0b0000_1110 => "Qkq",
        0b0000_1111 => "KQkq",

        _ => panic!("invalid castling rights"),
    };
    println!("Castling: {}", castling_rights);
    if b.en_passant != 64 {
        println!("En passant: {}", coordinate(b.en_passant));
    } else {
        println!("En passant: NONE");
    }
}
