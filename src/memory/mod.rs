#![allow(dead_code)]
use utils;
pub struct Memory
{
    ram: [u8;0x10000],
    rom: [u8;0x10000],
    // bank switching flags
    basic_on: bool,
    chargen_on: bool,
    io_on: bool,
    kernal_on: bool,
    cart_lo_on: bool, // cart flag - unused for now
    cart_hi_on: bool  // cart flag - unused for now
}

impl Memory
{
    pub fn new() -> Memory
    {
        let mut memory = Memory
        {
            ram: [0;0x10000],   // 64k
            rom: [0;0x10000],   // store ROM data in 64k array so it's easier to address it with bank switching
            basic_on: false,
            chargen_on: false,
            io_on: false,
            kernal_on: false,
            cart_lo_on: false, // unused for now
            cart_hi_on: false, // unused for now
        };

        // load basic
        let mut start_addr: u32 = 0xA000;
        let basic = utils::open_file("rom/basic.rom", 0);
        
        for (i,addr) in (start_addr..0xC000).enumerate()
        {
            memory.rom[addr as usize] = basic[i as usize];
        }

        // load chargen
        start_addr = 0xD000;
        let chargen = utils::open_file("rom/chargen.rom", 0);
        
        for (i,addr) in (start_addr..0xE000).enumerate()
        {
            memory.rom[addr as usize] = chargen[i as usize];
        }
              
        // load kernal
        start_addr = 0xE000;
        let kernal = utils::open_file("rom/kernal.rom", 0);
        
        for (i,addr) in (start_addr..0x10000).enumerate()
        {
            memory.rom[addr as usize] = kernal[i as usize];
        }
        
        memory
    }
    
    // returns memory bank for current address + latch setting and an indicator whether it's read-only to detect ROM writes
    pub fn get_bank(&mut self, addr: u16) -> (&mut [u8], bool)
    {
        match addr
        {
            0x0000...0x9FFF => (&mut self.ram[0..0x10000], false),
            0xA000...0xCFFF => if self.basic_on   { (&mut self.rom[0..0x10000], true) } else { (&mut self.ram[0..0x10000], false) },
            0xD000...0xDFFF => if self.chargen_on { (&mut self.rom[0..0x10000], true) } else { (&mut self.ram[0..0x10000], false) },
            0xE000...0xFFFF => if self.kernal_on  { (&mut self.rom[0..0x10000], true) } else { (&mut self.ram[0..0x10000], false) },
            _ => panic!("Address out of memory range")
        }
    }
    

    pub fn reset(&mut self)
    {
        self.write_byte(0x0000, 0xFF);
        self.write_byte(0x0001, 0x07); // enable kernal, chargen and basic access
    }

    fn update_bank_flags(&mut self)
    {
        // latch state is determined by 5 least significant bits from this location
        let latch = self.ram[0x0001] & 0x1F;

        // default to RAM only
        self.basic_on = false;
        self.chargen_on = false;
        self.io_on = false;
        self.kernal_on = false;
        self.cart_lo_on = false;
        self.cart_hi_on = false;
        
        match latch
        {
            0x02 => {
                self.chargen_on = true;
                self.kernal_on = true;
                self.cart_hi_on = true;
            },
            0x03 => {
                self.chargen_on = true;
                self.kernal_on = true;
                self.cart_lo_on = true;
                self.cart_hi_on = true;
            }
            0x05 | 0x0D | 0x1D => {
                self.io_on = true;
            }
            0x06 => {
                self.cart_hi_on = true;
                self.io_on = true;
            },
            0x07 => {
                self.cart_lo_on = true;
                self.cart_hi_on = true;
                self.io_on = true;
                self.kernal_on = true;
            },
            0x0B => {
                self.cart_lo_on = true;
                self.basic_on = true;
                self.chargen_on = true;
                self.kernal_on = true;
            },
            0x0F => {
                self.cart_lo_on = true;
                self.basic_on = true;
                self.io_on = true;
                self.kernal_on = true;
            },
            0x10...0x17 => {
                self.cart_lo_on = true;
                self.io_on = true;
                self.cart_hi_on = true;
            },
            0x19 | 0x09 => {
                self.chargen_on = true;
            },
            0x1A | 0x0A => {
                self.chargen_on = true;
                self.kernal_on = true;
            },
            0x1B => {
                self.basic_on = true;
                self.chargen_on = true;
                self.kernal_on = true;
            },
            0xE | 0x1E => {
                self.io_on = true;
                self.kernal_on = true;
            },
            0x1F => {
                self.basic_on = true;
                self.io_on = true;
                self.kernal_on = true;
            },
            _ => ()
        }
    }
    
    // Write a byte to memory
    pub fn write_byte(&mut self, addr: u16, value: u8)
    {
        // parentheses to avoid borrowing issues with changing the flags
        {
            let (bank, read_only) = self.get_bank(addr);
            if read_only
            {
                println!("Tried to write 0x{:02X} to read-only memory at: ${:04X}", value, addr);
                return;
            }
            else
            {
                bank[addr as usize] = value;
            }
        }

        // update the bank switching flags here, since they can only change on memory write
        // latch byte changed - update bank switching flags
        if addr == 0x0001 { self.update_bank_flags(); }
    }
    
    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        let bank = self.get_bank(addr);
        bank.0[addr as usize]
    }

    // Read a word from memory (stored in little endian)
    pub fn read_word_le(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);   
        let value_be: u16 = ((bank.0[addr as usize] as u16) << 8 & 0xFF00) |
                            ((bank.0[(addr + 0x0001) as usize] as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }

    // Read a word from memory (stored in big endian: swap low<->high)
    pub fn read_word_be(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);
        let value_le: u16 = ((bank.0[addr as usize] as u16) << 8 & 0xFF00) |
                            ((bank.0[(addr + 0x0001) as usize] as u16) & 0x00FF);
        value_le
    }

    // Write word in little endian format (low/high)
    pub fn write_word_le(&mut self, addr: u16, value: u16)
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        self.write_byte(addr, value_le_lo);
        self.write_byte(addr + 0x0001, value_le_hi);
    }

    // Write word in big endian format (high/low)
    pub fn write_word_be(&mut self, addr: u16, value: u16)
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        self.write_byte(addr, value_le_hi);
        self.write_byte(addr + 0x0001, value_le_lo);
    }

    // Debug: overwrite a byte in ROM
    pub fn debug_write_rom(&mut self, addr: u16, value: u8)
    {
        self.rom[addr as usize] = value;
    }

}

