extern crate minifb;

#[macro_use]
mod utils;
mod c64;
mod debugger;

use minifb::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut prg_to_load  = String::new();
    let mut debugger_on  = false;
    let mut window_scale = Scale::X1;

    // process cmd line params
    for i in 1..args.len() {
        if args[i] == "debugger" {
            debugger_on = true;
        }
        else if args[i] == "x2" {
            window_scale = Scale::X2;
        }
        else {
            prg_to_load = args[i].clone();
        }
    }
    
    let mut c64 = c64::C64::new(window_scale, debugger_on, &prg_to_load);
    c64.reset();

    // main update loop
    while c64.main_window.is_open() {
        c64.run();
    }
}
