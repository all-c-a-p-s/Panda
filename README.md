# Panda

Panda is a chess engine written in Rust (still a work in progress). I like working on this project for fun when I have free time. It is called Panda because:
- pandas are black and white like a chess board
- pandas are pretty cool
- red pandas are also pretty cool, and they are orange (like Rust)

## Stats
|                           Version                            |     Release Date    | [CCRL 40/15](https://www.computerchess.org.uk/ccrl/4040/) | [CCRL Blitz](https://computerchess.org.uk/ccrl/404/) | Notes |
| :-----------------------------------------------------------:|:-------------------:|:---------:|:----:|:---------------------------:|
| [1.1](https://github.com/all-c-a-p-s/Panda/releases/tag/1.1) |  5th    August 2025 |3240 (est.)|  -   | Major Search Improvements   |
| [1.0](https://github.com/all-c-a-p-s/Panda/releases/tag/1.0) |  20th    April 2025 | 3134      |  -   |       First Release         |


## What Makes Panda Interesting?
Panda definitely isn't anything revolutionary but its code contains some good ideas and some original ideas. Hopefully, it even includes some ideas which fall into both of those categories.

In particular, two major original ideas which Panda uses:
- internal aspiration windows (aspiration windows inside the recursive alpha-beta search)
- custom datagen method which uses hindsight to re-evaluate positions from the game.

As far as I know, these techniques are completely unique to this engine.

By far the most exciting game I've seen it play is [this one](https://www.chess.com/analysis/library/22UV4Zu2Bg) against a really cool MCTS engine called [Javelin](https://github.com/TomaszJaworski777/Javelin).

## Features (very brief version)
- UCI compliant (no GUI)
- magic bitboard move generation
- Alpha-Beta search with various enhancements
- NNUE with architecture (768->256)x2 -> 1, trained on self-play
- custom datagen method


## Lichess Bot

I used the repo https://github.com/lichess-bot-devs/lichess-bot to create a lichess bot for Panda. Unfortunately, it isn't online very often since I host it locally.

[Panda Lichess Bot](https://lichess.org/@/RedPandaBot)

## Todo
- endgame tablebases
- stronger NNUE

## Usage
- [Download Rust](https://www.rust-lang.org/)
- clone the repo for dev build, or get the latest published version from [Releases](https://github.com/all-c-a-p-s/Panda/releases)
- Build and run the project using ```make run``` or build to an executable using ```make build```
- connect to a UCI gui such as [CuteChess](https://cutechess.com/)

## Credits
- [BBC Chess Engine](https://github.com/maksimKorzh/bbc) + videos, which explain magic bitboards very clearly
- [Weiss](https://github.com/TerjeKir/weiss), which has incredibly clear code in its search function
- [Ethereal](https://github.com/AndyGrant/Ethereal) - Panda's SEE implementation is entirely based on Ethereal's
- [Carp](https://github.com/dede1751/carp) - extremely clear Rust code, which is always useful to read when I'm struggling to understand something
- Jamie Whiting for creating [bullet](https://github.com/jw1912/bullet/tree/main), which I use to train Panda's networks, and [akimbo](https://github.com/jw1912/akimbo/tree/main), which Panda's Lazy SMP implementation is inspired by
- [@mcthouacbb](https://github.com/mcthouacbb) for several helpful suggestions
- [weather-factory](https://github.com/jnlt3/weather-factory) for SPSA tuning search parameters
