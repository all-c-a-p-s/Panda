EXE = Panda
TEST_EXE = panda_test

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

ifeq ($(OS),Windows_NT)
	TEST_NAME := $(TEST_EXE).exe
else
	TEST_NAME := $(TEST_EXE)
endif

default: pgo

PGO_DIR = $(CURDIR)/pgo-data
PROFDATA = $(PGO_DIR)/merged.profdata

pgo: pgo_gen pgo_run pgo_merge pgo_build pgo_clean

pgo_gen:
	rm -rf $(PGO_DIR)
	mkdir -p $(PGO_DIR)
	RUSTFLAGS="-Cprofile-generate=$(PGO_DIR) -C target-feature=+crt-static" \
		cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

pgo_run:
	LLVM_PROFILE_FILE="$(PGO_DIR)/panda-%p-%m.profraw" ./$(NAME) < bench.txt

pgo_merge:
	llvm-profdata merge -o $(PROFDATA) $(PGO_DIR)/*.profraw
	ls -lh $(PROFDATA)

pgo_build:
	RUSTFLAGS="-Cprofile-use=$(PROFDATA) -C target-feature=+crt-static" \
		cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

pgo_clean:
	rm -rf $(PGO_DIR)
	rm -f *.pdb

for_sprt:
	cargo rustc --release -- -C target-cpu=native --emit link=$(TEST_NAME)

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

tune:
	cargo rustc --release --features tuning -- -C target-cpu=native --emit link=$(NAME)

test:
	cargo test --release

testd:
	cargo test --release -- --nocapture



