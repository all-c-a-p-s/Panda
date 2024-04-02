use crate::zobrist::hash;
use crate::*;

pub enum CommandType {
    Uci,
    UciNewGame, //can basically ignore
    IsReady,
    Position,
    Go,
    Stop,
    Quit,
    D, //not an actual UCI command but can be used to debug and display the board
}

pub fn recognise_command(command: &str) -> CommandType {
    let words = command.split_whitespace().collect::<Vec<&str>>();
    match words[0] {
        "uci" => CommandType::Uci,
        "ucinewgame" => CommandType::UciNewGame,
        "isready" => CommandType::IsReady,
        "position" => CommandType::Position,
        "go" => CommandType::Go,
        "stop" => CommandType::Stop,
        "quit" => CommandType::Quit,
        "d" => CommandType::D,
        _ => panic!("invalid command {}", command),
    }
}

pub const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn parse_move(input: &str, board: Board) -> Move {
    let sq_from = square(&input[0..2]);
    let sq_to = square(&input[2..4]);
    if input.len() == 5 {
        let promoted_piece = match input.chars().collect::<Vec<char>>()[4] {
            'q' | 'Q' => match board.side_to_move {
                Colour::White => WQ,
                Colour::Black => BQ,
            },
            'r' | 'R' => match board.side_to_move {
                Colour::White => WR,
                Colour::Black => BR,
            },
            'b' | 'B' => match board.side_to_move {
                Colour::White => WB,
                Colour::Black => BB,
            },
            'n' | 'N' => match board.side_to_move {
                Colour::White => WN,
                Colour::Black => BN,
            },
            _ => panic!(
                "invalid promoted piece {}",
                input.chars().collect::<Vec<char>>()[4]
            ),
        };
        return encode_move(sq_from, sq_to, piece_type(promoted_piece), PROMOTION_FLAG);
    }
    if (sq_from == E1 && get_bit(E1, board.bitboards[WK]) == 1 && (sq_to == G1 || sq_to == C1))
        || (sq_from == E8 && get_bit(E8, board.bitboards[BK]) == 1 && (sq_to == G8 || sq_to == C8))
    {
        return encode_move(sq_from, sq_to, NO_PIECE, CASTLING_FLAG);
    }
    encode_move(sq_from, sq_to, NO_PIECE, NO_FLAG)
}

pub fn parse_uci(command: &str) {
    match command {
        "uci" => {
            println!("uciok");
            println!("id name Panda 1.0");
            println!("id author Sebastiano Rebonato-Scott");
        }
        _ => panic!("invalid uci command"),
    }
}

pub fn parse_isready(command: &str) {
    match command {
        "isready" => println!("readyok"),
        _ => panic!("invalid isready command"),
    }
}

pub fn reset(b: &mut Board) {
    *b = Board::from(STARTPOS);
}

pub fn parse_position(command: &str, b: &mut Board) {
    reset(b);
    let words = command.split_whitespace().collect::<Vec<&str>>();
    if words.len() < 2 {
        panic!("invalid position command");
    }
    match words[1] {
        "startpos" => {
            if words.len() != 2 {
                for w in words.iter().skip(3) {
                    //parse moves
                    let m = parse_move(w, *b);
                    b.make_move(m);
                    let hash = hash(b);
                    unsafe { REPETITION_TABLE[b.ply] = hash }; //hash to avoid repetitions
                }
            }
        }
        "fen" => {
            let mut fen_string = String::new();
            for i in 2..words.len() {
                fen_string += words[i];
                if i != words.len() - 1 {
                    fen_string += " ";
                }
            }
            *b = Board::from(&fen_string)
        }
        "moves" => {
            for w in words.iter().skip(2) {
                let m = parse_move(w, *b);
                b.make_move(m);
                let hash = hash(b);
                unsafe { REPETITION_TABLE[b.ply] = hash }; //hash to avoid repetitions
            }
        }
        _ => panic!("invalid position command"),
    };
}

pub fn parse_go(command: &str, position: &mut Board) -> MoveData {
    let words = command.split_whitespace().collect::<Vec<&str>>();
    //go wtime x btime x winc x binc x movestogo x
    let w_time: usize = words[2].parse().expect("failed to convert wtime to int");
    let b_time: usize = words[4].parse().expect("failed to convert btime to int");
    let w_inc: usize = words[6].parse().expect("failed to convert winc to int");
    let b_inc: usize = words[8].parse().expect("failed to convert binc to int");

    let mut moves_to_go: usize = 20;

    if words.len() > 9 {
        moves_to_go = words[10]
            .parse()
            .expect("failed to convert movestogo to int");
    }

    let engine_time = match position.side_to_move {
        Colour::White => w_time,
        Colour::Black => b_time,
    };

    let engine_inc = match position.side_to_move {
        Colour::White => w_inc,
        Colour::Black => b_inc,
    };

    best_move(position, engine_time, engine_inc, moves_to_go)
}

pub fn uci_loop() {
    let mut board = Board::from(STARTPOS);
    loop {
        let mut buffer = String::new();
        let ok = std::io::stdin().read_line(&mut buffer);
        match ok {
            Ok(_) => (),
            Err(_) => panic!("failed to parse line"),
        }
        buffer = String::from(buffer.trim_end());
        if buffer == *"quit" {
            break;
        }

        let command_type = recognise_command(buffer.as_str());
        match command_type {
            CommandType::D => board.print_board(),
            CommandType::Uci => parse_uci(buffer.as_str()),
            CommandType::IsReady => parse_isready(buffer.as_str()),
            CommandType::Position => parse_position(buffer.as_str(), &mut board),
            CommandType::Go => {
                let move_data = parse_go(buffer.as_str(), &mut board);
                if move_data.m.is_null() {
                    break;
                }
                print!("bestmove ");
                println!("{}", {
                    coordinate(move_data.m.square_from())
                        + coordinate(move_data.m.square_to()).as_str()
                        + {
                            if move_data.m.is_promotion() {
                                match move_data.m.promoted_piece() {
                                    WN | BN => "n",
                                    WB | BB => "b",
                                    WR | BR => "r",
                                    WQ | BQ => "q",
                                    NO_PIECE => "",
                                    _ => "impossible",
                                }
                            } else {
                                ""
                            }
                        }
                });
            }
            CommandType::UciNewGame => board = Board::from(STARTPOS),
            _ => {}
        }
    }
}
