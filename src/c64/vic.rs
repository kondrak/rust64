// VIC-II
#![allow(dead_code)]
extern crate sdl2;
use c64::memory;
use video;

pub struct VIC
{
    mem_ref: memory::MemShared,
    font: video::font::SysFont,
}

impl VIC
{
    pub fn new(memory_ref: memory::MemShared, renderer: &sdl2::render::Renderer) -> VIC
    {
        VIC
        {
            mem_ref: memory_ref,
            font: video::font::SysFont::new(renderer),
        }
    }

    pub fn render(&self, renderer: &mut sdl2::render::Renderer)
    {
        // dump screen memory
        let mut start = 0x0400;

        for y in 0..25
        {
            for x in 0..40
            {
                let d = self.mem_ref.borrow_mut().read_byte(start);
                self.font.draw_char(renderer, x, y, d);
                start += 1;
            }
        }
    }
}
