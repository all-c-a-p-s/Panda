# Panda
Chess engine written in Rust (WIP).

Current elo ~1900

## Features:
- magic bitboards
- negamax
- alpha/beta pruning
- quiescence search
- mvv-lva move ordering
- iterative deepening
- principal variation search
- piece square tables
- mobility evaluation
- pawn structure evaluation
- basic tapered eval
- hashing (todo: incremental update hash key)

## Todo:
- king safety evaluation
- late move reductions, null move pruning etc.
- opening book

## Usage:
- download [rust](https://www.rust-lang.org/)
- build and run the project (NOTE: you must use ```--release``` mode or the magic bitboards will not work)
- connect to a uci gui such as cutechess or arena

