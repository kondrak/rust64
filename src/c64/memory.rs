#![allow(dead_code)]
use utils;
//use std::ops::{Index, Deref, DerefMut};
use std::cell::RefCell;
use std::rc::Rc;

pub type MemShared = Rc<RefCell<Memory>>;

pub enum MemType
{
    RAM,
    BASIC,
    CHARGEN,
    IO,
    KERNAL,
}

// specific memory bank - RAM, ROM, IO
struct MemBank
{
    bank_type: MemType, // what am I?
    read_only: bool,    // RAM or ROM?
    offset: u16,        // offset from start of address space
    data: Vec<u8>,
}

impl MemBank
{
    pub fn new(mem_type: MemType) -> MemBank
    {
        let mut mem_bank = MemBank
        {
            bank_type: mem_type,
            read_only: true,
            offset: 0x0000,
            data: Vec::<u8>::new(),
        };

        match mem_bank.bank_type
        {
            MemType::BASIC   => {
                mem_bank.data = utils::open_file("rom/basic.rom", 0);
                mem_bank.offset = 0xA000;
            },
            MemType::CHARGEN => {
                mem_bank.data = utils::open_file("rom/chargen.rom", 0);
                mem_bank.offset = 0xD000;
            },
            MemType::KERNAL  => {
                mem_bank.data = utils::open_file("rom/kernal.rom", 0);
                mem_bank.offset = 0xE000;
            },
            MemType::RAM => {
                mem_bank.data = Vec::<u8>::with_capacity(0x10000);
                for _ in 0..0x10000
                {
                    mem_bank.data.push(0);
                }
                
                mem_bank.read_only = false;
            }
            MemType::IO => {
                mem_bank.data = Vec::<u8>::with_capacity(0x1000);
                for _ in 0..0x1000
                {
                    mem_bank.data.push(0);
                }

                mem_bank.offset = 0xD000;
                mem_bank.read_only = false;
            }
        }
        
        mem_bank
    }

    pub fn write(&mut self, addr: u16, val: u8)
    {
        match self.bank_type
        {
            MemType::RAM => self.data[(addr - self.offset) as usize] = val,
            MemType::IO => {
                // TODO: IO access has specific behavior depending on address
                match addr
                {
                    0xD016          => self.data[(addr - self.offset) as usize] = 0xC0 | val,
                    0xD019          => self.data[(addr - self.offset) as usize] = 0x70 | val,
                    0xD01A          => self.data[(addr - self.offset) as usize] = 0xF0 | val,
                    0xD01E...0xD01F => (),                             // cannot be written - panic/fail on try?
                    0xD020...0xD02E => self.data[(addr - self.offset) as usize] = 0xF0 | val,
                    0xD02F...0xD03F => (),                             // write ignored
                    0xD040...0xD3FF => self.write(0xD000 + (addr % 0x0040), val), // same as 0xD000-0xD03F
                    _ => self.data[(addr - self.offset) as usize] = val
                }
                
            },
            _ => panic!("Can't write to ROM!")
        }
    }

    pub fn read(&mut self, addr: u16) -> u8
    {
        match self.bank_type
        {
            MemType::IO => {
                // TODO: IO access has specific behavior depending on address
                match addr
                {
                    0xD016          => 0xC0 | self.data[(addr - self.offset) as usize],
                    0xD018          => 0x01 | self.data[(addr - self.offset) as usize],
                    0xD019          => 0x70 | self.data[(addr - self.offset) as usize],
                    0xD01A          => 0xF0 | self.data[(addr - self.offset) as usize],
                    0xD01E...0xD01F => {                                  // cannot be written, cleared on read
                        let value = self.data[(addr - self.offset) as usize];
                        self.data[(addr - self.offset) as usize] = 0;
                        value
                    },
                    0xD020...0xD02E => 0xF0 | self.data[(addr - self.offset) as usize],
                    0xD02F...0xD03F => 0xFF,                              // always returns 0xFF
                    0xD040...0xD3FF => self.read(0xD000 + (addr % 0x0040)),          // same as 0xD000-0xD03F
                    _ => self.data[(addr - self.offset) as usize]
                }
            },
            _ => self.data[(addr - self.offset) as usize]
        }
    }    
}

/*impl Index<u16> for MemBank
{
    type Output = u8;

    fn index<'a>(&'a self, _index: u16) -> &'a u8
    {
        &self.data[_index as usize]
    }
}

impl Deref for MemBank
{
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8>
    {
        &self.data
    }
}

impl DerefMut for MemBank
{
    fn deref_mut(&mut self) -> &mut Vec<u8>
    {
        &mut self.data
    }
}*/



// collective memory storage with all the banks and bank switching support
pub struct Memory
{
    ram:     MemBank,
    basic:   MemBank,
    chargen: MemBank,
    io:      MemBank,
    kernal:  MemBank,

