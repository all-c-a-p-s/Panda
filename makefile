EXE = BabyPanda

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

build:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

run:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
	./$(NAME)

datagen:
	cargo rustc --release --features datagen -- -C target-cpu=native --emit link=$(NAME)
	./$(NAME) datagen

profile:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
	./$(NAME) profile

debug:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
	./$(NAME) debug

test:
	cargo test --release

testd:
	cargo test --release -- --nocapture


