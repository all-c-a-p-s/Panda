# Panda
Chess engine written in Rust (WIP).

Current elo ~2000

## Features:
- magic bitboards
- negamax search
- alpha/beta pruning
- quiescence search
- mvv-lva move ordering
- iterative deepening
- aspiration windows
- principal variation search
- late move reductions
- null move pruning
- piece square tables
- mobility evaluation + open files
- pawn structure evaluation + passed pawns
- tapered eval
- hashing

## Todo:
- king safety evaluation
- more search improvements
- eval tuning

## Usage:
- download [rust](https://www.rust-lang.org/)
- build and run the project (NOTE: you must use ```--release``` mode or the magic bitboards will not work)
- connect to a uci gui such as cutechess or arena

