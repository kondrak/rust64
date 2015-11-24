extern crate sdl2;
extern crate minifb;
//use sdl2::keyboard::Keycode;
use std::num::Wrapping;

#[macro_use]
mod utils;

mod c64;
mod video;

fn main()
{
    //let sdl_context = sdl2::init().unwrap();
    /*let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Rust64", SCREEN_WIDTH, SCREEN_HEIGHT)
        .build()
        .unwrap();
    
    let mut renderer = window.renderer().accelerated().present_vsync().build().unwrap();
     */
    
    let mut running = true;
    //let mut event_pump = sdl_context.event_pump().unwrap();

    let mut c64 = c64::C64::new();
    c64.reset();

    let mut render_cnt: u16 = 0;
    
    if !(minifb::open("Rust64", c64::SCREEN_WIDTH, c64::SCREEN_HEIGHT))
    {
        panic!("Could not create window");
    }
    
    while running
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
        
        c64.update();

        // don't refresh screen every cpu instruction - until VIC is done
        render_cnt = (Wrapping(render_cnt) + Wrapping(1)).0;

        if render_cnt == 0
        { running &= c64.render(); } //renderer.clear(); c64.render(&mut renderer); renderer.present(); }
    }

    minifb::close();
}
