# convenience makefile for Emacs M-x recompile 

# misleading name, I know - this makefile is used for Emacs debugging for now 
all:
	@export PATH
	@cargo run
debug:
	@export PATH
	cargo build
release:
	@export PATH
	cargo build --release
