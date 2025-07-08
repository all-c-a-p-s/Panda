# Baby Panda

Baby Panda is a highly experimental version of Panda, which updates a neural network during the search, and uses this for move ordering. This idea is interesting to try, but not very effective in practice - Baby Panda's strength is probably about 800 elo weaker than its classical counterpart.

## What Makes Baby Panda Interesting?

Baby panda trains a neural network on beta cutoffs as they occur, and uses this as the sole heuristic for move ordering (except for TT if a TT move is present).

## Usage (tricky and only tested on MacOS)
### 1. Install Rust and Clone Repo (Easy Part)
- [Download Rust](https://www.rust-lang.org/)
- clone this branch of the repo
  
### 2. Setup ```tch-rs``` (Tricky Part)
Follow instructions [here](https://github.com/LaurentMazare/tch-rs) to install tch-rs, also install libtorch as instructed
Set the following environment variables:
```
export LIBTORCH=path/to/libtorch
export DYLD_LIBRARY_PATH=path/to/libtorch
export LIBTORCH_INCLUDE=path/to/libtorch
export LIBTORCH_LIB=path/to/libtorch
```

On MacOS, you can permanently set these environment variables by adding the lines above to your ```.zshenv```
  
### 3. Run Baby Panda
- Build and run the project using ```make run``` or build to an executable using ```make build```
- connect to a UCI gui such as [CuteChess](https://cutechess.com/)

## Credits
Same as regular version of Panda
