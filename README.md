# Panda

Panda is a chess engine written in Rust (still a work in progress). I like working on this project for fun when I have free time. It is called Panda because:
- pandas are black and white like a chess board
- pandas are pretty cool
- red pandas are also pretty cool, and they are orange (like Rust)

## Stats
|                           Version                            |     Release Date    | [CCRL 40/15](https://www.computerchess.org.uk/ccrl/4040/) | [CCRL Blitz](https://computerchess.org.uk/ccrl/404/) | Notes |
| :-----------------------------------------------------------:|:-------------------:|:---------:|:----:|:---------------------------:|
| [1.2](https://github.com/all-c-a-p-s/Panda/releases/tag/1.2) |  10th     June 2026 | 3300 (est.)|  -   | Search Improvements + New Net  |
| [1.1](https://github.com/all-c-a-p-s/Panda/releases/tag/1.1) |  5th    August 2025 | 3227      |  -   | Major Search Improvements   |
| [1.0](https://github.com/all-c-a-p-s/Panda/releases/tag/1.0) |  20th    April 2025 | 3134      |  -   |       First Release         |


## What Makes Panda Interesting?
Panda definitely isn't anything revolutionary but its code contains some good ideas and some original ideas. Hopefully, it even includes some ideas which fall into both of those categories.

In particular, two major original ideas which Panda uses:
- internal aspiration windows (aspiration windows inside the recursive alpha-beta search)
- custom datagen method which uses hindsight to re-evaluate positions from the game.

As far as I know, these techniques are completely unique to this engine.

There are also many smaller ideas which (as far as I'm aware) are new, such as Panda's razoring approach and some of Panda's move ordering heuristics.

By far the most exciting game I've seen it play is [this one](https://www.chess.com/analysis/library/22UV4Zu2Bg), played by a pre-1.0 version of Panda against a really cool MCTS engine called [Javelin](https://github.com/TomaszJaworski777/Javelin).

## Features (very brief version)
- UCI compliant (no GUI)
- magic bitboard move generation
- Alpha-Beta search with various enhancements
- NNUE with architecture `(768->384)x2 -> 1`, trained on self-play
- custom datagen method


## TUI

Inspired by other engines (Viridithas in particular), I created a TUI for Panda, which it uses unless it receives the `uci` command. My dad wanted to try using the engine with a physical board (rather than a GUI like CuteChess), so I thought it would be good to make the interface a little nicer to look at:

<img width="1354" height="911" alt="Screenshot 2026-06-10 at 14 54 24" src="https://github.com/user-attachments/assets/46f1c2a8-a33b-4029-9cd4-3048a7b9f594" />

## Usage

You can clone the repo for dev build, or get the latest published version from [Releases](https://github.com/all-c-a-p-s/Panda/releases). To build/run any version of Panda, you will need to [Download Rust](https://www.rust-lang.org/).

> [!IMPORTANT]
> If you would like a fully optimised build, then you should use the pgo-optimised compilation (which is the default in the makefile). In my testing, this seems to be worth 5-10 elo, so please make sure you use this compilation if you are formally testing the engine!
> This adds the following dependencies:
> ```bash
> rustup component add llvm-tools-preview
> cargo install cargo-binutils
> ```
> (You probably already have LLVM installed, but if not you will also need to run a command similar to `sudo apt install llvm` to install it.)
> 
> You can then build the project using `make` or `make pgo`.
> The pgo compilation itself will take a few minutes. Thanks for you patience :)

Alternatively, you can build the project using `make build` or run it immediately with `make run` if you just want to try out the engine in a context where optimal performance isn't super crucial.

If you want to play against the engine or watch it play, you can connect to a UCI gui such as [CuteChess](https://cutechess.com/).

## Todo
- endgame tablebases
- stronger NNUE

## Credits
- [BBC Chess Engine](https://github.com/maksimKorzh/bbc) + videos, which explain magic bitboards very clearly
- [Weiss](https://github.com/TerjeKir/weiss), which has incredibly clear code in its search function
- [Ethereal](https://github.com/AndyGrant/Ethereal) - Panda's SEE implementation is entirely based on Ethereal's
- [Carp](https://github.com/dede1751/carp) - extremely clear Rust code, which is always useful to read when I'm struggling to understand something
- Jamie Whiting for creating [bullet](https://github.com/jw1912/bullet/tree/main), which I use to train Panda's networks, and [akimbo](https://github.com/jw1912/akimbo/tree/main), which Panda's Lazy SMP implementation is inspired by
- [@mcthouacbb](https://github.com/mcthouacbb) for several helpful suggestions
- [@rwbc](https://github.com/rwbc) for kindly providing windows binaries
- [weather-factory](https://github.com/jnlt3/weather-factory) for SPSA tuning search parameters
