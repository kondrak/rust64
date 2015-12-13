# convenience makefile for Emacs M-x recompile 
#export PATH := /cygdrive/d/Program Files (x86)/Rust/bin/:$(PATH)

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
