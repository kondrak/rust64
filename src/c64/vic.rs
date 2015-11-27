// VIC-II
#![allow(dead_code)]
extern crate sdl2;
use c64::memory;
use c64::cpu;
use std::cell::RefCell;
use std::rc::Rc;
use c64;
use utils;
//use video;

pub type VICShared = Rc<RefCell<VIC>>;

// number of rasterlines for PAL (0x138)
static NUM_RASTERLINES: u16 = 312;

// first and last possible lines for bad lines
static FIRST_DMA_LINE: u16 = 0x30;
static LAST_DMA_LINE: u16  = 0xF7;

static ROW25_YSTART: u16 = 0x33;
static ROW25_YSTOP:  u16 = 0xFB;
static ROW24_YSTART: u16 = 0x37;
static ROW24_YSTOP:  u16 = 0xF7;

// action to perform on specific VIC events (write, raster irq...)
pub enum VICCallbackAction
{
    None,
    TriggerVICIrq,
    ClearVICIrq,
}

pub struct VIC
{
    mem_ref: Option<memory::MemShared>,
    cpu_ref: Option<cpu::CPUShared>,
    //font: video::font::SysFont,

    pub window_buffer: [u32; c64::SCREEN_WIDTH * c64::SCREEN_HEIGHT],
    
    irq_flag: u8,
    irq_mask: u8,

    // background color palette indices
    ec_color: u8,
    b0c_color: u8,
    b1c_color: u8,
    b2c_color: u8,
    b3c_color: u8,
    
    matrix_line: [u8; 40], // video line buffer, read in bad lines
    color_line: [u8; 40],  // color line buffer, read in bad lines
    
    last_byte: u8,       // last byte read by VIC
    raster_x: u16,       // raster x position
    raster_cnt: u16,     // raster line counter (current raster line)
    raster_irq: u16,     // raster interrupt line
    dy_start: u16,       // border logic helper values
    dy_stop: u16,
    row_cnt: u16,        // row counter
    video_cnt: u16,      // video counter
    video_cnt_base: u16, // video counter base
    x_scroll: u16,
    y_scroll: u16,
    cia_vabase: u16,

    curr_cycle: u16,     // current cycle
    display_mode: u16,   // current display mode
    bad_lines_on: bool,
    lp_triggered: bool,  // lightpen irq triggered
    display_state: bool, // true: display state; false: idle state
    is_bad_line: bool,
    draw_this_line: bool,
    ml_idx: usize,         // matrix/color line index
    mx: Vec<u16>,       // special register: x position of sprites

    matrix_base: u16,
    char_base: u16,
    bitmap_base: u16,
    sprite_y_exp: u8,   // sprite y expansion flipflops
    gfx_data: u8,
    char_data: u8,
    color_data: u8,
    last_char_data: u8,
    first_ba_cycle: u32,
}

impl VIC
{    
    //pub fn new_shared(renderer: &sdl2::render::Renderer) -> VICShared
    pub fn new_shared() -> VICShared
    {
        Rc::new(RefCell::new(VIC
        {
            mem_ref: None,
            cpu_ref: None,
            window_buffer: [0; c64::SCREEN_WIDTH * c64::SCREEN_HEIGHT],
            //font: video::font::SysFont::new(renderer),
            irq_flag: 0,
            irq_mask: 0,
            ec_color: 0,
            b0c_color: 0,
            b1c_color: 0,
            b2c_color: 0,
            b3c_color: 0,
            matrix_line: [0; 40],
            color_line: [0; 40],
            last_byte: 0,
            raster_x: 0,
            raster_cnt: NUM_RASTERLINES - 1,
            row_cnt: 7,
            raster_irq: 0,
            dy_start: 0,
            dy_stop: 0,
            video_cnt: 0,
            video_cnt_base: 0,
            x_scroll: 0,
            y_scroll: 0,
            cia_vabase: 0,
            curr_cycle: 1,
            display_mode: 0,
            bad_lines_on: false,
            lp_triggered: false,
            display_state: false,
            is_bad_line: false,
            draw_this_line: false,
            ml_idx: 0,
            mx: vec![0; 8],
            matrix_base: 0,
            char_base: 0,
            bitmap_base: 0,
            sprite_y_exp: 0,
            gfx_data: 0,
            char_data: 0,
            color_data: 0,
            last_char_data: 0,
            first_ba_cycle: 0,
        }))
    }

    // debug
    pub fn render(&mut self)
    {
        // dump screen memory
        let mut start = 0x0400;

        for y in 0..25
        {
            for x in 0..40
            {
                let d = as_mut!(self.mem_ref).read_byte(start);
                //self.font.draw_char(renderer, x, y, d);
                //let muti = self.window_buffer[0];
                self.window_buffer[x + y * c64::SCREEN_WIDTH] = if d != 32 { 0x00FFFFFF } else { 0x000088FF };
                start += 1;
            }
        }
    }
    
    pub fn set_references(&mut self, memref: memory::MemShared, cpuref: cpu::CPUShared)
    {
        self.mem_ref = Some(memref);
        self.cpu_ref = Some(cpuref);
    }

