use utils;
extern crate sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

pub struct SysFont
{
    color: u8,
    texture: sdl2::render::Texture
}


impl SysFont
{
    pub fn new(renderer: &sdl2::render::Renderer) -> SysFont
    {
        let mut font = SysFont
        {
            color: 0,
            texture: renderer.create_texture_static(PixelFormatEnum::RGB24, (256, 64)).unwrap()
                
        };

        let font_data = utils::open_file("res/font.bmp", 54);
        let _ = font.texture.update(None, &font_data, 256*3);
        font
    }
    
    pub fn draw(&self, renderer: &mut sdl2::render::Renderer, x: u32, y: u32, text: &str )
    {
        renderer.copy_ex(&self.texture, None, Some(Rect::new_unwrap(0, 0, 256, 64)), 0.0, None, (false, true));
    }
}
