# convenience makefile for Emacs M-x recompile 

# misleading name, I know - this makefile is used for Emacs debugging for now 
all:
	@export PATH
	RUST_BACKTRACE=1 cargo run --release debugger
debug:
	@export PATH
	cargo build
release:
	@export PATH
	cargo build --release