    // TODO: prepare for more palettes?
    pub fn fetch_c64_color(&self, idx: u8) -> u8
    {
        idx & 0x0F
    }
    
    pub fn read_register(&self, addr: u16) -> u8
    {
        match addr
        {
            0xD011 => {
                let curr_val = as_ref!(self.mem_ref).read_byte(addr);
                // bit 7 in $d011 is bit 8 of $d012
                (curr_val & 0x7F) | ((self.raster_cnt & 0x100) >> 1) as u8
            },
            0xD012          => self.raster_cnt as u8,
            0xD019          => self.irq_flag | 0x70,
            0xD01A          => self.irq_mask | 0xF0,
            0xD040...0xD3FF => self.read_register(0xD000 + (addr % 0x0040)),
            _               => as_ref!(self.mem_ref).read_byte(addr)
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8, on_vic_write: &mut VICCallbackAction) -> bool
    {
        match addr
        {
            0xD000...0xD00E =>
            {
                if (addr % 2) == 0
                {
                    let idx = ((addr % 0x000F) >> 1) as usize;
                    self.mx[idx] = (self.mx[idx] & 0xFF00) | value as u16;
                    as_mut!(self.mem_ref).write_byte(addr, self.mx[idx] as u8)
                }
                else
                {
                    as_mut!(self.mem_ref).write_byte(addr, value)
                }
            },
            0xD010 =>
            {
                let mut j = 1;
                
                for i in 0..8
                {
                    if (value & j) != 0
                    {
                        self.mx[i] |= 0x100;
                    }
                    else
                    {
                        self.mx[i] &= 0x00FF;
                    }

                    j <<= 1;
                }
                
                as_mut!(self.mem_ref).write_byte(addr, value)
            },
            0xD011 =>
            {
                self.y_scroll = (value & 7) as u16;

                let new_raster_irq = (self.raster_irq & 0xFF) | ((0x80 & value as u16) << 1);
                if self.raster_irq != new_raster_irq && self.raster_cnt == new_raster_irq
                {
                    *on_vic_write = self.raster_irq();
                }

                self.raster_irq = new_raster_irq;

                if (value & 8) != 0
                {
                    self.dy_start = ROW25_YSTART;
                    self.dy_stop = ROW25_YSTOP;
                }
                else
                {
                    self.dy_start = ROW24_YSTART;
                    self.dy_stop = ROW24_YSTOP;
                }

                if (self.raster_cnt == 0x30) && ((value & 0x10) != 0)
                {
                    self.bad_lines_on = true;
                }

                self.is_bad_line = self.raster_cnt >= FIRST_DMA_LINE &&
                                    self.raster_cnt <= LAST_DMA_LINE &&
                                    ((self.raster_cnt & 7) == self.y_scroll) && self.bad_lines_on;
                let ctrl2 = self.read_register(0xD016);
                self.display_mode = (((value & 0x60) | (ctrl2 & 0x10)) >> 4) as u16;
                
                as_mut!(self.mem_ref).write_byte(addr, value)
            },
            0xD012 =>
            {
                let new_raster_irq = (self.raster_irq & 0xFF00) | value as u16;

                if (self.raster_irq != new_raster_irq) && (self.raster_cnt == new_raster_irq)
                {
                    *on_vic_write = self.raster_irq();
                }

                self.raster_irq = new_raster_irq;

                // TODO: is this correct?
                 as_mut!(self.mem_ref).write_byte(addr, value)
            },
            0xD016 =>
            {
                let ctrl1 = self.read_register(0xD011);
                self.x_scroll = (value & 7) as u16;
                self.display_mode = (((ctrl1 & 0x60) | (value & 0x10)) >> 4) as u16;
                
                as_mut!(self.mem_ref).write_byte(addr, value)
            },
            0xD017 =>
            {
                self.sprite_y_exp |= !value; // TODO: check "!"
                as_mut!(self.mem_ref).write_byte(addr, value)
            },
            0xD018 =>
            {
                self.matrix_base = ((value & 0xF0) as u16) << 6;
                self.char_base   = ((value & 0x0E) as u16) << 10;
                self.bitmap_base = ((value & 0x08) as u16) << 10;
                
                as_mut!(self.mem_ref).write_byte(addr, value)
            },
            0xD019 =>
            {
                self.irq_flag = self.irq_flag & (!value & 0x0F);
                
                if (self.irq_flag & self.irq_mask) != 0
                {
                    self.irq_flag |= 0x80;
                }
                else
                {
                    // normally we'd dereference the cpu directly but in Rust
                    // it's not possible due to RefCell already being borrowed (call by CPU)
                    *on_vic_write = VICCallbackAction::TriggerVICIrq;
                }
                true
            },
            0xD01A =>
            {
                self.irq_mask = value & 0x0F;

                if (self.irq_flag & self.irq_mask) != 0
                {
                    self.irq_flag |= 0x80;
                    *on_vic_write = VICCallbackAction::TriggerVICIrq;
                }
                else
                {
                    self.irq_flag &= 0x7F;
                    *on_vic_write = VICCallbackAction::ClearVICIrq;
                }
                true
            },
            0xD040...0xD3FF => self.write_register(0xD000 + (addr % 0x0040), value, on_vic_write),
            _ => as_mut!(self.mem_ref).write_byte(addr, value)
        }
    }
    
    pub fn update(&mut self) -> bool
    {
        // TODO main VIC loop
        true
    }

    pub fn trigger_lp_irq(&mut self)
    {
        // lightpen triggers only once per frame
        if !self.lp_triggered
        {
            self.lp_triggered = true;
            
            let lpx = self.raster_x >> 1;
            let lpy = self.raster_cnt;
            
            self.irq_flag |= 0x08;
            if (self.irq_mask & 0x08) != 0
            {
                self.irq_flag |= 0x80;
                as_mut!(self.cpu_ref).trigger_vic_irq();
            }

            let mut vicwrite: VICCallbackAction = VICCallbackAction::None;
            self.write_register(0xD013, lpx as u8, &mut vicwrite);
            self.write_register(0xD014, lpy as u8, &mut vicwrite);
        }
    }

    pub fn on_va_change(&mut self, new_va: u16)
    {
        self.cia_vabase = new_va << 14;
        let vbase = self.read_register(0xD018);
        let mut vicwrite: VICCallbackAction = VICCallbackAction::None;
        self.write_register(0xD018, vbase, &mut vicwrite);
    }

    pub fn raster_irq(&mut self) -> VICCallbackAction
    {
        self.irq_flag |= 0x01;
 
        if (self.irq_mask & 0x01) != 0
        {
            self.irq_flag |= 0x80;

            // TODO: when the time is right check if this works correctly (irq should be triggered here)
            //as_mut!(self.cpu_ref).trigger_vic_irq();
            VICCallbackAction::TriggerVICIrq
        }
        else
        {
            VICCallbackAction::None
        }
    }

    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        let va = addr | self.cia_vabase;

        if (va & 0x7000) == 0x1000
        {
            self.last_byte = as_mut!(self.mem_ref).get_rom_bank(memory::MemType::CHARGEN).read(va & 0x0FFF);
        }
        else
        {
            self.last_byte = as_mut!(self.mem_ref).get_ram_bank(memory::MemType::RAM).read(va);
        }

        self.last_byte
    }

