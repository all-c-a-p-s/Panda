use std::time::Instant;

#[cfg(feature = "tuning")]
use crate::set_param;

#[cfg(feature = "tuning")]
use crate::search::{list_params, params};

use crate::thread::{Searcher, Thread};
use crate::transposition::TranspositionTable;
use crate::types::{Piece, PieceType, Square};
use crate::{
    coordinate, encode_move, perft, piece_type, square, Board, Colour, Move, MoveData,
    CASTLING_FLAG, EN_PASSANT_FLAG, INFINITY, NO_FLAG, PROMOTION_FLAG,
};

pub enum CommandType {
    Unknown,
    Uci,
    UciNewGame, //can basically ignore
    IsReady,
    Position,
    Perft,
    Go,
    SetOption,
    Stop,
    Quit,
    D, //not an actual UCI command but can be used to debug and display the board
}

const DEFAULT_HASH_SIZE: usize = 16;
const DEFAULT_THREAD_COUNT: usize = 1;

pub struct UciOptions {
    pub hash_size: usize,
    pub threads: usize,
}

impl Default for UciOptions {
    fn default() -> Self {
        Self {
            hash_size: DEFAULT_HASH_SIZE,
            threads: DEFAULT_THREAD_COUNT,
        }
    }
}

impl Move {
    #[must_use]
    pub fn uci(self) -> String {
        let mut res = String::new();
        res += coordinate(self.square_from()).as_str();
        res += coordinate(self.square_to()).as_str();

        if self.is_promotion() {
            res += match self.promoted_piece() {
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Rook => "r",
                PieceType::Queen => "q",
                _ => unreachable!(),
            }
        }
        res
    }
}

#[must_use]
pub fn recognise_command(command: &str) -> CommandType {
    let words = command.split_whitespace().collect::<Vec<&str>>();
    match words[0] {
        "uci" => CommandType::Uci,
        "ucinewgame" => CommandType::UciNewGame,
        "isready" => CommandType::IsReady,
        "position" => CommandType::Position,
        "go" => {
            assert!(!words.is_empty(), "invalid uci command");

            if words[1] == "perft" {
                CommandType::Perft
            } else {
                CommandType::Go
            }
        }
        "setoption" => CommandType::SetOption,
        "stop" => CommandType::Stop,
        "quit" => CommandType::Quit,
        "d" => CommandType::D,
        _ => CommandType::Unknown,
    }
}

pub const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[must_use]
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
        println!("id name Panda 1.1");
        println!("option name Threads type spin default 1 min 1 max 256");
        println!("option name Hash type spin default 16 min 1 max 1048576");

        #[cfg(feature = "tuning")]
        list_params();

        println!("id author Sebastiano Rebonato-Scott");
        println!("uciok");
    }
}

pub fn parse_isready(command: &str) {
    if command == "isready" {
        println!("readyok");
    }
}

pub fn reset(b: &mut Board) {
    *b = Board::from(STARTPOS);
}

