use std::time::Instant;

use crate::types::*;
use crate::zobrist::*;
use crate::*;

pub enum CommandType {
    Unknown,
    Uci,
    UciNewGame, //can basically ignore
    IsReady,
    Position,
    Perft,
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
        "go" => {
            if words.is_empty() {
                panic!("invalid uci command");
            }

            if words[1] == "perft" {
                CommandType::Perft
            } else {
                CommandType::Go
            }
        }
        "stop" => CommandType::Stop,
        "quit" => CommandType::Quit,
        "d" => CommandType::D,
        _ => CommandType::Unknown,
    }
}

pub const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn parse_move(input: &str, board: &Board) -> Move {
    let sq_from = unsafe { Square::from(square(&input[0..2]) as u8) };
    let sq_to = unsafe { Square::from(square(&input[2..4]) as u8) };
    let piece = board.get_piece_at(sq_from);
    if input.len() == 5 {
        //only type of piece encoded because only 2 bits used in the move
        //and the flag is used to detect promotions
        let promoted_piece = match input.chars().collect::<Vec<char>>()[4] {
            'q' | 'Q' => PieceType::Queen,
            'r' | 'R' => PieceType::Rook,
            'b' | 'B' => PieceType::Bishop,
            'n' | 'N' => PieceType::Knight,
            _ => panic!(
                "invalid promoted piece {}",
                input.chars().collect::<Vec<char>>()[4]
            ),
        };
        return encode_move(sq_from, sq_to, Some(promoted_piece), PROMOTION_FLAG);
    }
    if (sq_from == Square::E1 && piece == Piece::WK && (sq_to == Square::G1 || sq_to == Square::C1))
        || (sq_from == Square::E8
            && piece == Piece::BK
            && (sq_to == Square::G8 || sq_to == Square::C8))
    {
        return encode_move(sq_from, sq_to, None, CASTLING_FLAG);
    } else if board.en_passant.is_some()
        && board.en_passant.unwrap() == sq_to
        && piece_type(piece) == PieceType::Pawn
    {
        return encode_move(sq_from, sq_to, None, EN_PASSANT_FLAG);
    }
    encode_move(sq_from, sq_to, None, NO_FLAG)
}

pub fn parse_uci(command: &str) {
    if command == "uci" {
        println!("uciok");
        println!("id name Panda 1.0");
        println!("id author Sebastiano Rebonato-Scott");
    }
}

pub fn parse_isready(command: &str) {
    if command == "isready" {
        println!("readyok")
    }
}

pub fn reset(b: &mut Board) {
    *b = Board::from(STARTPOS);
}

