mod cpu;
mod utils;
mod memory;
mod video;
extern crate sdl2;
use sdl2::keyboard::Keycode;


const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 200;


fn main()
{
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Rust64", SCREEN_WIDTH, SCREEN_HEIGHT)
        .resizable()
        .opengl()
        .build()
        .unwrap();
    
    let mut running = true;
    let mut renderer = window.renderer().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut cpu = cpu::CPU::new(&renderer);
    cpu.reset();
    
    while running
    {
        renderer.clear();
        //renderer.copy(&texture, None, Some(Rect::new_unwrap(0, 0, 256, 64)));
        //renderer.copy_ex(&texture, None, Some(Rect::new_unwrap(450, 100, 256, 256)), 30.0, None, (false, false));
        //renderer.present();
        
        for event in event_pump.poll_iter()
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
        }

        cpu.update();
        cpu.render(&mut renderer);
        renderer.present();
    }
}