pub fn parse_position(command: &str, b: &mut Board) {
    reset(b);
    let words = command.split_whitespace().collect::<Vec<&str>>();
    assert!((words.len() >= 2), "invalid position command");
    match words[1] {
        "startpos" => {
            if words.len() != 2 {
                for w in words.iter().skip(3) {
                    //parse moves
                    let m = parse_move(w, b);
                    let Ok(_) = b.try_move(m) else {
                        panic!("Illegal move: {}", m.uci());
                    };
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
            *b = Board::from(&fen_string);
        }
        "moves" => {
            for w in words.iter().skip(2) {
                let m = parse_move(w, b);
                let Ok(_) = b.try_move(m) else {
                    panic!("invalid move {}", m.uci());
                };
            }
        }
        _ => {}
    }
}

pub fn parse_special_go(
    command: &str,
    b: &mut Board,
    tt: &TranspositionTable,
    opts: &UciOptions,
) -> MoveData {
    //special combination of go and position command by lichess bot api
    reset(b);
    let words = command.split_whitespace().collect::<Vec<&str>>();
    assert!((words.len() >= 2), "invalid position command");

    let mut end_of_moves = 0;

    match words[1] {
        "startpos" => {
            if words.len() != 2 {
                for (i, &w) in words.iter().enumerate().skip(3) {
                    if w.chars().collect::<Vec<char>>()[0] == 'w' {
                        end_of_moves = i;
                        break;
                    }
                    //parse moves
                    let m = parse_move(w, b);
                    let Ok(_) = b.try_move(m) else {
                        panic!("invalid move {}", m.uci());
                    };
                }
            }
        }
        "fen" => {
            let mut fen_string = String::new();
            for (i, &w) in words.iter().enumerate().skip(2) {
                if w.chars().collect::<Vec<char>>()[0] == 'w' {
                    end_of_moves = i;
                    break;
                }
                fen_string += w;
                if i != words.len() - 1 {
                    fen_string += " ";
                }
            }
            *b = Board::from(&fen_string);
        }
        "moves" => {
            for (i, &w) in words.iter().enumerate().skip(2) {
                if w.chars().collect::<Vec<char>>()[0] == 'w' {
                    end_of_moves = i;
                    break;
                }
                let m = parse_move(w, b);
                let Ok(_) = b.try_move(m) else {
                    panic!("invalid move {}", m.uci());
                };
            }
        }
        _ => panic!("invalid position command"),
    }

    let time_words = &words[end_of_moves..];

    let mut fake_go_command = String::from("go ");
    for w in time_words {
        fake_go_command += w;
        fake_go_command += " ";
    }

    parse_go(fake_go_command.as_str(), b, tt, opts)
}

pub fn parse_go(
    command: &str,
    position: &mut Board,
    tt: &TranspositionTable,
    opts: &UciOptions,
) -> MoveData {
    let words = command.split_whitespace().collect::<Vec<&str>>();
    //go wtime x btime x winc x binc x movestogo x

    let max_nodes = INFINITY as usize;
    let mut movetime = 0;
    // if go command sets move time for engine

    let (mut w_inc, mut b_inc, mut moves_to_go) = (0, 0, 0);

    if words[1] == "moves" {
        //special command lichess-bot protocol uses
        return parse_special_go(command, position, tt, opts);
    } else if words[1] == "movetime" {
        movetime = words[2].parse().expect("failed to convert movetime to int");
        let s = Searcher::new(tt);
        return s.start_search(position, 0, 0, 0, movetime, max_nodes, opts.threads);
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
        _ => return parse_special_go(command, position, tt, opts),
    }

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

    let s = Searcher::new(tt);
    s.start_search(
        position,
        engine_time,
        engine_inc,
        moves_to_go,
        movetime,
        max_nodes,
        opts.threads,
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

        println!("\ninfo depth {x} nodes {nodes} time {time} nps {nps}");
    } else {
        eprintln!("expected integer depth in perft command (go perft <depth>)");
    }
}

fn set_options(command: &str, opts: &mut UciOptions, tt: &mut TranspositionTable) {
    let words = command.split_whitespace().collect::<Vec<_>>();
    match words[..] {
        ["setoption", "name", "Hash", "value", x] => {
            opts.hash_size = x.parse().expect("hash size should be a +ve integer");
            tt.resize(opts.hash_size);
        }
        ["setoption", "name", "Threads", "value", x] => {
            opts.threads = x.parse().expect("thread count should be a +ve integer");
        }

        #[cfg(feature = "tuning")]
        _ => match words[2..] {
            ["SINGULARITY_DE_MARGIN", "value", x] => {
                set_param!(SINGULARITY_DE_MARGIN, x.parse().expect("should be integer"))
            }
            ["ASPIRATION_WINDOW", "value", x] => {
                set_param!(ASPIRATION_WINDOW, x.parse().expect("should be integer"))
            }
            ["BETA_PRUNING_DEPTH", "value", x] => {
                set_param!(BETA_PRUNING_DEPTH, x.parse().expect("should be integer"))
            }
            ["BETA_PRUNING_MARGIN", "value", x] => {
                set_param!(BETA_PRUNING_MARGIN, x.parse().expect("should be integer"))
            }
            ["ALPHA_PRUNING_DEPTH", "value", x] => {
                set_param!(ALPHA_PRUNING_DEPTH, x.parse().expect("should be integer"))
            }
            ["ALPHA_PRUNING_MARGIN", "value", x] => {
                set_param!(ALPHA_PRUNING_MARGIN, x.parse().expect("should be integer"))
            }
            ["SEE_PRUNING_DEPTH", "value", x] => {
                set_param!(SEE_PRUNING_DEPTH, x.parse().expect("should be integer"))
            }
            ["SEE_QUIET_MARGIN", "value", x] => {
                set_param!(SEE_QUIET_MARGIN, x.parse().expect("should be integer"))
            }
            ["SEE_NOISY_MARGIN", "value", x] => {
                set_param!(SEE_NOISY_MARGIN, x.parse().expect("should be integer"))
            }
            ["SEE_QSEARCH_MARGIN", "value", x] => {
                set_param!(SEE_QSEARCH_MARGIN, x.parse().expect("should be integer"))
            }
            ["LMP_DEPTH", "value", x] => {
                set_param!(LMP_DEPTH, x.parse().expect("should be integer"))
            }
            ["IIR_DEPTH_MINIMUM", "value", x] => {
                set_param!(IIR_DEPTH_MINIMUM, x.parse().expect("should be integer"))
            }
            ["HASH_MOVE_SCORE", "value", x] => {
                set_param!(HASH_MOVE_SCORE, x.parse().expect("should be integer"))
            }
            ["QUEEN_PROMOTION", "value", x] => {
                set_param!(QUEEN_PROMOTION, x.parse().expect("should be integer"))
            }
            ["WINNING_CAPTURE", "value", x] => {
                set_param!(WINNING_CAPTURE, x.parse().expect("should be integer"))
            }
            ["FIRST_KILLER_MOVE", "value", x] => {
                set_param!(FIRST_KILLER_MOVE, x.parse().expect("should be integer"))
            }
            ["SECOND_KILLER_MOVE", "value", x] => {
                set_param!(SECOND_KILLER_MOVE, x.parse().expect("should be integer"))
            }
            ["LOSING_CAPTURE", "value", x] => {
                set_param!(LOSING_CAPTURE, x.parse().expect("should be integer"))
            }
            ["UNDER_PROMOTION", "value", x] => {
                set_param!(UNDER_PROMOTION, x.parse().expect("should be integer"))
            }
            ["COUNTERMOVE_BONUS", "value", x] => {
                set_param!(COUNTERMOVE_BONUS, x.parse().expect("should be integer"))
            }
            ["QSEARCH_FP_MARGIN", "value", x] => {
                set_param!(QSEARCH_FP_MARGIN, x.parse().expect("should be integer"))
            }
            ["NMP_BASE", "value", x] => {
                set_param!(NMP_BASE, x.parse().expect("should be integer"))
            }
            ["NMP_FACTOR", "value", x] => {
                set_param!(NMP_FACTOR, x.parse().expect("should be integer"))
            }
            ["LMR_TACTICAL_BASE", "value", x] => {
                set_param!(LMR_TACTICAL_BASE, x.parse().expect("should be integer"))
            }
            ["LMR_TACTICAL_DIVISOR", "value", x] => {
                set_param!(LMR_TACTICAL_DIVISOR, x.parse().expect("should be integer"))
            }
            ["LMR_QUIET_BASE", "value", x] => {
                set_param!(LMR_QUIET_BASE, x.parse().expect("should be integer"))
            }
            ["LMR_QUIET_DIVISOR", "value", x] => {
                set_param!(LMR_QUIET_DIVISOR, x.parse().expect("should be integer"))
            }
            _ => {}
        },

        #[cfg(not(feature = "tuning"))]
        _ => {}
    }
}

pub fn print_thinking(depth: u8, eval: i32, s: &Thread, start: Instant) {
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
    let mut tt = TranspositionTable::in_megabytes(DEFAULT_HASH_SIZE);

    let mut opts = UciOptions::default();

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
                let move_data = parse_go(buffer.as_str(), &mut board, &tt, &opts);
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
            CommandType::SetOption => set_options(buffer.as_str(), &mut opts, &mut tt),
            CommandType::UciNewGame => board = Board::from(STARTPOS),
            _ => {}
        }
    }
}
