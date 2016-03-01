[![Build Status](https://travis-ci.org/kondrak/rust64.svg)](https://travis-ci.org/kondrak/rust64)
[![Build status](https://ci.appveyor.com/api/projects/status/77otp2475g7v95mb?svg=true)](https://ci.appveyor.com/project/kondrak/rust64)

# Rust64 - a C64 emulator written in Rust
Some people learn a new language with "Hello world". I decided to write a C64 emulator. This is my attempt to learn the Rust programming language and have fun at the same time.

Dependencies
------------------
- minifb: https://github.com/emoon/rust_minifb, https://crates.io/crates/minifb

Requires Rust 1.5.0 to compile and run.

Youtube demo:

[![Screenshot](http://kondrak.info/images/rust64_youtube.png?raw=true)](https://www.youtube.com/watch?v=b6OSsTPwLaE)

Screenshot:

![Screenshot](http://kondrak.info/images/rust64_github.png?raw=true)

The emulator comes with a memory debugger - press PgUp/PgDwn to flip between memory pages and Home/End to change memory banks (Ram, VIC registers, CIA registers, Color Ram). The VIC window is a ICU64-style raster debugger, each pixel representing one VIC cycle and events associated with it.

Build instructions
------------------
```
cargo build
cargo run --release
```

You can pass a .prg file as a command line parameter to load it into memory once the emulators boots (type RUN to start the program):
```
cargo run --release tests/bgcolor.prg
```
To run with double-sized window:
```
cargo run --release x2 tests/bgcolor.prg
```
To run with double-sized window and debug windows enabled:
```
cargo run --release x2 debugger tests/bgcolor.prg
```

C64 and special key mappings
-------------------
```
ESC   - Run/Stop
END   - Restore
LCTRL - C=
F11   - start asm output to console (very slow!)
F12   - reset C64

In debugger window:
PGUP/PGDWN - flip currently displayed memory page
HOME/END   - change currently displayed memory banks between RAM, Color RAM, VIC, CIA and SID
```

TODO
------------------
- SID emulation
- serial bus/disk drives (d64, t64)
- implement remaining undocumented ops

Known Issues
------------------
Due to lack of any SID register and serial bus handling, some programs may not perform correctly or get stuck in infinite loops.

This is an on-off WIP project, so update frequency may vary.

Resources
------------------
The following has been used to create this emulator:

- http://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt
- http://frodo.cebix.net/ (inspired the VIC-II implementaiton)
- https://www.c64-wiki.com
- http://www.oxyron.de/html/opcodes02.html
- http://www.6502.org/tutorials/6502opcodes.html
- http://www.pagetable.com/c64rom/c64rom_en.html
- http://archive.6502.org/datasheets/mos_6526_cia.pdf
- https://www.yoyogames.com/tech_blog/95
- http://code.google.com/p/hmc-6502/source/browse/trunk/emu/testvectors/AllSuiteA.asm
- https://t.co/J40UKu7RBf
- http://www.waitingforfriday.com/index.php/Commodore_SID_6581_Datasheet
- http://sta.c64.org/cbm64mem.html
