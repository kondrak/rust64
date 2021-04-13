// memory debug window
extern crate minifb;

mod font;

use c64;
use minifb::*;
use std::io::Write;
use utils;

const DEBUG_W: usize = 640;
const DEBUG_H: usize = 432;
const RASTER_DEBUG_W: usize = 3*63;
const RASTER_DEBUG_H: usize = 312;

// color constants for VIC raster window
const BORDER_COLOR: u32    = 0x00404040;
const BG_COLOR: u32        = 0x00000000;
const VIC_WRITE_COLOR: u32 = 0x00FF0000;
const RASTER_COLOR: u32    = 0x000000FF;
const BADLINE_COLOR: u32   = 0x0000FF00;


pub struct Debugger {
    debug_window: minifb::Window,
    vic_window:   minifb::Window,
    font: font::SysFont,
    window_buffer: Vec<u32>, // main debugger window data buffer
    vic_buffer: Vec<u32>,    // VIC window data buffer
    mempage_offset: u32,     // RAM preview memory page offset
    draw_mode: u8,
}

impl Debugger {
    pub fn new() -> Debugger {
        let mut dbg = Debugger {
            debug_window: Window::new("Debug window", DEBUG_W, DEBUG_H, WindowOptions { scale: Scale::X2, ..Default::default() }).unwrap(),
            vic_window: Window::new("VIC", RASTER_DEBUG_W, RASTER_DEBUG_H, WindowOptions::default()).unwrap(),
            font: font::SysFont::new(),
            window_buffer: vec![0; DEBUG_W * DEBUG_H],
            vic_buffer: vec![0; RASTER_DEBUG_W * RASTER_DEBUG_H],
            mempage_offset: 0,
            draw_mode: 0,
        };

        dbg.debug_window.set_position(480, 20);
        dbg.vic_window.set_position(270, 340);

        for y in 1..26 {
            for x in 0..40 {
                dbg.font.draw_char_rgb(&mut dbg.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, 102, 0x00101010);
            }
        }

        dbg.draw_vic_window_txt();        
        dbg
    }


    pub fn render(&mut self, cpu: &mut c64::cpu::CPUShared, memory: &mut c64::memory::MemShared) {
        if self.debug_window.is_open() {
            self.draw_border();

            let home_pressed = self.debug_window.is_key_pressed(Key::Home, KeyRepeat::No);
            let end_pressed  = self.debug_window.is_key_pressed(Key::End, KeyRepeat::No);

            if home_pressed || end_pressed {
                if home_pressed {
                    if self.draw_mode == 0 {
                        self.draw_mode = 4;
                    }
                    else {
                        self.draw_mode -= 1;
                    }
                }
                if end_pressed {
                    if self.draw_mode == 4 {
                        self.draw_mode = 0;
                    }
                    else {
                        self.draw_mode += 1;
                    }
                }
                // clear memdump
                for y in 0..26 {
                    for x in 0..40 {
                        self.font.draw_char_rgb(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, 102, 0x00101010);
                    }
                }
                // clear hex region
                for y in 28..54 {
                    for x in 0..80 {
                        self.clear_char(x, y);
                    }
                }
            }

            match self.draw_mode {
                0 => self.draw_ram(memory),
                1 => self.draw_vic(memory),
                2 => self.draw_cia(cpu),
                3 => self.draw_color_ram(memory),
                4 => self.draw_sid(memory),
                _ => ()
            }

            self.draw_gfx_mode(memory);
            self.draw_latch_status(memory);
            self.draw_data(memory);
            self.draw_cpu(cpu);

            let _ = self.debug_window.update_with_buffer(&self.window_buffer,DEBUG_W, DEBUG_H);
        }
        if self.vic_window.is_open() {
            let _ = self.vic_window.update_with_buffer(&self.vic_buffer,DEBUG_W, DEBUG_H);
        }
    }

    
    pub fn update_vic_window(&mut self, vic: &mut c64::vic::VICShared) {
        if !self.vic_window.is_open() {
            return;
        }

        let x = vic.borrow_mut().curr_cycle;
        let y = vic.borrow_mut().raster_cnt;
        let is_bad_line = vic.borrow_mut().is_bad_line;
        let is_raster_irq = vic.borrow_mut().raster_irq == y;
        let is_border = vic.borrow_mut().border_on;
        let is_state_changed = vic.borrow_mut().dbg_reg_changed;
        
        let mut dst_color = if is_border { BORDER_COLOR } else { BG_COLOR };
        dst_color = if is_state_changed { self.mix_colors(VIC_WRITE_COLOR, dst_color, 0.8) } else { dst_color };
        dst_color = if is_raster_irq { self.mix_colors(RASTER_COLOR, dst_color, 0.8) } else { dst_color };
        dst_color = if is_bad_line { self.mix_colors(BADLINE_COLOR, dst_color, 0.5) } else { dst_color };

        self.vic_buffer[((x as u32) - 1 + (RASTER_DEBUG_W as u32)*(y as u32)) as usize] = dst_color;
    }


