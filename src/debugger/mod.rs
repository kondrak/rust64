// memory debug window
extern crate minifb;
use minifb::*;
use std::io::Write;
use c64;
use utils;
mod font;

const DEBUG_W: usize = 640;
const DEBUG_H: usize = 416;


pub struct Debugger
{
    debug_window: minifb::Window,
    font: font::SysFont,
    window_buffer: Vec<u32>,
    mempage_offset: u32,
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
        if self.debug_window.is_key_pressed(Key::PageUp, KeyRepeat::Yes)
        {
            self.mempage_offset += 0x400;
            if self.mempage_offset > 0xFC00 { self.mempage_offset = 0; }
        }

        if self.debug_window.is_key_pressed(Key::PageDown, KeyRepeat::Yes)
        {
            if self.mempage_offset == 0x0000 { self.mempage_offset = 0x10000; }
            self.mempage_offset -= 0x400;
        }

        self.draw_border();
        
        // dump memory page to screen
        let mut start = 0x0000 + self.mempage_offset as u16;

        let mut title = Vec::new();
        let _ = write!(&mut title, "Memory page ${:04x}-${:04x}", start, start + 0x3FF);
        self.font.draw_text(&mut self.window_buffer, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        let mut hex_offset_x = 0;

        for y in 0..25
        {
            for x in 0..40
            {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::RAM).read(start);
                self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 27 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;
            }

            hex_offset_x = 0;
        }

        self.draw_gfx_mode(memory);
        self.draw_data(memory);
        
        self.debug_window.update(&self.window_buffer);
    }

    fn draw_hex(&mut self, x_pos: usize, y_pos: usize, byte: u8 )
    {
        let mut hex_value = Vec::new();
        let _ = write!(&mut hex_value, "{:02X}", byte);
        
        let mut base_color = utils::fetch_c64_color_rgba(byte >> 4);
        if base_color == 0 { base_color = 0x00333333; }
        //self.set_saturation(&mut base_color, (byte >> 4) as f64 / 15.0);
        
        // all black? make it at least somewhat visible
        if byte == 0 { base_color = 0x00101010; }
        
        self.font.draw_text_rgb(&mut self.window_buffer, x_pos, y_pos, &String::from_utf8(hex_value).unwrap().to_owned()[..], base_color);        
    }

    fn draw_data(&mut self, memory: &mut c64::memory::MemShared)
    {
        let d018 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(0xD018);
        let dd00 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(0xDD00);
        
        let mut vmatrix_txt = Vec::new();
        let mut char_txt = Vec::new();
        let mut bmp_txt = Vec::new();
        let mut bank_txt = Vec::new();
        let _ = write!(&mut vmatrix_txt, "${:04X}", (((d018 >> 4) & 0xF) as u16) * 0x400);
        let _ = write!(&mut char_txt, "${:04X}", (((d018 >> 1) & 0x07) as u16) * 0x800);
        let _ = write!(&mut bmp_txt, "${:04X}", (((d018 >> 3) & 0x01) as u16) * 0x2000);
        let _ = write!(&mut bank_txt, "${:04X}", dd00);
        self.font.draw_text(&mut self.window_buffer, 43, 3, "Screen: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 3, &String::from_utf8(vmatrix_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 45, 4, "Char: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 4, &String::from_utf8(char_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 43, 5, "Bitmap: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 5, &String::from_utf8(bmp_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 41, 6, "VIC Bank: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 6, &String::from_utf8(bank_txt).unwrap().to_owned()[..], 0x0E);
    }

    fn draw_gfx_mode(&mut self, memory: &mut c64::memory::MemShared)
    {
        self.font.draw_text(&mut self.window_buffer, 52, 1, "ECM", 0x0B);
        self.font.draw_text(&mut self.window_buffer, 57, 1, "CHR", 0x0B);
        self.font.draw_text(&mut self.window_buffer, 62, 1, "BMP", 0x0B);
        self.font.draw_text(&mut self.window_buffer, 67, 1, "MCM", 0x0B);
    }

    fn draw_border(&mut self)
    {
        for x in 0..80
        {
            self.font.draw_char(&mut self.window_buffer, 8*x as usize, 0, 64, 0x0B);
            self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8*26, 64, 0x0B);
        }
        
        for y in 1..26
        {
            self.font.draw_char(&mut self.window_buffer, 8*40, 8*y as usize, 66, 0x0B);
        }
        
        self.font.draw_char(&mut self.window_buffer, 8*40, 0, 114, 0x0B);
        self.font.draw_char(&mut self.window_buffer, 8*40, 8*26, 113, 0x0B);
    }
    
   /* fn set_saturation(&self, color: &mut u32, change: f64)
    {
        let Pr = 0.299;
        let Pg = 0.587;
        let Pb = 0.114;
        let mut r = ((*color >> 16) & 0xFF) as f64;
        let mut g = ((*color >> 8) & 0xFF) as f64;
        let mut b = (*color & 0xFF) as f64;

        let P = ((r*r*Pr + g*g*Pg + b*b*Pb) as f64).sqrt();

        r = P + (r - P) * change;
        g = P + (g - P) * change;
        b = P + (b - P) * change;

        *color = (r as u32) << 16;
        *color |= (g as u32) << 8;
        *color |= b as u32;
    }*/
}
