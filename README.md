[![Build Status](https://travis-ci.org/kondrak/rust64.svg)](https://travis-ci.org/kondrak/rust64)
[![Build status](https://ci.appveyor.com/api/projects/status/77otp2475g7v95mb?svg=true)](https://ci.appveyor.com/project/kondrak/rust64)

# Rust64 - a C64 emulator written in Rust
This is my attempt to study the Rust programming language and have fun at the same time. The goal is to present in the least obfuscated way how the Commodore 64 works and what's happening behind the scenes once you start a program. Emulation is cycle based and fairly accurate at this point.

The emulator has a built-in visual debugger which lets you view the contents of each memory page in RAM, Color RAM, VIC registers, CIA registers and SID registers. The VIC window is a ICU64-style raster debugger where each pixel represents one VIC cycle and any events occuring at that time.

Major dependencies
------------------
- minifb: https://crates.io/crates/minifb (works out of the box)
- sdl2: https://crates.io/crates/sdl2 (requires extra steps, see [here](https://github.com/AngryLawyer/rust-sdl2) for instructions)

Requires Rust 1.5.0 or higher to compile and run.

### Youtube demo #1:
[![Screenshot](http://kondrak.info/images/rust64_youtube.png?raw=true)](https://www.youtube.com/watch?v=b6OSsTPwLaE)
### Youtube demo #2:
[![Screenshot](http://kondrak.info/images/rust64_youtube2.png?raw=true)](https://www.youtube.com/watch?v=g4d_1vPV6So)
### Screenshot:
[![Screenshot](http://kondrak.info/images/rust64_github_prev.png?raw=true)](http://kondrak.info/images/rust64_github.png?raw=true)



Build instructions
------------------
```
cargo build
cargo run --release
```

You can pass a .prg file as a command line parameter to load it into memory once the emulator boots (just type RUN to start the program):
```
cargo run --release prgs/colors.prg
```
To run with double-sized window:
```
cargo run --release x2 prgs/colors.prg
```
To run with double-sized window and debug windows enabled:
```
cargo run --release x2 debugger prgs/colors.prg
```

C64 and special key mappings
-------------------
```
ESC     - Run/Stop
END     - Restore
TAB     - Control
LCTRL   - C=
`       - <-
-       - +
INS     - &
HOME    - CLR/Home
BSPACE  - INST/DEL
[       - @
]       - *
DEL     - ^
;       - :
'       - ;
\       - =
F11     - start asm output to console (very slow!)
F12     - reset C64
RCTRL   - joystick fire button
NUMLOCK - toggle between joystick ports 1 and 2 (default: port 2)

In debugger window:
PGUP/PGDWN - flip currently displayed memory page
HOME/END   - switch currently displayed memory banks between RAM, Color RAM, VIC, CIA and SID
```

TODO
------------------
- serial bus/disk drives (d64, t64, tap)
- implement remaining undocumented ops
- switch from SDL2 to [cpal](https://github.com/tomaka/cpal) for audio once it supports OSX
- improve SID emulation

Known Issues
------------------
- missing serial bus may cause some very specific programs to perform incorrectly or get stuck in infinite loops
- elaborate programs that require very precise timing are not running correctly yet

This is an on-off WIP project, so update frequency may vary.

Resources
------------------
The following documents and websites have been used to create this emulator:

- http://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt
- http://frodo.cebix.net/ (inspired the VIC-II and SID implementaiton)
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
- https://svn.code.sf.net/p/vice-emu/code/testprogs/
- http://www.classiccmp.org/cini/pdf/Commodore/ds_6581.pdf

Special thanks
------------------
- [Daniel Collin](https://twitter.com/daniel_collin) and Magnus "Pantaloon" Sjöberg for excessive test programs!
- [Jake Taylor](https://twitter.com/ferristweetsnow) for general Rust tips!
