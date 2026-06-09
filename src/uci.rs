use std::time::Instant;

use crate::helper::BIG_INF;
#[cfg(feature = "tuning")]
use crate::set_param;

#[cfg(feature = "tuning")]
use crate::search::{list_params, params};

use crate::thread::{Searcher, Thread};
use crate::transposition::TranspositionTable;
use crate::types::{Piece, PieceType, Square};
use crate::{
    Board, CASTLING_FLAG, Colour, EN_PASSANT_FLAG, Move, MoveData, NO_FLAG, PROMOTION_FLAG,
    coordinate, encode_move, perft, piece_type, square,
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

#[cfg(feature = "tuning")]
macro_rules! try_set_param {
    ($name:expr, $value:expr, $($param:ident),* $(,)?) => {
        match $name {
            $(stringify!($param) => {
                set_param!($param, $value.parse().expect("should be integer"));
            },)*
            _ => {}
        }
    };
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
pub fn recognise_command(words: &[&str]) -> CommandType {
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

pub fn parse_uci(words: &[&str]) {
    if words == ["uci"] {
        println!("id name Panda 1.2");
        println!("option name Threads type spin default 1 min 1 max 256");
        println!("option name Hash type spin default 16 min 1 max 1048576");

        #[cfg(feature = "tuning")]
        list_params();

        println!("id author Sebastiano Rebonato-Scott");
        println!("uciok");
    }
}

pub fn parse_isready(words: &[&str]) {
    if words == ["isready"] {
        println!("readyok");
    }
}

pub fn reset(b: &mut Board) {
    *b = Board::from(STARTPOS);
}

fn apply_uci_move(b: &mut Board, w: &str) {
    let m = parse_move(w, b);
    let Ok(_) = b.try_move(m) else {
        panic!("invalid move {}", m.uci());
    };
}

fn parse_position_words(words: &[&str], b: &mut Board, end: usize) {
    assert!((words.len() >= 2), "invalid position command");

    match words[1] {
        "startpos" => {
            if end != 2 {
                for &w in words.iter().take(end).skip(3) {
                    apply_uci_move(b, w);
                }
            }
        }
        "fen" => {
            let fen_string = words
                .iter()
                .take(end)
                .copied()
                .skip(2)
                .collect::<Vec<_>>()
                .join(" ");
            *b = Board::from(&fen_string);
        }
        "moves" => {
            for &w in words.iter().take(end).skip(2) {
                apply_uci_move(b, w);
            }
        }
        _ => {}
    }
}

pub fn parse_position(words: &[&str], b: &mut Board) {
    reset(b);
    parse_position_words(words, b, words.len());
}

pub fn parse_special_go(
    words: &[&str],
    b: &mut Board,
    tt: &TranspositionTable,
    opts: &UciOptions,
) -> MoveData {
    //special combination of go and position command by lichess bot api
    reset(b);
    assert!((words.len() >= 2), "invalid position command");

    let end_of_moves = words
        .iter()
        .position(|x| x.starts_with('w'))
        .expect("invalid go command");

    parse_position_words(words, b, end_of_moves);

    let mut go_words = vec!["go"];
    go_words.extend_from_slice(&words[end_of_moves..]);

    parse_go(&go_words, b, tt, opts)
}

pub fn parse_go(
    words: &[&str],
    position: &mut Board,
    tt: &TranspositionTable,
    opts: &UciOptions,
) -> MoveData {
    //go wtime x btime x winc x binc x movestogo x

    let max_nodes = BIG_INF as usize;
    let mut movetime = 0;
    // if go command sets move time for engine

    let (mut w_inc, mut b_inc, mut moves_to_go) = (0, 0, 0);

    if words[1] == "moves" {
        //special command lichess-bot protocol uses
        return parse_special_go(words, position, tt, opts);
    } else if words[1] == "movetime" {
        movetime = words[2].parse().expect("failed to convert movetime to int");
        let s = Searcher::new(tt);
        return s.start_search(position, 0, 0, 0, movetime, max_nodes, opts.threads);
    }

    let w_time = words[2].parse().expect("failed to convert wtime to int");
    let b_time = words[4].parse().expect("failed to convert btime to int");

    match words[5..] {
        [] => {}
        ["movestogo", x] => moves_to_go = x.parse().expect("failed to convert movestogo to int"),
        ["winc", x, "binc", y] => {
            w_inc = x.parse().expect("failed to convert winc to int");
            b_inc = y.parse().expect("failed to covnert binc to int");
        }
        ["winc", x, "binc", y, "movestogo", z] => {
            w_inc = x.parse().expect("failed to convert winc to int");
            b_inc = y.parse().expect("failed to covnert binc to int");
            moves_to_go = z.parse().expect("failed to convert movestogo to int");
        }
        _ => return parse_special_go(words, position, tt, opts),
    }

    let (engine_time, engine_inc) = match position.side_to_move {
        Colour::White => (w_time, w_inc),
        Colour::Black => (b_time, b_inc),
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

fn parse_perft(words: &[&str], position: &mut Board) {
    match words[..] {
        ["go", "perft", x] => {
            let Ok(x) = x.parse() else {
                panic!("expected integer depth in perft command (go perft <depth>)");
            };
            let start = Instant::now();
            let nodes = perft::<true, false, false>(x, position, Some(x));
            let micros = start.elapsed().as_micros() as usize;

            #[allow(clippy::manual_checked_ops)]
            let nps = if micros == 0 {
                nodes * 1_000_000
            } else {
                nodes * 1_000_000 / micros
            };

            let time = micros / 1000;

            println!("\ninfo depth {x} nodes {nodes} time {time}  nps {nps}");
        }
        _ => panic!("expected perft command in the following format: go perft <depth>"),
    }
}

fn set_options(words: &[&str], opts: &mut UciOptions, tt: &mut TranspositionTable) {
    match words[..] {
        ["setoption", "name", "Hash", "value", x] => {
            opts.hash_size = x.parse().expect("hash size should be a +ve integer");
            tt.resize(opts.hash_size);
        }
        ["setoption", "name", "Threads", "value", x] => {
            opts.threads = x.parse().expect("thread count should be a +ve integer");
        }

        #[cfg(feature = "tuning")]
        _ => {
            if let [name, "value", x] = words[2..] {
                try_set_param!(
                    name,
                    x,
                    SINGULARITY_DE_MARGIN,
                    ASPIRATION_WINDOW,
                    SEE_PRUNING_DEPTH,
                    SEE_QUIET_MARGIN,
                    SEE_NOISY_MARGIN,
                    SEE_QSEARCH_MARGIN,
                    LMP_DEPTH,
                    IIR_DEPTH_MINIMUM,
                    HASH_MOVE_SCORE,
                    QUEEN_PROMOTION,
                    WINNING_CAPTURE,
                    FIRST_KILLER_MOVE,
                    LOSING_CAPTURE,
                    UNDER_PROMOTION,
                    QSEARCH_FP_MARGIN,
                    NMP_BASE,
                    NMP_FACTOR,
                    LMR_TACTICAL_BASE,
                    LMR_TACTICAL_DIVISOR,
                    LMR_QUIET_BASE,
                    LMR_QUIET_DIVISOR,
                    RFP_BETA_WEIGHT,
                    NMP_BETA_WEIGHT,
                    STAND_PAT_BETA_WEIGHT,
                );
            }
        }

        #[cfg(not(feature = "tuning"))]
        _ => {}
    }
}

pub fn print_thinking(depth: u8, eval: i32, s: &Thread, start: Instant) {
    let pv = s.pv[0]
        .iter()
        .take(s.pv_length[0])
        .map(|m| m.uci())
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "info depth {} score cp {} nodes {} pv {} time {} nps {}",
        depth,
        eval,
        s.nodes,
        pv,
        start.elapsed().as_millis(),
        {
            let micros = start.elapsed().as_micros() as usize;
            #[allow(clippy::manual_checked_ops)]
            if micros == 0 {
                s.nodes * 1_000_000
            } else {
                s.nodes * 1_000_000 / micros
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
        std::io::stdin().read_line(&mut buffer).unwrap();

        let buffer = buffer.trim_end();
        if buffer == "quit" {
            break;
        }

        let words = buffer.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            continue;
        }

        let command_type = recognise_command(&words);
        match command_type {
            CommandType::D => board.print_board(),
            CommandType::Uci => parse_uci(&words),
            CommandType::IsReady => parse_isready(&words),
            CommandType::Position => parse_position(&words, &mut board),
            CommandType::Go => {
                let move_data = parse_go(&words, &mut board, &tt, &opts);
                if move_data.m.is_null() {
                    break;
                }
                print!("bestmove ");
                println!("{}", move_data.m.uci());
            }
            CommandType::Perft => parse_perft(&words, &mut board),
            CommandType::SetOption => set_options(&words, &mut opts, &mut tt),
            CommandType::UciNewGame => board = Board::from(STARTPOS),
            _ => {}
        }
    }
}
