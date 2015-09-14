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
    
    pub fn draw_char(&self, renderer: &mut sdl2::render::Renderer, x: i32, y: i32, charcode: u8 )
    {
        let char_w: u32 = 8;
        let char_h: u32 = 8;
        let tex_h: i32 = 64;
        let char_x = (charcode % 32) as i32;
        let char_y = (charcode / 32) as i32;

        renderer.copy_ex(&self.texture, Some(Rect::new_unwrap(char_x * char_w as i32, tex_h - char_h as i32 - char_y * char_h as i32, char_w, char_h)), Some(Rect::new_unwrap(x * char_w as i32, y * char_h as i32, char_w, char_h)), 0.0, None, (false, true));
    }
}
