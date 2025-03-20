# Panda
Panda is a chess engine written in Rust (still a work in progress). It is called Panda because:
- pandas are black and white like a chess board
- pandas are pretty cool
- red pandas are also pretty cool, and they are orange (like Rust)

![](logo.jpeg)

## Lichess Bot

I used the repo https://github.com/lichess-bot-devs/lichess-bot to create a lichess bot for Panda. Unfortunately it probably won't be online that much because I'm hosting it locally.

[Panda Lichess Bot](https://lichess.org/@/BotNickal)

## Features:
- __Move Generation__
  - Magic Bitboards
  - Make/Unmake Approach
- __Search__
  - Negamax Search
  - Quiescence Search
  - Principal Variation Search
  - Iterative Deepening
  - Transposition Table
  - Aspiration Windows
  - Internal Iterative Deepening
  - __Pruning__
    - Alpha/Beta Pruning
    - Null Move Pruning
    - Beta Pruning/Reverse Futility Pruning
    - Alpha Pruning/Futility Pruning
    - Razoring into Quiescence Search
    - SEE Pruning
    - Mate Distance Pruning
    - Late Move Reductions
  - __Move Ordering__
    - Moves are ordered as follows:
    - Move from transposition table (if available)
    - PV Move
    - Winning captures by SEE (ordered by MVV/LVA)
    - Killer Moves
    - Moves Sorted by History Heuristic
    - Losing captures by SEE (ordered by MVV/LVA)
    - Underpromotions
- __Evaluation__
  - Piece-Square Tables with middlegame and endgame weights
  - Mobility Calculations
  - Pawn Structure Evaluation + Passed Pawns
  - Tapered Evaluation
  - Mobility Score
  - King Safety
  + Some other stuff

## Todo:
- faster perft results (big room for improvement here)
- tune thresholds in search function
- endgame tablebases
- NNUE

## Usage:
- [Download Rust](https://www.rust-lang.org/)
- Build and run the project (NOTE: you must use ```--release``` mode or the magic bitboards will not work)
- connect to a UCI gui such as CuteChess or Arena

## Acknowledgements
Here are some of the many resources without which this engine would be much less strong:
- [Chess Programming Wiki](https://www.chessprogramming.org/Main_Page)
- [BBC Chess Engine](https://github.com/maksimKorzh/bbc) + videos
- Most of all, open source chess engines such as Stockfish, Ethereal and Weiss which have extremely clear and helpful documentation 