pub fn parse_position(command: &str, b: &mut Board) {
    reset(b);
    b.hash_key = hash(b);
    //^ is necessary becuse reset doesn't update hash key
    unsafe { REPETITION_TABLE[b.ply] = b.hash_key }
    let words = command.split_whitespace().collect::<Vec<&str>>();
    if words.len() < 2 {
        panic!("invalid position command");
    }
    match words[1] {
        "startpos" => {
            if words.len() != 2 {
                for w in words.iter().skip(3) {
                    //parse moves
                    let m = parse_move(w, b);
                    let Ok(_) = b.try_move(m) else {
                        panic!("Illegal move: {}", m.uci());
                    };
                    unsafe { REPETITION_TABLE[b.ply] = b.hash_key }; //hash to avoid repetitions
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
                let m = parse_move(w, b);
                let Ok(_) = b.try_move(m) else {
                    panic!("invalid move {}", m.uci());
                };
                unsafe { REPETITION_TABLE[b.ply] = b.hash_key }; //hash to avoid repetitions
            }
        }
        _ => {}
    };
}

pub fn parse_special_go(command: &str, b: &mut Board, s: &mut Searcher) -> MoveData {
    //special combination of go and position command by lichess bot api
    reset(b);
    b.hash_key = hash(b);
    //^ is necessary becuse reset doesn't update hash key
    unsafe { REPETITION_TABLE[b.ply] = b.hash_key }
    let words = command.split_whitespace().collect::<Vec<&str>>();
    if words.len() < 2 {
        panic!("invalid position command");
    }

    let mut end_of_moves = 0;

    match words[1] {
        "startpos" => {
            if words.len() != 2 {
                #[allow(clippy::needless_range_loop)]
                for i in 3..words.len() {
                    if words[i].chars().collect::<Vec<char>>()[0] == 'w' {
                        end_of_moves = i;
                        break;
                    }
                    //parse moves
                    let m = parse_move(words[i], b);
                    let Ok(_) = b.try_move(m) else {
                        panic!("invalid move {}", m.uci());
                    };
                    unsafe { REPETITION_TABLE[b.ply] = b.hash_key }; //hash to avoid repetitions
                }
            }
        }
        "fen" => {
            let mut fen_string = String::new();
            #[allow(clippy::needless_range_loop)]
            for i in 2..words.len() {
                if words[i].chars().collect::<Vec<char>>()[0] == 'w' {
                    end_of_moves = i;
                    break;
                }
                fen_string += words[i];
                if i != words.len() - 1 {
                    fen_string += " ";
                }
            }
            *b = Board::from(&fen_string)
        }
        "moves" => {
            #[allow(clippy::needless_range_loop)]
            for i in 2..words.len() {
                if words[i].chars().collect::<Vec<char>>()[0] == 'w' {
                    end_of_moves = i;
                    break;
                }
                let m = parse_move(words[i], b);
                let Ok(_) = b.try_move(m) else {
                    panic!("invalid move {}", m.uci());
                };
                unsafe { REPETITION_TABLE[b.ply] = b.hash_key }; //hash to avoid repetitions
            }
        }
        _ => panic!("invalid position command"),
    };

    let time_words = &words[end_of_moves..];

    let mut fake_go_command = String::from("go ");
    for w in time_words {
        fake_go_command += w;
        fake_go_command += " ";
    }

    parse_go(fake_go_command.as_str(), b, s)
}

pub fn parse_go(command: &str, position: &mut Board, s: &mut Searcher) -> MoveData {
    let words = command.split_whitespace().collect::<Vec<&str>>();
    //go wtime x btime x winc x binc x movestogo x

    let mut movetime = 0;
    // if go command sets move time for engine

    let (mut w_inc, mut b_inc, mut moves_to_go) = (0, 0, 0);

    if words[1] == "moves" {
        //special command lichess-bot protocol uses
        return parse_special_go(command, position, s);
    } else if words[1] == "movetime" {
        movetime = words[2].parse().expect("failed to convert movetime to int");
        return iterative_deepening(position, 0, 0, 0, movetime, s, true);
    }

    let w_time = words[2].parse().expect("failed to convert wtime to int");
    let b_time = words[4].parse().expect("failed to convert btime to int");

    match words.len() {
        5 => {
            //go wtime x btime x
        }
        7 => {
            //go wtime x btime x movestogo x
            moves_to_go = words[6]
                .parse()
                .expect("failed to convert movestogo to int");
        }
        9 => {
            //go wtime x btime x winc x binc x
            w_inc = words[6].parse().expect("failed to convert winc to int");
            b_inc = words[8].parse().expect("failed to covnert binc to int");
        }
        11 => {
            //go wtime x btime x winc x binc x movestogo x
            w_inc = words[6].parse().expect("failed to convert winc to int");
            b_inc = words[8].parse().expect("failed to covnert binc to int");
            moves_to_go = words[10]
                .parse()
                .expect("failed to convert movestogo to int");
        }
        _ => return parse_special_go(command, position, s),
    };

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

    iterative_deepening(
        position,
        engine_time,
        engine_inc,
        moves_to_go,
        movetime,
        s,
        true,
    )
}

fn parse_perft(command: &str, position: &mut Board) {
    let words = command.split_whitespace().collect::<Vec<&str>>();
    if words.len() != 3 {
        eprintln!("invalid perft command: expected command of form go perft <depth>");
        return;
    }

    if let Ok(x) = words[2].parse::<usize>() {
        let start = Instant::now();
        let nodes = perft::<true>(x, position, Some(x));
        let time = start.elapsed().as_millis() as usize;

        let nps = if time == 0 {
            nodes * 1000
        } else {
            (nodes / time) * 1000
        };

        println!(
            "\ninfo depth {} nodes {} time {} nps {}",
            x, nodes, time, nps
        );
    } else {
        eprintln!("expected integer depth in perft command (go perft <depth>)")
    }
}

pub fn print_thinking(depth: u8, eval: i32, s: &Searcher, start: Instant) {
    println!(
        "info depth {} score cp {} nodes {} pv{} time {} nps {}",
        depth,
        eval,
        s.nodes,
        {
            let mut pv = String::new();
            for i in 0..s.pv_length[0] {
                pv += " ";
                pv += s.pv[0][i].uci().as_str();
            }
            pv
        },
        start.elapsed().as_millis(),
        {
            let micros = start.elapsed().as_micros() as f64;
            if micros == 0.0 {
                0
            } else {
                ((s.nodes as f64 / micros) * 1_000_000.0) as u64
            }
        }
    );
}

pub fn uci_loop() {
    let mut board = Board::from(STARTPOS);
    let mut s = Searcher::new(Instant::now(), usize::MAX);
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
                let move_data = parse_go(buffer.as_str(), &mut board, &mut s);
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
                                    PieceType::Knight => "n",
                                    PieceType::Bishop => "b",
                                    PieceType::Rook => "r",
                                    PieceType::Queen => "q",
                                    _ => unreachable!(),
                                }
                            } else {
                                ""
                            }
                        }
                });
            }
            CommandType::Perft => parse_perft(buffer.as_str(), &mut board),
            CommandType::UciNewGame => board = Board::from(STARTPOS),
            _ => {}
        }
    }
}