    pub fn matrix_access(&mut self, c64_cycle_cnt: u32)
    {
        if as_ref!(self.cpu_ref).ba_low
        {
            if (c64_cycle_cnt - self.first_ba_cycle) < 3
            {
                self.color_line[self.ml_idx]  = 0xFF;
                self.matrix_line[self.ml_idx] = 0xFF;
            }
            else
            {
                let addr = (self.video_cnt & 0x03FF) | self.matrix_base;
                self.matrix_line[self.ml_idx] = self.read_byte(addr);

                // assign value from color ram
                // TODO: is this done correctly?
                self.color_line[self.ml_idx] = as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).read(0xD800 + (addr & 0x03FF));
            }
        }
    }

    pub fn graphics_access(&mut self)
    {
        let ctrl1 = self.read_register(0xD011);
        
        if self.display_state
        {
            let mut addr: u16;

            if (ctrl1 & 0x20) != 0 // bitmap
            {
                addr = ((self.video_cnt & 0x03FF) << 3) as u16 | self.bitmap_base | self.row_cnt;
            }
            else // text
            {
                addr = (self.matrix_line[self.ml_idx] << 3) as u16 | self.char_base | self.row_cnt;
            }

            if (ctrl1 & 0x40) != 0 // ECM
            {
                addr &= 0xF9FF;
            }

            self.gfx_data = self.read_byte(addr);
            self.char_data = self.matrix_line[self.ml_idx];
            self.color_data = self.color_line[self.ml_idx];

            self.ml_idx += 1;
            self.video_cnt += 1;
        }
        else
        {
            // display is off
            self.gfx_data = self.read_byte(if (ctrl1 & 0x40) != 0 { 0x39FF } else { 0x3FFF });
            self.char_data = 0;
            self.color_data = 0;
        }
    }

    pub fn draw_background(&mut self)
    {
        let mut dst_color: u8;

        if !self.draw_this_line { return }

        match self.display_mode
        {
            // standard text, multicolor text, multicolor bitmap
            0 | 1 | 3 => {
                dst_color = self.b0c_color;
            },
            // standard bitmap
            2 => {
                dst_color = self.fetch_c64_color(self.last_char_data);
            },
            // ECM text
            4 => {
                if (self.last_char_data & 0x80) != 0
                {
                    if (self.last_char_data & 0x40) != 0
                    {
                        dst_color = self.b3c_color;
                    }
                    else
                    {
                        dst_color = self.b2c_color;
                    }
                }
                else
                {
                    if (self.last_char_data & 0x40) != 0
                    {
                        dst_color = self.b1c_color;
                    }
                    else
                    {
                        dst_color = self.b0c_color;
                    }
                }
            },
            _ => dst_color = self.fetch_c64_color(0),
        }

        //utils::memset8(&mut self.screen, dst_color);
    }
    
    pub fn draw_graphics(&self)
    {
        // TODO
    }

    pub fn draw_sprites(&self)
    {
        // TODO
    }
        
}
