use crate::{helper::coordinate, *};

pub const START_DEPTH: usize = 3;

pub fn perft(depth: usize, b: Board) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut total = 0;
    let moves = gen_legal(b);
    for i in 0..MAX_MOVES {
        if moves.moves[i] == NULL_MOVE {
            break;
        }
        let updated_board = make_move(moves.moves[i], b);
        let added = perft(depth - 1, updated_board);
        if depth == START_DEPTH {
            println!("{}{}: {}", coordinate(moves.moves[i].square_from()), coordinate(moves.moves[i].square_to()), added);
        }
        total += added;
    }

    total
}
