//extern crate sdl2;
extern crate minifb;
use std::env;
//use sdl2::keyboard::Keycode;

#[macro_use]
mod utils;

mod c64;
mod debugger;

use minifb::*;

fn main()
{
    let args: Vec<String>  = env::args().collect();
    
    //let sdl_context = sdl2::init().unwrap();
    /*let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Rust64", SCREEN_WIDTH, SCREEN_HEIGHT)
        .build()
        .unwrap();
    
    let mut renderer = window.renderer().accelerated().present_vsync().build().unwrap();
     */
    
    //let mut event_pump = sdl_context.event_pump().unwrap();

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
        //renderer.copy(&texture, None, Some(Rect::new_unwrap(0, 0, 256, 64)));
        //renderer.copy_ex(&texture, None, Some(Rect::new_unwrap(450, 100, 256, 256)), 30.0, None, (false, false));
        //renderer.present();
        
        /* for event in event_pump.poll_iter()
        {
            use sdl2::event::Event;
            
            match event
            {
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    let _ = (); //texture.update(None, &buff2, 768);
                },                
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        running = false;
                },               
                Event::Quit { .. } => {
                    running = false;
                },
                _ => ()
            }
        } */
        
        c64.run();
    }
}
