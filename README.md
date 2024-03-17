# Panda
Chess engine written in rust (WIP).

Current elo ~1800

## Features:
- magic bitboards
- negamax
- alpha/beta pruning
- quiescence search
- mvv-lva move ordering
- piece square tables
- mobility evaluation
- basic tapered eval

## Todo:
- pawn struture evaluation
- king safety evaluation
- search improvements
- opening book
- hashing

## Usage:
- download [rust](https://www.rust-lang.org/)
- build and run the project (NOTE: you must use ```--release``` mode or the magic bitboards will not work)
- currently no uci protocol. input ```w``` to play white or ```b``` to play black. then input moves in the format e.g. e2e4, g1f3, h7h8Q

