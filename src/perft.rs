use crate::*;

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
        total += perft(depth - 1, updated_board);
    }
    total
}
