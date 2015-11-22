// VIC-II
#![allow(dead_code)]
extern crate sdl2;
use c64::memory;
use std::cell::RefCell;
use std::rc::Rc;
use video;

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

pub struct VIC
{
    mem_ref: memory::MemShared,
    font: video::font::SysFont,

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
    is_bad_line: bool,
    ml_idx: u16,         // matrix/color line index
    mx: Vec<u16>,       // special register: x position of sprites

    matrix_base: u16,
    char_base: u16,
    bitmap_base: u16,
    sprite_y_exp: u8,   // sprite y expansion flipflops

}

impl VIC
{
    pub fn new_shared(memory_ref: memory::MemShared, renderer: &sdl2::render::Renderer) -> VICShared
    {
        Rc::new(RefCell::new(VIC
        {
            mem_ref: memory_ref,
            font: video::font::SysFont::new(renderer),
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
            is_bad_line: false,
            ml_idx: 0,
            mx: vec![0; 8],
            matrix_base: 0,
            char_base: 0,
            bitmap_base: 0,
            sprite_y_exp: 0
        }))
    }

    pub fn read_register(&self, addr: u16) -> u8
    {
        match addr
        {
            0xD011 => {
                let curr_val = self.mem_ref.borrow_mut().read_byte(addr);
                // bit 7 in $d011 is bit 8 of $d012
                (curr_val & 0x7F) | ((self.raster_cnt & 0x100) >> 1) as u8
            },
            0xD012          => self.raster_cnt as u8,
            0xD040...0xD3FF => self.read_register(0xD000 + (addr % 0x0040)),
            _               => self.mem_ref.borrow_mut().read_byte(addr)
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) -> bool
    {
        match addr
        {
            0xD000...0xD00E =>
            {
                if (addr % 2) == 0
                {
                    let idx = ((addr % 0x000F) >> 1) as usize;
                    self.mx[idx] = (self.mx[idx] & 0xFF00) | value as u16;
                    self.mem_ref.borrow_mut().write_byte(addr, self.mx[idx] as u8)
                }
                else
                {
                    self.mem_ref.borrow_mut().write_byte(addr, value)
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
                
                self.mem_ref.borrow_mut().write_byte(addr, value)
            },
            0xD011 =>
            {
                self.y_scroll = (value & 7) as u16;

                let new_raster_irq = (self.raster_irq & 0xFF) | ((0x80 & value as u16) << 1);
                if self.raster_irq != new_raster_irq && self.raster_cnt == new_raster_irq
                {
                    self.raster_irq();
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
                
                self.mem_ref.borrow_mut().write_byte(addr, value)
            },
            0xD012 =>
            {
                let new_raster_irq = (self.raster_irq & 0xFF00) | value as u16;

                if (self.raster_irq != new_raster_irq) && (self.raster_cnt == new_raster_irq)
                {
                    self.raster_irq();
                }

                self.raster_irq = new_raster_irq;

                // TODO: is this correct?
                 self.mem_ref.borrow_mut().write_byte(addr, value)
            },
            0xD016 =>
            {
                let ctrl1 = self.read_register(0xD011);
                self.x_scroll = (value & 7) as u16;
                self.display_mode = (((ctrl1 & 0x60) | (value & 0x10)) >> 4) as u16;
                
                self.mem_ref.borrow_mut().write_byte(addr, value)
            },
            0xD017 =>
            {
                self.sprite_y_exp |= !value; // TODO: check "!"
                self.mem_ref.borrow_mut().write_byte(addr, value)
            },
            0xD018 =>
            {
                self.matrix_base = ((value & 0xF0) as u16) << 6;
                self.char_base   = ((value & 0x0E) as u16) << 10;
                self.bitmap_base = ((value & 0x08) as u16) << 10;
                
                self.mem_ref.borrow_mut().write_byte(addr, value)
            },
            0xD019 =>
            {
                let curr_irq_flag = self.read_register(addr);
                let irq_mask = self.read_register(0xD01A);
                let mut new_irq_flag = curr_irq_flag & (!value & 0x0F);

                if (new_irq_flag & irq_mask) != 0
                {
                    new_irq_flag |= 0x80;
                }
                else
                {
                    // TODO: clear vic irq
                }

                // TODO: is this correct?
                self.mem_ref.borrow_mut().write_byte(addr, new_irq_flag)
            },
            0xD01A =>
            {
                let new_irq_mask = value & 0x0F;
                let mut irq_flag = self.read_register(0xD019);

                if (irq_flag & new_irq_mask) > 0
                {
                   irq_flag |= 0x80
                    // TODO: trigger vic irq
                }
                else
                {
                    irq_flag &= 0x7F;
                    // TODO: clear vic irq
                }
                
                
                // TODO: is this correct?
                self.mem_ref.borrow_mut().write_byte(addr, new_irq_mask)                
            },
            0xD040...0xD3FF => self.write_register(0xD000 + (addr % 0x0040), value),
            _ => self.mem_ref.borrow_mut().write_byte(addr, value)
        }
    }
    
    pub fn update(&self) -> bool
    {
        // TODO main VIC loop
        true
    }

    pub fn draw_background(&self)
    {
        // TODO
    }
    
    pub fn draw_graphics(&self)
    {
        // TODO
    }

    pub fn draw_sprites(&self)
    {
        // TODO
    }
    
    pub fn trigger_lp_irq(&self)
    {
        // TODO: trigger lightpen irq
    }

    pub fn on_va_change(&self, new_va: u16)
    {
        // TODO cia eveny
    }

    pub fn raster_irq(&self)
    {
        // TODO
    }

    // debug
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
