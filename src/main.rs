extern crate minifb;
use std::env;

#[macro_use]
mod utils;

mod c64;
mod debugger;

use minifb::*;

fn main()
{
    let args: Vec<String>  = env::args().collect();

    let mut load_prg = String::new();
    let mut debugger_on = false;
    let mut window_scale = Scale::X1;
    for i in 1..args.len()
    {
        if args[i] == "debugger"
        {
            debugger_on = true;
        }
        else if args[i] == "x2"
        {
            window_scale = Scale::X2;
        }
        else
        {
            load_prg = args[i].clone();
        }
    }
    
    let mut c64 = c64::C64::new(window_scale, debugger_on);
    c64.file_to_load = load_prg;
    c64.reset();

    while c64.window.is_open()
    {
        c64.run();
    }
}
