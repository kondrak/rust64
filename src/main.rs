extern crate sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

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
    let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGB24, (256, 256)).unwrap();
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize|
                      {
                          for y in (0..256)
                          {
                              for x in (0..256)
                              {
                                  let offset = y*pitch + x*3;
                                  buffer[offset + 0] = x as u8;
                                  buffer[offset + 1] = y as u8;
                                  buffer[offset + 2] = 0;
                              }                              
                          }                          
                      }).unwrap();

    
    let mut buff2: [u8; 256*256*3] = [0; 256*256*3];

    let pitch = 768;
                          for y in (0..256)
                          {
                              for x in (0..256)
                              {
                                  let offset = y*pitch + x*3;
                                  buff2[offset + 0] = y as u8;
                                  buff2[offset + 1] = x as u8;
                                  buff2[offset + 2] = 0;
                              }                              
                          }                          
   
    while running
    {
        renderer.clear();
        renderer.copy(&texture, None, Some(Rect::new_unwrap(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)));
        //renderer.copy_ex(&texture, None, Some(Rect::new_unwrap(450, 100, 256, 256)), 30.0, None, (false, false));
        renderer.present();

        for event in event_pump.poll_iter()
        {
            use sdl2::event::Event;
            
            match event
            {
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    texture.update(None, &buff2, 768);
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
        
    }
    
    println!("Here we go.");
}
