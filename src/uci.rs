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
            'q' => match board.side_to_move {
                Colour::White => 4,
                Colour::Black => 10,
            },
            'r' => match board.side_to_move {
                Colour::White => 3,
                Colour::Black => 9,
            },
            'b' => match board.side_to_move {
                Colour::White => 2,
                Colour::Black => 8,
            },
            'n' => match board.side_to_move {
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
                if move_data.m == NULL_MOVE {
                    break;
                }
                print!("bestmove ");
                println!("{}", {
                    coordinate(move_data.m.square_from())
                        + coordinate(move_data.m.square_to()).as_str()
                        + match move_data.m.promoted_piece() {
                            1 | 7 => "n",
                            2 | 8 => "b",
                            3 | 9 => "r",
                            4 | 10 => "q",
                            15 => "",
                            _ => "impossible",
                        }
                });
            }
            CommandType::UciNewGame => board = Board::from(STARTPOS),
            _ => {}
        }
    }
}
