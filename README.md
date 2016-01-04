# Rust64 - a C64 emulator written in Rust
Some people learn a new language with "Hello world". Others toy around with it implementing amazing graphical effects. I decided to write a C64 emulator. This is my attempt to learn the Rust programming language and have fun at the same time.

Dependencies:
- rust-sdl2 bindings, follow the readme here to compile: https://github.com/AngryLawyer/rust-sdl2
- minifb: https://github.com/emoon/rust_minifb

Requires Rust 1.5.0 to compile and run.

TODO:
- sprites
- SID emulation
- serial bus/disk drives


This is an on-off WIP project, so update frequency may vary.

Resources used to create this emulator:

- http://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt
- http://frodo.cebix.net/ (inspired the VIC-II implementaiton)
- https://www.c64-wiki.com
- http://www.oxyron.de/html/opcodes02.html
- http://www.6502.org/tutorials/6502opcodes.html
- http://www.pagetable.com/c64rom/c64rom_en.html
- http://archive.6502.org/datasheets/mos_6526_cia.pdf
