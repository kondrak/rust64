# convenience makefile for Emacs M-x recompile 

all:
	@export PATH
	RUST_BACKTRACE=1 cargo run --release debugger
debug:
	@export PATH
	cargo build
release:
	@export PATH
	cargo build --release