    // bank switching flags
    basic_on:   bool,
    chargen_on: bool,
    io_on:      bool,
    kernal_on:  bool,
    cart_lo_on: bool, // cart flag - unused for now
    cart_hi_on: bool  // cart flag - unused for now
}

impl Memory
{
    pub fn new_shared() -> MemShared
    {
        Rc::new(RefCell::new(Memory
        {
            ram:     MemBank::new(MemType::RAM),     // 64k
            basic:   MemBank::new(MemType::BASIC),   // 8k
            chargen: MemBank::new(MemType::CHARGEN), // 4k
            io:      MemBank::new(MemType::IO),      // 4k (VIC, SID, CIA, Color RAM)
            kernal:  MemBank::new(MemType::KERNAL),  // 8k
            basic_on:   false,
            chargen_on: false,
            io_on:      false,
            kernal_on:  false,
            cart_lo_on: false, // unused for now
            cart_hi_on: false, // unused for now
        }))
    }
    
    // returns memory bank for current latch setting and address
    pub fn get_bank(&mut self, addr: u16) -> (&mut MemBank)
    {
        match addr
        {
            0x0000...0x9FFF => &mut self.ram,
            0xA000...0xCFFF => if self.basic_on { &mut self.basic } else { &mut self.ram },
            0xD000...0xDFFF => {
                if self.chargen_on { return &mut self.chargen }
                if self.io_on      { return &mut self.io; }
                return &mut self.ram;
            },
            0xE000...0xFFFF => if self.kernal_on  { &mut self.kernal } else { &mut self.ram },
            _ => panic!("Address out of memory range")
        }
    }

    // returns specific modifiable memory bank
    pub fn get_ram_bank(&mut self, bank_type: MemType) -> (&mut MemBank)
    {
        match bank_type
        {
            MemType::RAM => &mut self.ram,
            MemType::IO  => &mut self.io,
            _            => panic!("Unrecognized RAM bank"),
        }
    }

    // returns specific non-modifiable memory bank
    pub fn get_rom_bank(&mut self, bank_type: MemType) -> (&mut MemBank)
    {
        match bank_type
        {
            MemType::BASIC   => &mut self.basic,
            MemType::CHARGEN => &mut self.chargen,
            MemType::KERNAL  => &mut self.kernal,
            _                => panic!("Unrecognized ROM Abank"),
        }
    }    
    
    pub fn reset(&mut self)
    {
        self.write_byte(0x0000, 0xFF);
        self.write_byte(0x0001, 0x07); // enable kernal, chargen and basic ROMs
    }

    fn update_bank_flags(&mut self)
    {
        // latch state is determined by 3 least significant bits from this location
        let latch = self.ram.read(0x0001) & 0x07;

        self.chargen_on = ((latch & 0x04) == 0) && ((latch & 0x03) != 0); // %0xx except %000
        self.io_on      = ((latch & 0x04) != 0) && ((latch & 0x03) != 0); // %1xx except %100
        self.basic_on   = (latch & 0x03) == 0x03;
        self.kernal_on  = self.basic_on || ((latch & 0x03) == 0x02);

        if self.io_on == false { panic!("oops"); }
    }
    
    // Write a byte to memory - returns whether RAM was written (true) or RAM under ROM (false)
    pub fn write_byte(&mut self, addr: u16, value: u8) -> bool
    {
            // RAM under ROM written? Return false to let us know about it
            if self.get_bank(addr).read_only
            {
                self.ram.write(addr, value);
                return false;
            }
            else
            {
                self.get_bank(addr).write(addr, value);
            }

        // update the bank switching flags here, since they can only change on memory write
        // latch byte changed - update bank switching flags
        if addr == 0x0001 { self.update_bank_flags(); }
        
        return true;
    }
    
    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        self.get_bank(addr).read(addr)
    }

    // Read a word from memory (stored in little endian)
    pub fn read_word_le(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);
        let value_be: u16 = ((bank.read(addr) as u16) << 8 & 0xFF00) |
                            ((bank.read(addr + 0x0001) as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }

    // Read a word from memory (stored in big endian: swap low<->high)
    pub fn read_word_be(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);
        let value_le: u16 = ((bank.read(addr) as u16) << 8 & 0xFF00) |
                            ((bank.read(addr + 0x0001) as u16) & 0x00FF);
        value_le
    }

    // Write word in little endian format (low/high)
    pub fn write_word_le(&mut self, addr: u16, value: u16) -> bool
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        let hi = self.write_byte(addr, value_le_lo);
        let lo = self.write_byte(addr + 0x0001, value_le_hi);

        return hi && lo;
    }

    // Write word in big endian format (high/low)
    pub fn write_word_be(&mut self, addr: u16, value: u16) -> bool
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        let hi = self.write_byte(addr, value_le_hi);
        let lo = self.write_byte(addr + 0x0001, value_le_lo);

        return hi && lo;
    }
}

