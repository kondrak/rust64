export PATH := /cygdrive/c/Program Files/Rust stable 1.2/bin/:$(PATH)

all:
	@export PATH
	@rustc main.rs
	@./main.exe