# Rust64 - a C64 emulator written in Rust
Some people learn a new language with "Hello world". Others toy around with it implementing amazing graphical effects. I decided to write a C64 emulator. 

From scratch. 

Currently depends on rust-sdl2 bindings, follow the readme here to compile: https://github.com/AngryLawyer/rust-sdl2

Status:

- <code>CPU</code>      - done
- <code>memory</code>   - done
- <code>VIC-II</code>   - missing sprites (todo)
- <code>CIA</code>      - in progress
- <code>IO</code>       - in progress
- <code>SID</code>      - todo
- <code>load prg</code> - todo
- <code>drives</code>   - todo


This is an on-off WIP project, so update frequency may vary.

Resources used to create this emulator:

- http://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt
- http://frodo.cebix.net/ (inspired the VIC-II implementaiton)
- https://www.c64-wiki.com
- http://archive.6502.org/datasheets/mos_6526_cia.pdf
