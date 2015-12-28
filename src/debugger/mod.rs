// memory debug window
extern crate minifb;
use minifb::*;
use std::io::Write;
use c64;
mod font;

const DEBUG_W: usize = 640;
const DEBUG_H: usize = 416;


pub struct Debugger
{
    debug_window: minifb::Window,
    font: font::SysFont,
    window_buffer: Vec<u32>,
    mempage_offset: u16,
}

impl Debugger
{
    pub fn new() -> Debugger
    {
        Debugger {
            debug_window: Window::new("Debug window", DEBUG_W, DEBUG_H, Scale::X2, Vsync::No).unwrap(),
            font: font::SysFont::new(DEBUG_W, DEBUG_H),
            window_buffer: vec![0; DEBUG_W * DEBUG_H],
            mempage_offset: 0,
        }
    }

    pub fn render(&mut self, memory: &mut c64::memory::MemShared)
    {
        if self.debug_window.is_key_pressed(Key::F9, KeyRepeat::No)
        {
            self.mempage_offset += 0x400;
            if self.mempage_offset > 0x3C00 { self.mempage_offset = 0; }
        }
        // dump screen memory
        let mut start = 0x0000 + self.mempage_offset;

        let mut title = Vec::new();
        let _ = write!(&mut title, "Page ${:04x}-${:04x}", start, start + 0x400);
        self.font.draw_text(&mut self.window_buffer, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        let mut hex_offset_x = 0;
        let mut hex_offset_y = 27;
        for y in 0..25
        {
            for x in 0..40
            {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::RAM).read(start);
                self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                let mut hex_value = Vec::new();
                let _ = write!(&mut hex_value, "{:02X}", byte);

                //let mut hex_color: u32 = (2*byte as u32) << 16;
                //hex_color |= (byte as u32) << 8;
                //hex_color |= byte as u32;
                //if hex_color < 0x00222222 { hex_color = 0x00222222; }
                
                self.font.draw_text(&mut self.window_buffer, hex_offset_x + x as usize, hex_offset_y + y as usize, &String::from_utf8(hex_value).unwrap().to_owned()[..], byte);
                hex_offset_x += 1;
                start += 1;
            }

            hex_offset_x = 0;
        }
        
        self.debug_window.update(&self.window_buffer);
    }
}