    // *** private functions *** //

    // dump RAM page to screen
    fn draw_ram(&mut self, memory: &mut c64::memory::MemShared) {
        if self.debug_window.is_key_pressed(Key::PageUp, KeyRepeat::Yes) {
            self.mempage_offset += 0x400;

            if self.mempage_offset > 0xFC00 {
                self.mempage_offset = 0;
            }
        }
        if self.debug_window.is_key_pressed(Key::PageDown, KeyRepeat::Yes) {
            if self.mempage_offset == 0x0000 {
                self.mempage_offset = 0x10000;
            }
            self.mempage_offset -= 0x400;
        }

        let mut start = 0x0000 + self.mempage_offset as u16;
        let mut title = Vec::new();
        let mut hex_offset_x = 0;
        let _ = write!(&mut title, "Memory page ${:04x}-${:04x}", start, start + 0x3FF);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 34, 0, "*RAM*", 0x0E);

        for y in 0..26 {
            for x in 0..40 {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Ram).read(start);
                self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 28 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == (self.mempage_offset as u16 + 0x0400) { return; }
            }
            hex_offset_x = 0;
        }
    }


    // VIC registers
    fn draw_vic(&mut self, memory: &mut c64::memory::MemShared) {
        let mut start = 0xD000;
        let mut title = Vec::new();
        let mut hex_offset_x = 0;
        let _ = write!(&mut title, "VIC ${:04x}-${:04x}", start, start + 0x03F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 34, 0, "*VIC*", 0x0E);

        for y in 0..25 {
            for x in 0..40 {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Io).read(start);
                self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, byte, 0x05);
                self.draw_hex(hex_offset_x + x as usize, 28 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == 0xD040 {
                    return;
                }
            }
            hex_offset_x = 0;
        }
    }


    // CIA registers
    fn draw_cia(&mut self, cpu: &mut c64::cpu::CPUShared) {
        let mut start = 0xDC00;
        let mut title = Vec::new();
        let mut hex_offset_x = 0;
        let _ = write!(&mut title, "CIA ${:04x}-${:04x}", start, start + 0x1FF);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 34, 0, "*CIA*", 0x0E);

        for y in 0..25 {
            for x in 0..40 {
                if start >= 0xDC10 && start < 0xDD00 {
                    hex_offset_x += 1;
                    start += 1;
                    continue;
                }
                let byte = cpu.borrow_mut().read_byte(start);
                self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, byte, 0x05);
                self.draw_hex(hex_offset_x + x as usize, 28 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == 0xDD10 {
                    return;
                }
            }
            hex_offset_x = 0;
        }
    }


    // SID registers
    fn draw_sid(&mut self, memory: &mut c64::memory::MemShared) {
        let mut start = 0xD400;
        let mut title = Vec::new();
        let mut hex_offset_x = 0;
        let _ = write!(&mut title, "SID ${:04x}-${:04x}", start, start + 0x03F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 34, 0, "*SID*", 0x0E);

        for y in 0..25 {
            for x in 0..40 {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Io).read(start);
                self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 28 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == 0xD420 {
                    return;
                }
            }
            hex_offset_x = 0;
        }
    }


    // Color RAM
    fn draw_color_ram(&mut self, memory: &mut c64::memory::MemShared) {
        let mut start = 0xD800;

        let mut title = Vec::new();
        let _ = write!(&mut title, "COLOR ${:04x}-${:04x}", start, start + 0x3FF);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 28, 0, "*COLOR RAM*", 0x0E);
        
        let mut hex_offset_x = 0;

        for y in 0..25 {
            for x in 0..40 {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Io).read(start);
                self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 28 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == 0xDC00 {
                    return;
                }
            }

            hex_offset_x = 0;
        }
    }


    // draw colored hex value of memory cell
    fn draw_hex(&mut self, x_pos: usize, y_pos: usize, byte: u8 ) {
        let mut hex_value = Vec::new();
        let _ = write!(&mut hex_value, "{:02X}", byte);
        
        let mut base_color = utils::fetch_c64_color_rgba(byte >> 4);
        if base_color == 0 {
            base_color = 0x00333333;
        }
        
        // all black? make it at least somewhat visible
        if byte == 0 {
            base_color = 0x00101010;
        }
        
        self.font.draw_text_rgb(&mut self.window_buffer, DEBUG_W, x_pos, y_pos, &String::from_utf8(hex_value).unwrap().to_owned()[..], base_color);        
    }


    // basic C64 settings
    fn draw_data(&mut self, memory: &mut c64::memory::MemShared) {
        let d018 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Io).read(0xD018);
        let dd00 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Io).read(0xDD00);
        
        let mut vmatrix_txt = Vec::new();
        let mut char_txt = Vec::new();
        let mut bmp_txt = Vec::new();
        let mut bank_txt = Vec::new();
        let _ = write!(&mut vmatrix_txt, "${:04X}", 0x400 * ((d018 >> 4) & 0xF) as u16);
        let _ = write!(&mut char_txt, "${:04X}", 0x800 * ((d018 >> 1) & 0x07) as u16);
        let _ = write!(&mut bmp_txt, "${:04X}", 0x2000 * ((d018 >> 3) & 0x01) as u16);
        let _ = write!(&mut bank_txt, "${:04X}", 0xC000 - 0x4000 * (dd00 & 0x03) as u16);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 43, 3, "Screen: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 51, 3, &String::from_utf8(vmatrix_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 45, 4, "Char: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 51, 4, &String::from_utf8(char_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 43, 5, "Bitmap: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 51, 5, &String::from_utf8(bmp_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 41, 6, "VIC Bank: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 51, 6, &String::from_utf8(bank_txt).unwrap().to_owned()[..], 0x0E);
    }


    // current graphics mode tags
    fn draw_gfx_mode(&mut self, memory: &mut c64::memory::MemShared) {
        let d011 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Io).read(0xD011);
        let d016 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::Io).read(0xD016);
        let ecm_on = (d011 & 0x40) != 0;
        let mcm_on = (d016 & 0x10) != 0;
        let bmp_on = (d011 & 0x20) != 0;
        
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 52, 1, "ECM", if ecm_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 57, 1, "CHR", if !bmp_on & !ecm_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 62, 1, "BMP", if bmp_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 67, 1, "MCM", if mcm_on { 0x0A } else { 0x0B });
    }


    // active memory banks
    fn draw_latch_status(&mut self, memory: &mut c64::memory::MemShared) {
        let basic_on = memory.borrow_mut().basic_on;
        let chargen_on = memory.borrow_mut().chargen_on;
        let io_on = memory.borrow_mut().io_on;
        let kernal_on = memory.borrow_mut().kernal_on;
        
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 48, 25, "BASIC", if basic_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 55, 25, "CHARGEN", if chargen_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 64, 25, "IO", if io_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 68, 25, "KERNAL", if kernal_on { 0x0A } else { 0x0B });
    }    


    // draw CPU flags and registers
    fn draw_cpu(&mut self, cpu: &mut c64::cpu::CPUShared) {
        let mut pc_txt = Vec::new();
        let mut a_txt = Vec::new();
        let mut x_txt = Vec::new();
        let mut y_txt = Vec::new();
        let mut sp_txt = Vec::new();
        let mut p_txt = Vec::new();
        let _ = write!(&mut pc_txt, "${:04X}", cpu.borrow_mut().pc);
        let _ = write!(&mut a_txt, "${:02X}", cpu.borrow_mut().a);
        let _ = write!(&mut x_txt, "${:02X}", cpu.borrow_mut().x);
        let _ = write!(&mut y_txt, "${:02X}", cpu.borrow_mut().y);
        let _ = write!(&mut sp_txt, "${:02X}", cpu.borrow_mut().sp);
        let _ = write!(&mut p_txt, "[{:08b}]", cpu.borrow_mut().p);
        
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 44, 22, "PC:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 47, 22, &String::from_utf8(pc_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 53, 22, "A:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 55, 22, &String::from_utf8(a_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 59, 22, "X:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 61, 22, &String::from_utf8(x_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 65, 22, "Y:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 67, 22, &String::from_utf8(y_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 71, 22, "SP:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 74, 22, &String::from_utf8(sp_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 51, 23, "NV-BDIZC:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 61, 23, &String::from_utf8(p_txt).unwrap().to_owned()[..], 0x0E);
    }


    // draw window border
    fn draw_border(&mut self) {
        for x in 0..80 {
            self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 0, 64, 0x0B);
            self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8*27, 64, 0x0B);
        }
        
        for y in 1..27 {
            self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*40, 8*y as usize, 66, 0x0B);
        }

        self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*40, 0, 114, 0x0B);
        self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*40, 8*27, 113, 0x0B);
    }

    
    // one-time text draw for VIC raster window
    fn draw_vic_window_txt(&mut self) {    
        self.font.draw_text_rgb(&mut self.vic_buffer, RASTER_DEBUG_W, 10, 3, "*VIC events*", utils::fetch_c64_color_rgba(0x0F));
        self.font.draw_text_rgb(&mut self.vic_buffer, RASTER_DEBUG_W, 11, 5, "Border on", BORDER_COLOR);
        self.font.draw_text_rgb(&mut self.vic_buffer, RASTER_DEBUG_W, 11, 6, "Bad line", BADLINE_COLOR);
        self.font.draw_text_rgb(&mut self.vic_buffer, RASTER_DEBUG_W, 11, 7, "Reg. change", VIC_WRITE_COLOR);
        self.font.draw_text_rgb(&mut self.vic_buffer, RASTER_DEBUG_W, 11, 8, "Raster IRQ", RASTER_COLOR);
    }


    fn clear_char(&mut self, x_pos: usize, y_pos: usize) {
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, x_pos, y_pos, " ", 0x00);
    }


    fn mix_colors(&self, new: u32, old: u32, alpha: f32) -> u32 {
        let rn = ((new >> 16) & 0xFF) as f32;
        let gn = ((new >> 8) & 0xFF) as f32;
        let bn = (new & 0xFF) as f32;

        let ro = ((old >> 16) & 0xFF) as f32;
        let go = ((old >> 8) & 0xFF) as f32;
        let bo = (old & 0xFF) as f32;

        let rd = alpha * rn + (1.0 - alpha) * ro;
        let gd = alpha * gn + (1.0 - alpha) * go;
        let bd = alpha * bn + (1.0 - alpha) * bo;

        let mut dst_color = (rd as u32) << 16;
        dst_color |= (gd as u32) << 8;
        dst_color |= bd as u32;

        dst_color
    }
}
