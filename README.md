# Panda
Panda is a chess engine written in Rust (still a work in progress). I work on this project for fun when I have the time. It is called Panda because:
- pandas are black and white like a chess board
- pandas are pretty cool
- red pandas are also pretty cool, and they are orange (like Rust)

![](logo4.png)

## What Makes Panda Interesting?

In terms of strength, Panda is pretty unremarkable - currently somewhere around 2620. One fairly original idea is that it considers uncertainty in evaluation of a position instead of just returning one evaluation like most engines do. The intention is that this makes it value practical chances (i.e. expected score from the game) over just maximising its evaluation. Although its current NNUE evaluation doesn't use these, it was trained on data which used this technique. However, the purpose of this project is mainly just for me to practice Rust and to combine two of my hobbies (programming and chess).

## Lichess Bot

I used the repo https://github.com/lichess-bot-devs/lichess-bot to create a lichess bot for Panda. Unfortunately it probably won't be online that much because I'm hosting it locally.

[Panda Lichess Bot](https://lichess.org/@/BotNickal)

## Todo:
- faster perft results
- tune thresholds in search function
- endgame tablebases
- stronger NNUE

## Usage:
- [Download Rust](https://www.rust-lang.org/)
- Build and run the project using ```make run``` or build to an executable using ```make build```
- connect to a UCI gui such as CuteChess or Arena

## Acknowledgements
Here are some of the many resources without which this engine would be much less strong:
- [Chess Programming Wiki](https://www.chessprogramming.org/Main_Page)
- [BBC Chess Engine](https://github.com/maksimKorzh/bbc) + videos
- Open source chess projects such as Ethereal, Weiss and Cozy Chess, which have extremely clear and helpful documentation
- [bullet](https://github.com/jw1912/bullet/tree/main), which I used to train the neural network
